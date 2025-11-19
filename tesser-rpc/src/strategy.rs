use async_trait::async_trait;
use serde::Deserialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tesser_core::{Candle, Fill, OrderBook, Signal, Symbol, Tick};
use tesser_strategy::{
    register_strategy, Strategy, StrategyContext, StrategyError, StrategyResult,
};
use tokio::sync::Mutex as AsyncMutex;
use tokio::task::JoinHandle;
use tokio::time::{interval, MissedTickBehavior};
use tracing::{error, info, warn};

use crate::client::RemoteStrategyClient;
use crate::proto::{CandleRequest, FillRequest, InitRequest, OrderBookRequest, TickRequest};
use crate::transport::grpc::GrpcAdapter;

#[derive(Clone, Deserialize)]
#[serde(tag = "transport")]
enum TransportConfig {
    #[serde(rename = "grpc")]
    Grpc {
        endpoint: String,
        #[serde(default = "default_timeout_ms")]
        timeout_ms: u64,
    },
    // Future expansion: ZMQ, SHM, etc.
}

fn default_timeout_ms() -> u64 {
    500
}

type SharedClient = Arc<AsyncMutex<Box<dyn RemoteStrategyClient>>>;

const DEFAULT_HEARTBEAT_INTERVAL: Duration = Duration::from_millis(5_000);
const MAX_HEARTBEAT_FAILURES: u32 = 3;

/// A strategy adapter that delegates decision making to an external service via a pluggable transport.
pub struct RpcStrategy {
    client: Option<SharedClient>,
    transport_config: Option<TransportConfig>,
    config_payload: String,
    subscriptions: Vec<String>,
    pending_signals: Vec<Signal>,
    symbol: String, // Primary symbol fallback
    health: Arc<AtomicBool>,
    heartbeat_handle: Option<JoinHandle<()>>,
    heartbeat_interval: Duration,
    max_heartbeat_failures: u32,
}

impl Default for RpcStrategy {
    fn default() -> Self {
        Self {
            client: None,
            transport_config: None,
            config_payload: "{}".to_string(),
            subscriptions: vec![],
            pending_signals: vec![],
            symbol: "UNKNOWN".to_string(),
            health: Arc::new(AtomicBool::new(true)),
            heartbeat_handle: None,
            heartbeat_interval: DEFAULT_HEARTBEAT_INTERVAL,
            max_heartbeat_failures: MAX_HEARTBEAT_FAILURES,
        }
    }
}

impl RpcStrategy {
    fn build_client(config: &TransportConfig) -> Box<dyn RemoteStrategyClient> {
        match config {
            TransportConfig::Grpc {
                endpoint,
                timeout_ms,
            } => {
                info!(target: "rpc", endpoint, "configured gRPC transport");
                Box::new(GrpcAdapter::new(endpoint.clone(), *timeout_ms))
            }
        }
    }

    fn teardown_client(&mut self) {
        if let Some(handle) = self.heartbeat_handle.take() {
            handle.abort();
        }
        self.client = None;
        self.health.store(false, Ordering::Relaxed);
    }

    fn spawn_heartbeat(&mut self, client: SharedClient) {
        if let Some(handle) = self.heartbeat_handle.take() {
            handle.abort();
        }
        let interval_duration = self.heartbeat_interval;
        let max_failures = self.max_heartbeat_failures;
        let health = self.health.clone();
        self.heartbeat_handle = Some(tokio::spawn(async move {
            let mut ticker = interval(interval_duration);
            ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);
            let mut failures = 0u32;
            loop {
                ticker.tick().await;
                let mut guard = client.lock().await;
                match guard.heartbeat().await {
                    Ok(resp) if resp.healthy => {
                        health.store(true, Ordering::Relaxed);
                        failures = 0;
                    }
                    Ok(resp) => {
                        warn!(
                            target: "rpc",
                            status = %resp.status_msg,
                            "heartbeat reported unhealthy"
                        );
                        failures += 1;
                        health.store(false, Ordering::Relaxed);
                    }
                    Err(err) => {
                        warn!(target: "rpc", %err, "heartbeat failure");
                        failures += 1;
                        health.store(false, Ordering::Relaxed);
                    }
                }

                if failures >= max_failures {
                    error!(
                        target: "rpc",
                        failures,
                        "heartbeat exceeded failure threshold"
                    );
                    break;
                }
            }
        }));
    }

    async fn ensure_client(&mut self) -> StrategyResult<SharedClient> {
        if let Some(handle) = &self.client {
            if self.health.load(Ordering::Relaxed) {
                return Ok(handle.clone());
            }
            self.teardown_client();
        }

        if self.client.is_none() {
            let config = self
                .transport_config
                .clone()
                .ok_or_else(|| StrategyError::InvalidConfig("transport config missing".into()))?;

            let mut client = Self::build_client(&config);

            client
                .connect()
                .await
                .map_err(|e| StrategyError::Internal(format!("RPC connect failed: {e}")))?;

            let init_request = InitRequest {
                config_json: self.config_payload.clone(),
            };

            let response = client.initialize(init_request).await.map_err(|e| {
                StrategyError::Internal(format!("remote strategy init failed: {e}"))
            })?;

            if !response.success {
                return Err(StrategyError::Internal(format!(
                    "remote strategy rejected init: {}",
                    response.error_message
                )));
            }

            self.apply_remote_metadata(response.symbols);
            info!(target: "rpc", symbols = ?self.subscriptions, "RPC strategy initialized");
            self.health.store(true, Ordering::Relaxed);
            let shared = Arc::new(AsyncMutex::new(client));
            self.spawn_heartbeat(shared.clone());
            self.client = Some(shared.clone());
            return Ok(shared);
        }

        self.client
            .as_ref()
            .cloned()
            .ok_or_else(|| StrategyError::Internal("RPC client not initialized".into()))
    }

    fn apply_remote_metadata(&mut self, mut symbols: Vec<String>) {
        if symbols.is_empty() {
            symbols.push(self.symbol.clone());
        }
        if let Some(primary) = symbols.first() {
            self.symbol = primary.clone();
        }
        self.subscriptions = symbols;
    }

    fn handle_signals(&mut self, signals: Vec<crate::proto::Signal>) {
        for proto_sig in signals {
            self.pending_signals.push(proto_sig.into());
        }
    }
}

#[async_trait]
impl Strategy for RpcStrategy {
    fn name(&self) -> &str {
        "rpc-strategy"
    }

    fn symbol(&self) -> &str {
        &self.symbol
    }

    fn subscriptions(&self) -> Vec<Symbol> {
        if self.subscriptions.is_empty() {
            vec![self.symbol.clone()]
        } else {
            self.subscriptions.clone()
        }
    }

    fn configure(&mut self, params: toml::Value) -> StrategyResult<()> {
        let config: TransportConfig = params.clone().try_into().map_err(|e| {
            StrategyError::InvalidConfig(format!("failed to parse RPC config: {}", e))
        })?;

        self.transport_config = Some(config);
        self.teardown_client();
        self.subscriptions.clear();
        self.symbol = "UNKNOWN".to_string();
        self.pending_signals.clear();
        self.config_payload = serde_json::to_string(&params).unwrap_or_else(|_| "{}".to_string());
        Ok(())
    }

    async fn on_tick(&mut self, ctx: &StrategyContext, tick: &Tick) -> StrategyResult<()> {
        let request = TickRequest {
            tick: Some(tick.clone().into()),
            context: Some(ctx.into()),
        };

        let client = self.ensure_client().await?;
        let mut transport = client.lock().await;
        match transport.on_tick(request).await {
            Ok(response) => self.handle_signals(response.signals),
            Err(e) => error!("RPC OnTick error: {}", e),
        }
        Ok(())
    }

    async fn on_candle(&mut self, ctx: &StrategyContext, candle: &Candle) -> StrategyResult<()> {
        let request = CandleRequest {
            candle: Some(candle.clone().into()),
            context: Some(ctx.into()),
        };

        let client = self.ensure_client().await?;
        let mut transport = client.lock().await;
        match transport.on_candle(request).await {
            Ok(response) => self.handle_signals(response.signals),
            Err(e) => error!("RPC OnCandle error: {}", e),
        }
        Ok(())
    }

    async fn on_fill(&mut self, ctx: &StrategyContext, fill: &Fill) -> StrategyResult<()> {
        let request = FillRequest {
            fill: Some(fill.clone().into()),
            context: Some(ctx.into()),
        };

        let client = self.ensure_client().await?;
        let mut transport = client.lock().await;
        match transport.on_fill(request).await {
            Ok(response) => self.handle_signals(response.signals),
            Err(e) => error!("RPC OnFill error: {}", e),
        }
        Ok(())
    }

    async fn on_order_book(
        &mut self,
        ctx: &StrategyContext,
        book: &OrderBook,
    ) -> StrategyResult<()> {
        let request = OrderBookRequest {
            order_book: Some(book.clone().into()),
            context: Some(ctx.into()),
        };

        let client = self.ensure_client().await?;
        let mut transport = client.lock().await;
        match transport.on_order_book(request).await {
            Ok(response) => self.handle_signals(response.signals),
            Err(e) => error!("RPC OnOrderBook error: {}", e),
        }
        Ok(())
    }

    fn drain_signals(&mut self) -> Vec<Signal> {
        std::mem::take(&mut self.pending_signals)
    }
}

register_strategy!(RpcStrategy, "RpcStrategy");

impl Drop for RpcStrategy {
    fn drop(&mut self) {
        self.teardown_client();
    }
}
