use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use clap::{Args, Parser, Subcommand};
use serde::Deserialize;
use tesser_backtester::{BacktestConfig, BacktestReport, Backtester};
use tesser_config::{load_config, AppConfig};
use tesser_core::{Candle, Interval, Symbol};
use tesser_execution::{ExecutionEngine, FixedOrderSizer};
use tesser_paper::PaperExecutionClient;
use tesser_strategy::{build_builtin_strategy, builtin_strategy_names};
use tracing::info;

#[derive(Parser)]
#[command(author, version, about = "Tesser CLI")]
struct Cli {
    /// Selects which configuration environment to load (maps to config/{env}.toml)
    #[arg(long, default_value = "default")]
    env: String,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Data engineering tasks
    Data {
        #[command(subcommand)]
        action: DataCommand,
    },
    /// Backtesting workflows
    Backtest {
        #[command(subcommand)]
        action: BacktestCommand,
    },
    /// Live trading workflows
    Live {
        #[command(subcommand)]
        action: LiveCommand,
    },
    /// Strategy management helpers
    Strategies,
}

#[derive(Subcommand)]
enum DataCommand {
    /// Download historical market data (placeholder)
    Download(DataDownloadArgs),
    /// Validate a local data set (placeholder)
    Validate(DataValidateArgs),
    /// Resample existing data (placeholder)
    Resample(DataResampleArgs),
}

#[derive(Subcommand)]
enum BacktestCommand {
    /// Run a backtest from a strategy config file
    Run(BacktestRunArgs),
}

#[derive(Subcommand)]
enum LiveCommand {
    /// Start a live trading session (scaffolding)
    Run(LiveRunArgs),
}

#[derive(Args)]
struct DataDownloadArgs {
    #[arg(long, default_value = "bybit")]
    exchange: String,
    #[arg(long)]
    symbol: String,
    #[arg(long)]
    start: String,
    #[arg(long)]
    end: Option<String>,
}

#[derive(Args)]
struct DataValidateArgs {
    #[arg(long)]
    path: PathBuf,
}

#[derive(Args)]
struct DataResampleArgs {
    #[arg(long)]
    input: PathBuf,
    #[arg(long)]
    output: PathBuf,
    #[arg(long, default_value = "1h")]
    interval: String,
}

#[derive(Args)]
struct BacktestRunArgs {
    #[arg(long)]
    strategy_config: PathBuf,
    #[arg(long, default_value_t = 500)]
    candles: usize,
    #[arg(long, default_value_t = 0.01)]
    quantity: f64,
}

#[derive(Args)]
struct LiveRunArgs {
    #[arg(long)]
    strategy_config: PathBuf,
    #[arg(long, default_value = "bybit_testnet")]
    exchange: String,
}

#[derive(Deserialize)]
struct StrategyConfigFile {
    #[serde(rename = "strategy_name")]
    name: String,
    #[serde(default = "empty_table")]
    params: toml::Value,
}

fn empty_table() -> toml::Value {
    toml::Value::Table(Default::default())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,tesser_cli=info".into()),
        )
        .init();

    let cli = Cli::parse();
    let config = load_config(Some(&cli.env)).context("failed to load configuration")?;

    match cli.command {
        Commands::Data { action } => handle_data(action, &config).await?,
        Commands::Backtest {
            action: BacktestCommand::Run(args),
        } => args.run(&config).await?,
        Commands::Live {
            action: LiveCommand::Run(args),
        } => args.run(&config).await?,
        Commands::Strategies => list_strategies(),
    }

    Ok(())
}

async fn handle_data(cmd: DataCommand, config: &AppConfig) -> Result<()> {
    match cmd {
        DataCommand::Download(args) => {
            info!(
                "stub: downloading data for {} on {} into {} ({} -> {:?})",
                args.symbol,
                args.exchange,
                config.data_path.display(),
                args.start,
                args.end
            );
        }
        DataCommand::Validate(args) => {
            info!("stub: validating dataset at {}", args.path.display());
        }
        DataCommand::Resample(args) => {
            info!(
                "stub: resampling {} into {} at {}",
                args.input.display(),
                args.output.display(),
                args.interval
            );
        }
    }
    Ok(())
}

impl BacktestRunArgs {
    async fn run(&self, _config: &AppConfig) -> Result<()> {
        let contents = std::fs::read_to_string(&self.strategy_config)
            .with_context(|| format!("failed to read {}", self.strategy_config.display()))?;
        let def: StrategyConfigFile =
            toml::from_str(&contents).context("failed to parse strategy config file")?;
        let strategy = build_builtin_strategy(&def.name, def.params)
            .with_context(|| format!("failed to configure strategy {}", def.name))?;
        let symbols = strategy.subscriptions();
        if symbols.is_empty() {
            return Err(anyhow::anyhow!("strategy did not declare subscriptions"));
        }

        let mut candles = Vec::new();
        for (idx, symbol) in symbols.iter().enumerate() {
            let offset = idx as i64 * 10;
            candles.extend(synth_candles(symbol, self.candles, offset));
        }
        candles.sort_by_key(|c| c.timestamp);

        let execution = ExecutionEngine::new(
            PaperExecutionClient::default(),
            Box::new(FixedOrderSizer {
                quantity: self.quantity,
            }),
        );

        let cfg = BacktestConfig::new(symbols[0].clone(), candles);
        let report = Backtester::new(cfg, strategy, execution)
            .run()
            .await
            .context("backtest failed")?;
        print_report(report);
        Ok(())
    }
}

impl LiveRunArgs {
    async fn run(&self, config: &AppConfig) -> Result<()> {
        let exchange_cfg = config
            .exchange
            .get(&self.exchange)
            .ok_or_else(|| anyhow::anyhow!("exchange profile {} not found", self.exchange))?;
        info!(
            "stub: launching live session on {} (REST {}, WS {})",
            self.exchange, exchange_cfg.rest_url, exchange_cfg.ws_url
        );
        info!(
            "strategy config located at {} (not yet wired to execution)",
            self.strategy_config.display()
        );
        Ok(())
    }
}

fn list_strategies() {
    println!("Built-in strategies:");
    for name in builtin_strategy_names() {
        println!("- {name}");
    }
}

fn print_report(report: BacktestReport) {
    println!("Backtest completed:");
    println!("  Signals generated: {}", report.signals_emitted);
    println!("  Orders sent: {}", report.orders_sent);
    println!("  Ending equity: {:.2}", report.ending_equity);
}

fn synth_candles(symbol: &str, len: usize, offset_minutes: i64) -> Vec<Candle> {
    let mut candles = Vec::with_capacity(len);
    for i in 0..len {
        let base = 50_000.0 + ((i as f64) + offset_minutes as f64).sin() * 500.0;
        let open = base + (i as f64 % 3.0) * 10.0;
        let close = open + (i as f64 % 5.0) * 5.0 - 10.0;
        candles.push(Candle {
            symbol: Symbol::from(symbol),
            interval: Interval::OneMinute,
            open,
            high: open.max(close) + 20.0,
            low: open.min(close) - 20.0,
            close,
            volume: 1.0,
            timestamp: Utc::now() - Duration::minutes((len - i) as i64)
                + Duration::minutes(offset_minutes),
        });
    }
    candles
}
