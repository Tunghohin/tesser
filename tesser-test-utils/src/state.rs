use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use anyhow::{anyhow, Result};
use tokio::sync::{mpsc, Mutex};

use tesser_core::{AccountBalance, Candle, Fill, Order, OrderId, Position, Symbol, Tick};

use crate::scenario::ScenarioManager;

pub type ApiKey = String;

/// Message pushed onto the private WebSocket stream.
pub type PrivateMessage = serde_json::Value;

/// Shared state for the in-memory mock exchange.
#[derive(Clone)]
pub struct MockExchangeState {
    inner: Arc<Mutex<Inner>>,
    scenarios: ScenarioManager,
}

#[allow(dead_code)]
pub(crate) struct Inner {
    pub accounts: HashMap<ApiKey, AccountState>,
    pub orders: HashMap<OrderId, Order>,
    pub market_data: MarketDataQueues,
    pub private_ws_sender: Option<mpsc::UnboundedSender<PrivateMessage>>,
}

#[derive(Clone)]
pub struct AccountState {
    pub api_secret: String,
    pub balances: HashMap<String, AccountBalance>,
    pub positions: HashMap<Symbol, Position>,
    pub executions: VecDeque<Fill>,
}

impl AccountState {
    fn from_config(config: AccountConfig) -> Self {
        Self {
            api_secret: config.api_secret,
            balances: config
                .balances
                .into_iter()
                .map(|balance| (balance.currency.clone(), balance))
                .collect(),
            positions: config
                .positions
                .into_iter()
                .map(|position| (position.symbol.clone(), position))
                .collect(),
            executions: VecDeque::new(),
        }
    }
}

#[derive(Default)]
pub struct MarketDataQueues {
    pub candles: VecDeque<Candle>,
    pub ticks: VecDeque<Tick>,
}

impl MarketDataQueues {
    pub fn push_candle(&mut self, candle: Candle) {
        self.candles.push_back(candle);
    }

    pub fn push_tick(&mut self, tick: Tick) {
        self.ticks.push_back(tick);
    }

    pub fn next_candle(&mut self) -> Option<Candle> {
        self.candles.pop_front()
    }

    pub fn next_tick(&mut self) -> Option<Tick> {
        self.ticks.pop_front()
    }
}

/// Declarative account bootstrap configuration.
#[derive(Clone)]
pub struct AccountConfig {
    pub api_key: String,
    pub api_secret: String,
    pub balances: Vec<AccountBalance>,
    pub positions: Vec<Position>,
}

impl AccountConfig {
    pub fn new(api_key: impl Into<String>, api_secret: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            api_secret: api_secret.into(),
            balances: Vec::new(),
            positions: Vec::new(),
        }
    }

    pub fn with_balance(mut self, balance: AccountBalance) -> Self {
        self.balances.push(balance);
        self
    }

    pub fn with_position(mut self, position: Position) -> Self {
        self.positions.push(position);
        self
    }
}

/// Configuration object passed into [`MockExchangeState::new`].
#[derive(Clone)]
pub struct MockExchangeConfig {
    pub accounts: Vec<AccountConfig>,
    pub candles: Vec<Candle>,
    pub ticks: Vec<Tick>,
    pub scenarios: ScenarioManager,
}

impl MockExchangeConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_account(mut self, account: AccountConfig) -> Self {
        self.accounts.push(account);
        self
    }

    pub fn with_candles(mut self, candles: impl IntoIterator<Item = Candle>) -> Self {
        self.candles.extend(candles);
        self
    }

    pub fn with_ticks(mut self, ticks: impl IntoIterator<Item = Tick>) -> Self {
        self.ticks.extend(ticks);
        self
    }

    pub fn with_scenarios(mut self, scenarios: ScenarioManager) -> Self {
        self.scenarios = scenarios;
        self
    }
}

impl Default for MockExchangeConfig {
    fn default() -> Self {
        Self {
            accounts: Vec::new(),
            candles: Vec::new(),
            ticks: Vec::new(),
            scenarios: ScenarioManager::new(),
        }
    }
}

impl MockExchangeState {
    pub fn new(config: MockExchangeConfig) -> Self {
        let market_data = MarketDataQueues {
            candles: config.candles.into_iter().collect(),
            ticks: config.ticks.into_iter().collect(),
        };
        let accounts = config
            .accounts
            .into_iter()
            .map(|account| {
                let api_key = account.api_key.clone();
                (api_key, AccountState::from_config(account))
            })
            .collect();
        let inner = Inner {
            accounts,
            orders: HashMap::new(),
            market_data,
            private_ws_sender: None,
        };
        Self {
            inner: Arc::new(Mutex::new(inner)),
            scenarios: config.scenarios,
        }
    }

    pub fn scenarios(&self) -> ScenarioManager {
        self.scenarios.clone()
    }

    #[allow(dead_code)]
    pub(crate) fn inner(&self) -> &Arc<Mutex<Inner>> {
        &self.inner
    }

    pub async fn set_private_ws_sender(&self, sender: mpsc::UnboundedSender<PrivateMessage>) {
        let mut guard = self.inner.lock().await;
        guard.private_ws_sender = Some(sender);
    }

    pub async fn clear_private_ws_sender(&self) {
        let mut guard = self.inner.lock().await;
        guard.private_ws_sender = None;
    }

    pub async fn emit_private_message(&self, payload: PrivateMessage) -> Result<()> {
        let sender = {
            let guard = self.inner.lock().await;
            guard.private_ws_sender.clone()
        };
        if let Some(tx) = sender {
            tx.send(payload)
                .map_err(|err| anyhow!("failed to deliver private stream message: {err}"))
        } else {
            Ok(())
        }
    }
}
