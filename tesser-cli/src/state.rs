use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::fs;

use tesser_core::Order;
use tesser_portfolio::PortfolioState;

/// Durable snapshot of the live trading runtime.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct LiveState {
    pub portfolio: Option<PortfolioState>,
    pub open_orders: Vec<Order>,
    pub last_prices: HashMap<String, f64>,
    pub last_candle_ts: Option<DateTime<Utc>>,
}

/// Helper responsible for loading and saving `LiveState` documents.
pub struct LiveStateStore {
    path: PathBuf,
}

impl LiveStateStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub async fn load(&self) -> Result<LiveState> {
        if !self.path.exists() {
            return Ok(LiveState::default());
        }
        let bytes = fs::read(&self.path)
            .await
            .with_context(|| format!("failed to read live state from {}", self.path.display()))?;
        let state = serde_json::from_slice(&bytes)
            .with_context(|| format!("failed to parse live state at {}", self.path.display()))?;
        Ok(state)
    }

    pub async fn save(&self, state: &LiveState) -> Result<()> {
        if let Some(dir) = self.path.parent() {
            fs::create_dir_all(dir)
                .await
                .with_context(|| format!("failed to create state directory {dir:?}"))?;
        }
        let bytes = serde_json::to_vec_pretty(state)?;
        fs::write(&self.path, bytes)
            .await
            .with_context(|| format!("failed to persist live state to {}", self.path.display()))
    }
}
