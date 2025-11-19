use anyhow::{anyhow, Result};
use std::future::Future;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use tokio::runtime::{Builder, Runtime};
use tonic::transport::{Channel, Endpoint};
use tracing::debug;

use crate::client::RemoteStrategyClient;
use crate::proto::strategy_service_client::StrategyServiceClient;
use crate::proto::{
    CandleRequest, FillRequest, InitRequest, InitResponse, OrderBookRequest, SignalList,
    TickRequest,
};

/// A gRPC-based implementation of the strategy client.
pub struct GrpcAdapter {
    endpoint: String,
    client: Option<StrategyServiceClient<Channel>>,
    timeout: Duration,
    runtime: Option<Runtime>,
}

impl GrpcAdapter {
    pub fn new(endpoint: String, timeout_ms: u64) -> Self {
        let runtime = Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .expect("failed to create RPC runtime");

        Self {
            endpoint,
            client: None,
            timeout: Duration::from_millis(timeout_ms.max(1)),
            runtime: Some(runtime),
        }
    }

    fn client(&self) -> Result<StrategyServiceClient<Channel>> {
        self.client
            .clone()
            .ok_or_else(|| anyhow!("gRPC client not connected"))
    }

    fn block_on_task<F, T>(&self, fut: F) -> Result<T>
    where
        F: Future<Output = Result<T, anyhow::Error>> + Send + 'static,
        T: Send + 'static,
    {
        let (tx, rx) = mpsc::channel();
        let runtime = self.runtime.as_ref().expect("runtime not initialized");
        runtime.spawn(async move {
            let _ = tx.send(fut.await);
        });

        rx.recv().map_err(|e| anyhow!(e.to_string()))?
    }

    fn dispatch<F, Fut, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(StrategyServiceClient<Channel>, Duration) -> Fut + Send + 'static,
        Fut: Future<Output = Result<T, tonic::Status>> + Send + 'static,
        T: Send + 'static,
    {
        let client = self.client()?;
        let timeout = self.timeout;
        self.block_on_task(async move { f(client, timeout).await.map_err(|e| anyhow!(e)) })
    }
}

impl RemoteStrategyClient for GrpcAdapter {
    fn connect(&mut self) -> Result<()> {
        debug!("connecting to gRPC strategy at {}", self.endpoint);
        let endpoint = self.endpoint.clone();
        let timeout = self.timeout;
        let client = self.block_on_task(async move {
            let channel = Endpoint::from_shared(endpoint)
                .map_err(|e| anyhow!(e))?
                .connect_timeout(timeout)
                .timeout(timeout)
                .connect()
                .await
                .map_err(|e| anyhow!(e))?;
            Ok(StrategyServiceClient::new(channel))
        })?;

        self.client = Some(client);
        Ok(())
    }

    fn initialize(&mut self, req: InitRequest) -> Result<InitResponse> {
        self.dispatch(|mut client, timeout| async move {
            let mut request = tonic::Request::new(req);
            request.set_timeout(timeout);
            let response = client.initialize(request).await?;
            Ok(response.into_inner())
        })
    }

    fn on_tick(&mut self, req: TickRequest) -> Result<SignalList> {
        self.dispatch(|mut client, timeout| async move {
            let mut request = tonic::Request::new(req);
            request.set_timeout(timeout);
            let response = client.on_tick(request).await?;
            Ok(response.into_inner())
        })
    }

    fn on_candle(&mut self, req: CandleRequest) -> Result<SignalList> {
        self.dispatch(|mut client, timeout| async move {
            let mut request = tonic::Request::new(req);
            request.set_timeout(timeout);
            let response = client.on_candle(request).await?;
            Ok(response.into_inner())
        })
    }

    fn on_order_book(&mut self, req: OrderBookRequest) -> Result<SignalList> {
        self.dispatch(|mut client, timeout| async move {
            let mut request = tonic::Request::new(req);
            request.set_timeout(timeout);
            let response = client.on_order_book(request).await?;
            Ok(response.into_inner())
        })
    }

    fn on_fill(&mut self, req: FillRequest) -> Result<SignalList> {
        self.dispatch(|mut client, timeout| async move {
            let mut request = tonic::Request::new(req);
            request.set_timeout(timeout);
            let response = client.on_fill(request).await?;
            Ok(response.into_inner())
        })
    }
}

impl Drop for GrpcAdapter {
    fn drop(&mut self) {
        if let Some(runtime) = self.runtime.take() {
            thread::spawn(move || drop(runtime));
        }
    }
}
