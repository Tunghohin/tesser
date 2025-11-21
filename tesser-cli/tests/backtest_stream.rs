use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Result;
use assert_cmd::prelude::*;
use chrono::{Duration, Utc};
use parquet::arrow::ArrowWriter;
use parquet::file::properties::WriterProperties;
use rust_decimal::Decimal;
use tempfile::tempdir;

use tesser_core::{Candle, Interval};
use tesser_data::encoding::candles_to_batch;

const STRATEGY_CONFIG: &str = r#"
strategy_name = "SmaCross"

[params]
symbol = "BTCUSDT"
fast_period = 3
slow_period = 5
min_samples = 5
"#;

#[test]
fn backtest_runs_with_csv_and_parquet_inputs() -> Result<()> {
    let temp = tempdir()?;
    let strategy_path = temp.path().join("strategy.toml");
    fs::write(&strategy_path, STRATEGY_CONFIG)?;

    let candles = sample_candles();
    let csv_path = temp.path().join("bars.csv");
    write_csv(&csv_path, &candles)?;

    let parquet_path = temp.path().join("bars.parquet");
    write_parquet(&parquet_path, &candles)?;

    run_backtest(&strategy_path, &csv_path)?;
    run_backtest(&strategy_path, &parquet_path)?;
    Ok(())
}

fn run_backtest(strategy: &Path, data: &Path) -> Result<()> {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let markets_file = workspace_root.join("config/markets.toml");
    let binary = assert_cmd::cargo::cargo_bin!("tesser-cli");
    let mut cmd = Command::new(binary);
    cmd.current_dir(&workspace_root);
    cmd.args([
        "--env",
        "default",
        "backtest",
        "run",
        "--strategy-config",
        strategy.to_str().unwrap(),
        "--data",
        data.to_str().unwrap(),
        "--markets-file",
        markets_file.to_str().unwrap(),
        "--quantity",
        "0.01",
        "--candles",
        "64",
    ]);
    cmd.assert().success();
    Ok(())
}

fn write_csv(path: &Path, candles: &[Candle]) -> Result<()> {
    let mut file = File::create(path)?;
    writeln!(file, "symbol,timestamp,open,high,low,close,volume")?;
    for candle in candles {
        writeln!(
            file,
            "{},{},{},{},{},{},{}",
            candle.symbol,
            candle.timestamp.to_rfc3339(),
            candle.open,
            candle.high,
            candle.low,
            candle.close,
            candle.volume
        )?;
    }
    Ok(())
}

fn write_parquet(path: &Path, candles: &[Candle]) -> Result<()> {
    let batch = candles_to_batch(candles)?;
    let file = File::create(path)?;
    let props = WriterProperties::builder().build();
    let mut writer = ArrowWriter::try_new(file, batch.schema(), Some(props))?;
    writer.write(&batch)?;
    writer.close()?;
    Ok(())
}

fn sample_candles() -> Vec<Candle> {
    let base = Utc::now() - Duration::minutes(10);
    (0..8)
        .map(|idx| Candle {
            symbol: "BTCUSDT".into(),
            interval: Interval::OneMinute,
            open: Decimal::new(20_000 + idx as i64, 0),
            high: Decimal::new(20_010 + idx as i64, 0),
            low: Decimal::new(19_990 + idx as i64, 0),
            close: Decimal::new(20_005 + idx as i64, 0),
            volume: Decimal::new(1, 0),
            timestamp: base + Duration::minutes(idx as i64),
        })
        .collect()
}
