//! Order management and signal execution helpers.

pub mod algorithm;
pub mod orchestrator;
pub mod repository;

// Re-export key types for convenience
pub use algorithm::{AlgoStatus, ChildOrderRequest, ExecutionAlgorithm};
pub use orchestrator::OrderOrchestrator;
pub use repository::{AlgoStateRepository, SqliteAlgoStateRepository};

use anyhow::{anyhow, bail, Context};
use rust_decimal::{
    prelude::{FromPrimitive, ToPrimitive},
    Decimal,
};
use std::sync::Arc;
use tesser_broker::{BrokerError, BrokerResult, ExecutionClient};
use tesser_bybit::{BybitClient, BybitCredentials};
use tesser_core::{
    Order, OrderRequest, OrderType, Price, Quantity, Side, Signal, SignalKind, Symbol,
};
use thiserror::Error;
use tracing::{info, warn};

/// Determine how large an order should be for a given signal.
pub trait OrderSizer: Send + Sync {
    /// Calculate the desired base asset quantity.
    fn size(
        &self,
        signal: &Signal,
        portfolio_equity: f64,
        last_price: f64,
    ) -> anyhow::Result<Quantity>;
}

/// Simplest possible sizer that always returns a fixed size.
pub struct FixedOrderSizer {
    pub quantity: Quantity,
}

impl OrderSizer for FixedOrderSizer {
    fn size(
        &self,
        _signal: &Signal,
        _portfolio_equity: f64,
        _last_price: f64,
    ) -> anyhow::Result<Quantity> {
        Ok(self.quantity)
    }
}

/// Sizes orders based on a fixed percentage of portfolio equity.
pub struct PortfolioPercentSizer {
    /// The fraction of equity to allocate per trade (e.g., 0.02 for 2%).
    pub percent: f64,
}

impl OrderSizer for PortfolioPercentSizer {
    fn size(
        &self,
        _signal: &Signal,
        portfolio_equity: f64,
        last_price: f64,
    ) -> anyhow::Result<Quantity> {
        let price = decimal_from_f64(last_price, "last price")?;
        if price <= Decimal::ZERO {
            bail!("cannot size order with zero or negative price");
        }
        let equity = decimal_from_f64(portfolio_equity, "portfolio equity")?;
        let percent = decimal_from_f64(self.percent, "allocation percent")?;
        if percent <= Decimal::ZERO {
            return Ok(0.0);
        }
        let notional = equity * percent;
        let quantity = notional / price;
        quantity_from_decimal(quantity, "order quantity")
    }
}

/// Sizes orders based on position volatility. (Placeholder)
#[derive(Default)]
pub struct RiskAdjustedSizer {
    /// Target risk contribution per trade, as a fraction of equity (e.g., 0.002 for 0.2%).
    pub risk_fraction: f64,
}

impl OrderSizer for RiskAdjustedSizer {
    fn size(
        &self,
        _signal: &Signal,
        portfolio_equity: f64,
        last_price: f64,
    ) -> anyhow::Result<Quantity> {
        let price = decimal_from_f64(last_price, "last price")?;
        if price <= Decimal::ZERO {
            bail!("cannot size order with zero or negative price");
        }
        let equity = decimal_from_f64(portfolio_equity, "portfolio equity")?;
        let risk_fraction = decimal_from_f64(self.risk_fraction, "risk fraction")?;
        if risk_fraction <= Decimal::ZERO {
            return Ok(0.0);
        }
        // Placeholder volatility; replace with instrument-specific estimator.
        let volatility = Decimal::from_f64(0.02).expect("0.02 should convert to Decimal");
        let denom = price * volatility;
        if denom <= Decimal::ZERO {
            bail!("volatility multiplier produced an invalid denominator");
        }
        let dollars_at_risk = equity * risk_fraction;
        let quantity = dollars_at_risk / denom;
        quantity_from_decimal(quantity, "risk-adjusted quantity")
    }
}

/// Context passed to risk checks describing current exposure state.
#[derive(Clone, Copy, Debug, Default)]
pub struct RiskContext {
    /// Signed quantity of the current open position (long positive, short negative).
    pub signed_position_qty: f64,
    /// Total current portfolio equity.
    pub portfolio_equity: Price,
    /// Last known price for the signal's symbol.
    pub last_price: Price,
    /// When true, only exposure-reducing orders are allowed.
    pub liquidate_only: bool,
}

/// Validates an order before it reaches the broker.
pub trait PreTradeRiskChecker: Send + Sync {
    /// Return `Ok(())` if the order passes risk checks.
    fn check(&self, request: &OrderRequest, ctx: &RiskContext) -> Result<(), RiskError>;
}

/// No-op risk checker used by tests/backtests.
pub struct NoopRiskChecker;

impl PreTradeRiskChecker for NoopRiskChecker {
    fn check(&self, _request: &OrderRequest, _ctx: &RiskContext) -> Result<(), RiskError> {
        Ok(())
    }
}

/// Upper bounds enforced by the [`BasicRiskChecker`].
#[derive(Clone, Copy, Debug)]
pub struct RiskLimits {
    pub max_order_quantity: f64,
    pub max_position_quantity: f64,
}

impl RiskLimits {
    /// Ensure limits are non-negative and default to zero (disabled) when NaN.
    pub fn sanitized(self) -> Self {
        Self {
            max_order_quantity: self.max_order_quantity.max(0.0),
            max_position_quantity: self.max_position_quantity.max(0.0),
        }
    }
}

/// Simple risk checker enforcing fat-finger order size limits plus position caps.
pub struct BasicRiskChecker {
    limits: RiskLimits,
}

impl BasicRiskChecker {
    /// Build a new checker with the provided limits.
    pub fn new(limits: RiskLimits) -> Self {
        Self {
            limits: limits.sanitized(),
        }
    }
}

impl PreTradeRiskChecker for BasicRiskChecker {
    fn check(&self, request: &OrderRequest, ctx: &RiskContext) -> Result<(), RiskError> {
        let qty = request.quantity.abs();
        if self.limits.max_order_quantity > 0.0 && qty > self.limits.max_order_quantity {
            return Err(RiskError::MaxOrderSize {
                quantity: qty,
                limit: self.limits.max_order_quantity,
            });
        }

        let projected_position = match request.side {
            Side::Buy => ctx.signed_position_qty + qty,
            Side::Sell => ctx.signed_position_qty - qty,
        };

        if self.limits.max_position_quantity > 0.0
            && projected_position.abs() > self.limits.max_position_quantity
        {
            return Err(RiskError::MaxPositionExposure {
                projected: projected_position,
                limit: self.limits.max_position_quantity,
            });
        }

        if ctx.liquidate_only {
            let position = ctx.signed_position_qty;
            if position.abs() < f64::EPSILON {
                return Err(RiskError::LiquidateOnly);
            }
            let reduces = (position > 0.0 && request.side == Side::Sell)
                || (position < 0.0 && request.side == Side::Buy);
            if !reduces {
                return Err(RiskError::LiquidateOnly);
            }
            if qty > position.abs() + f64::EPSILON {
                return Err(RiskError::LiquidateOnly);
            }
        }

        Ok(())
    }
}

fn decimal_from_f64(value: f64, label: &str) -> anyhow::Result<Decimal> {
    if !value.is_finite() {
        bail!("{label} must be finite (got {value})");
    }
    Decimal::from_f64(value)
        .or_else(|| Decimal::from_f64_retain(value))
        .ok_or_else(|| anyhow!("failed to convert {label} ({value}) into Decimal"))
}

fn quantity_from_decimal(value: Decimal, label: &str) -> anyhow::Result<Quantity> {
    value
        .to_f64()
        .ok_or_else(|| anyhow!("failed to convert {label} ({value}) into f64"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tesser_core::SignalKind;

    fn dummy_signal() -> Signal {
        Signal::new("BTCUSDT", SignalKind::EnterLong, 1.0)
    }

    #[test]
    fn portfolio_percent_sizer_matches_decimal_math() {
        let signal = dummy_signal();
        let sizer = PortfolioPercentSizer { percent: 0.05 };
        let qty = sizer.size(&signal, 25_000.0, 50_000.0).unwrap();
        assert!((qty - 0.025).abs() < f64::EPSILON);
    }

    #[test]
    fn risk_adjusted_sizer_respects_zero_price_guard() {
        let signal = dummy_signal();
        let sizer = RiskAdjustedSizer {
            risk_fraction: 0.01,
        };
        let err = sizer.size(&signal, 10_000.0, 0.0).unwrap_err();
        assert!(
            err.to_string().contains("zero or negative price"),
            "unexpected error: {err}"
        );
    }
}

/// Errors surfaced by pre-trade risk checks.
#[derive(Debug, Error)]
pub enum RiskError {
    #[error("order quantity {quantity:.4} exceeds limit {limit:.4}")]
    MaxOrderSize { quantity: f64, limit: f64 },
    #[error("projected position {projected:.4} exceeds limit {limit:.4}")]
    MaxPositionExposure { projected: f64, limit: f64 },
    #[error("liquidate-only mode active")]
    LiquidateOnly,
}

/// Translates signals into orders using a provided [`ExecutionClient`].
pub struct ExecutionEngine {
    client: Arc<dyn ExecutionClient>,
    sizer: Box<dyn OrderSizer>,
    risk: Arc<dyn PreTradeRiskChecker>,
}

impl ExecutionEngine {
    /// Instantiate the engine with its dependencies.
    pub fn new(
        client: Arc<dyn ExecutionClient>,
        sizer: Box<dyn OrderSizer>,
        risk: Arc<dyn PreTradeRiskChecker>,
    ) -> Self {
        Self {
            client,
            sizer,
            risk,
        }
    }

    /// Consume a signal and forward it to the broker.
    pub async fn handle_signal(
        &self,
        signal: Signal,
        ctx: RiskContext,
    ) -> BrokerResult<Option<Order>> {
        let qty = self
            .sizer
            .size(&signal, ctx.portfolio_equity, ctx.last_price)
            .context("failed to determine order size")
            .map_err(|err| BrokerError::Other(err.to_string()))?;

        if qty <= 0.0 {
            warn!(signal = ?signal.id, "order size is zero, skipping");
            return Ok(None);
        }

        let client_order_id = signal.id.to_string();
        let request = match signal.kind {
            SignalKind::EnterLong => self.build_request(
                signal.symbol.clone(),
                Side::Buy,
                qty,
                Some(client_order_id.clone()),
            ),
            SignalKind::ExitLong | SignalKind::Flatten => self.build_request(
                signal.symbol.clone(),
                Side::Sell,
                qty,
                Some(client_order_id.clone()),
            ),
            SignalKind::EnterShort => self.build_request(
                signal.symbol.clone(),
                Side::Sell,
                qty,
                Some(client_order_id.clone()),
            ),
            SignalKind::ExitShort => self.build_request(
                signal.symbol.clone(),
                Side::Buy,
                qty,
                Some(client_order_id.clone()),
            ),
        };

        let order = self.send_order(request, &ctx).await?;

        let stop_side = match signal.kind {
            SignalKind::EnterLong | SignalKind::ExitShort => Side::Sell,
            SignalKind::EnterShort | SignalKind::ExitLong => Side::Buy,
            SignalKind::Flatten => return Ok(Some(order)),
        };

        if let Some(sl_price) = signal.stop_loss {
            let sl_request = OrderRequest {
                symbol: signal.symbol.clone(),
                side: stop_side,
                order_type: OrderType::StopMarket,
                quantity: qty,
                price: None,
                trigger_price: Some(sl_price),
                time_in_force: None,
                client_order_id: Some(format!("{}-sl", signal.id)),
                take_profit: None,
                stop_loss: None,
                display_quantity: None,
            };
            if let Err(e) = self.send_order(sl_request, &ctx).await {
                warn!(error = %e, "failed to place stop-loss order");
            }
        }

        if let Some(tp_price) = signal.take_profit {
            let tp_request = OrderRequest {
                symbol: signal.symbol.clone(),
                side: stop_side,
                order_type: OrderType::StopMarket,
                quantity: qty,
                price: None,
                trigger_price: Some(tp_price),
                time_in_force: None,
                client_order_id: Some(format!("{}-tp", signal.id)),
                take_profit: None,
                stop_loss: None,
                display_quantity: None,
            };
            if let Err(e) = self.send_order(tp_request, &ctx).await {
                warn!(error = %e, "failed to place take-profit order");
            }
        }

        Ok(Some(order))
    }

    fn build_request(
        &self,
        symbol: Symbol,
        side: Side,
        qty: Quantity,
        client_order_id: Option<String>,
    ) -> OrderRequest {
        OrderRequest {
            symbol,
            side,
            order_type: OrderType::Market,
            quantity: qty,
            price: None,
            trigger_price: None,
            time_in_force: None,
            client_order_id,
            take_profit: None,
            stop_loss: None,
            display_quantity: None,
        }
    }

    async fn send_order(&self, request: OrderRequest, ctx: &RiskContext) -> BrokerResult<Order> {
        self.risk
            .check(&request, ctx)
            .map_err(|err| BrokerError::InvalidRequest(err.to_string()))?;
        let order = self.client.place_order(request).await?;
        info!(order_id = %order.id, qty = order.request.quantity, "order sent to broker");
        Ok(order)
    }

    pub fn client(&self) -> Arc<dyn ExecutionClient> {
        Arc::clone(&self.client)
    }

    pub fn sizer(&self) -> &dyn OrderSizer {
        self.sizer.as_ref()
    }

    pub fn credentials(&self) -> Option<BybitCredentials> {
        self.client
            .as_any()
            .downcast_ref::<BybitClient>()
            .and_then(|client| client.get_credentials())
    }

    pub fn ws_url(&self) -> String {
        self.client
            .as_any()
            .downcast_ref::<BybitClient>()
            .map(|client| client.get_ws_url())
            .unwrap_or_default()
    }
}
