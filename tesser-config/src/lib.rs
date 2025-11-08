//! Layered configuration loading utilities.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::Result;
use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

/// Root application configuration deserialized from layered sources.
#[derive(Debug, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_data_path")]
    pub data_path: PathBuf,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default)]
    pub backtest: BacktestConfig,
    #[serde(default)]
    pub exchange: HashMap<String, ExchangeConfig>,
}

#[derive(Debug, Deserialize)]
pub struct BacktestConfig {
    #[serde(default = "default_equity")]
    pub initial_equity: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ExchangeConfig {
    pub rest_url: String,
    pub ws_url: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub api_secret: String,
}

impl Default for BacktestConfig {
    fn default() -> Self {
        Self {
            initial_equity: default_equity(),
        }
    }
}

fn default_data_path() -> PathBuf {
    PathBuf::from("./data")
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_equity() -> f64 {
    10_000.0
}

/// Loads configuration by merging files and environment variables.
///
/// Sources (lowest to highest precedence):
/// 1. `config/default.toml`
/// 2. `config/{environment}.toml` (if `environment` is Some)
/// 3. `config/local.toml` (optional, ignored in git)
/// 4. Environment variables prefixed with `TESSER_`
pub fn load_config(env: Option<&str>) -> Result<AppConfig> {
    let base_path = Path::new("config");
    let mut builder =
        Config::builder().add_source(File::from(base_path.join("default.toml")).required(true));

    if let Some(env_name) = env {
        builder = builder
            .add_source(File::from(base_path.join(format!("{env_name}.toml"))).required(false));
    }

    builder = builder.add_source(File::from(base_path.join("local.toml")).required(false));

    builder = builder.add_source(
        Environment::with_prefix("TESSER")
            .separator("__")
            .ignore_empty(true),
    );

    let config = builder.build()?;
    config
        .try_deserialize()
        .map_err(|err: ConfigError| err.into())
}
