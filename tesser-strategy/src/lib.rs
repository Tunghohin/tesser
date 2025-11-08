//! Strategy trait definitions, shared context, and reference implementations.

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};
use tesser_core::{Candle, Fill, Position, Signal, Symbol, Tick};
use thiserror::Error;

/// Result alias used within strategy implementations.
pub type StrategyResult<T> = Result<T, StrategyError>;

/// Failure variants surfaced by strategies.
#[derive(Debug, Error)]
pub enum StrategyError {
    /// Raised when a strategy's configuration cannot be parsed or is invalid.
    #[error("configuration is invalid: {0}")]
    InvalidConfig(String),
    /// Raised when the strategy lacks sufficient historical data to proceed.
    #[error("not enough historical data to compute indicators")]
    NotEnoughData,
    /// Used for all other errors that should bubble up to the caller.
    #[error("an internal strategy error occurred: {0}")]
    Internal(String),
}

/// Immutable view of recent market data and portfolio state shared with strategies.
pub struct StrategyContext {
    recent_candles: VecDeque<Candle>,
    recent_ticks: VecDeque<Tick>,
    positions: Vec<Position>,
    max_history: usize,
}

impl StrategyContext {
    /// Create a new context keeping up to `max_history` candles/ticks in memory.
    pub fn new(max_history: usize) -> Self {
        Self {
            recent_candles: VecDeque::with_capacity(max_history),
            recent_ticks: VecDeque::with_capacity(max_history),
            positions: Vec::new(),
            max_history,
        }
    }

    /// Push a candle while respecting the configured history size.
    pub fn push_candle(&mut self, candle: Candle) {
        if self.recent_candles.len() >= self.max_history {
            self.recent_candles.pop_front();
        }
        self.recent_candles.push_back(candle);
    }

    /// Push a tick while respecting the configured history size.
    pub fn push_tick(&mut self, tick: Tick) {
        if self.recent_ticks.len() >= self.max_history {
            self.recent_ticks.pop_front();
        }
        self.recent_ticks.push_back(tick);
    }

    /// Replace the in-memory position snapshot.
    pub fn update_positions(&mut self, positions: Vec<Position>) {
        self.positions = positions;
    }

    /// Access recently observed candles.
    #[must_use]
    pub fn candles(&self) -> &VecDeque<Candle> {
        &self.recent_candles
    }

    /// Access recently observed ticks.
    #[must_use]
    pub fn ticks(&self) -> &VecDeque<Tick> {
        &self.recent_ticks
    }

    /// Access all tracked positions.
    #[must_use]
    pub fn positions(&self) -> &Vec<Position> {
        &self.positions
    }

    /// Find the position for a specific symbol, if any.
    #[must_use]
    pub fn position(&self, symbol: &Symbol) -> Option<&Position> {
        self.positions.iter().find(|p| &p.symbol == symbol)
    }
}

impl Default for StrategyContext {
    fn default() -> Self {
        Self::new(512)
    }
}

/// Strategy lifecycle hooks used by engines that drive market data and fills.
pub trait Strategy: Send + Sync {
    /// Human-friendly identifier used in logs and telemetry.
    fn name(&self) -> &str;

    /// The primary symbol operated on by the strategy.
    fn symbol(&self) -> &str;

    /// Called once before the strategy is registered, allowing it to parse parameters.
    fn configure(&mut self, params: toml::Value) -> StrategyResult<()>;

    /// Called whenever the data pipeline emits a new tick.
    fn on_tick(&mut self, ctx: &StrategyContext, tick: &Tick) -> StrategyResult<()>;

    /// Called whenever a candle is produced or replayed.
    fn on_candle(&mut self, ctx: &StrategyContext, candle: &Candle) -> StrategyResult<()>;

    /// Called whenever one of the strategy's orders is filled.
    fn on_fill(&mut self, ctx: &StrategyContext, fill: &Fill) -> StrategyResult<()>;

    /// Allows the strategy to emit one or more signals after processing events.
    fn drain_signals(&mut self) -> Vec<Signal>;
}

/// Example configuration for a moving-average crossover strategy.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct SmaCrossConfig {
    pub symbol: Symbol,
    pub fast_period: usize,
    pub slow_period: usize,
    pub min_samples: usize,
}

impl Default for SmaCrossConfig {
    fn default() -> Self {
        Self {
            symbol: "BTCUSDT".to_string(),
            fast_period: 5,
            slow_period: 20,
            min_samples: 25,
        }
    }
}

impl TryFrom<toml::Value> for SmaCrossConfig {
    type Error = StrategyError;

    fn try_from(value: toml::Value) -> Result<Self, Self::Error> {
        value.try_into().map_err(|err: toml::de::Error| {
            StrategyError::InvalidConfig(format!("failed to parse SmaCross config: {err}"))
        })
    }
}

/// Very small reference implementation that can be expanded later.
#[derive(Default)]
pub struct SmaCross {
    cfg: SmaCrossConfig,
    signals: Vec<Signal>,
}

impl SmaCross {
    /// Instantiate a new SMA cross strategy with the provided configuration.
    pub fn new(cfg: SmaCrossConfig) -> Self {
        Self {
            cfg,
            signals: Vec::new(),
        }
    }

    fn maybe_emit_signal(&mut self, candles: &VecDeque<Candle>) -> StrategyResult<()> {
        if candles.len() < self.cfg.min_samples
            || candles.len() < self.cfg.fast_period
            || candles.len() < self.cfg.slow_period
        {
            return Ok(());
        }

        let fast = Self::sma(candles, self.cfg.fast_period)?;
        let slow = Self::sma(candles, self.cfg.slow_period)?;

        if let (Some(fast_prev), Some(slow_prev)) = (fast.first(), slow.first()) {
            let fast_last = *fast.last().unwrap_or(fast_prev);
            let slow_last = *slow.last().unwrap_or(slow_prev);
            if fast_prev <= slow_prev && fast_last > slow_last {
                self.signals.push(Signal::new(
                    self.cfg.symbol.clone(),
                    tesser_core::SignalKind::EnterLong,
                    0.75,
                ));
            } else if fast_prev >= slow_prev && fast_last < slow_last {
                self.signals.push(Signal::new(
                    self.cfg.symbol.clone(),
                    tesser_core::SignalKind::ExitLong,
                    0.75,
                ));
            }
        }
        Ok(())
    }

    fn sma(candles: &VecDeque<Candle>, period: usize) -> StrategyResult<Vec<f64>> {
        if period == 0 {
            return Err(StrategyError::InvalidConfig(
                "period must be greater than zero".into(),
            ));
        }
        let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
        let mut values = Vec::with_capacity(closes.len() - period + 1);
        for window in closes.windows(period) {
            values.push(window.iter().sum::<f64>() / period as f64);
        }
        Ok(values)
    }
}

impl Strategy for SmaCross {
    fn name(&self) -> &str {
        "sma-cross"
    }

    fn symbol(&self) -> &str {
        &self.cfg.symbol
    }

    fn configure(&mut self, params: toml::Value) -> StrategyResult<()> {
        let cfg = SmaCrossConfig::try_from(params)?;
        if cfg.fast_period == 0 || cfg.slow_period == 0 {
            return Err(StrategyError::InvalidConfig(
                "period values must be greater than zero".into(),
            ));
        }
        self.cfg = cfg;
        Ok(())
    }

    fn on_tick(&mut self, _ctx: &StrategyContext, _tick: &Tick) -> StrategyResult<()> {
        Ok(())
    }

    fn on_candle(&mut self, ctx: &StrategyContext, candle: &Candle) -> StrategyResult<()> {
        if candle.symbol != self.cfg.symbol {
            return Ok(());
        }
        self.maybe_emit_signal(ctx.candles())
    }

    fn on_fill(&mut self, _ctx: &StrategyContext, _fill: &Fill) -> StrategyResult<()> {
        Ok(())
    }

    fn drain_signals(&mut self) -> Vec<Signal> {
        std::mem::take(&mut self.signals)
    }
}
