"""Grid search for RSI thresholds and period using historical candles."""

import argparse
from pathlib import Path
from typing import Iterable, Tuple

import numpy as np
import pandas as pd
import toml


def load_csv(path: Path) -> pd.Series:
    df = pd.read_csv(path)
    if "timestamp" not in df or "close" not in df:
        raise ValueError("CSV must contain timestamp and close columns")
    df["timestamp"] = pd.to_datetime(df["timestamp"])
    df = df.sort_values("timestamp")
    return df["close"].astype(float)


def rsi(series: pd.Series, period: int) -> pd.Series:
    diff = series.diff()
    up = diff.clip(lower=0)
    down = -diff.clip(upper=0)
    avg_gain = up.rolling(period).mean()
    avg_loss = down.rolling(period).mean()
    rs = avg_gain / avg_loss
    rsi_series = 100 - (100 / (1 + rs))
    return rsi_series


def evaluate(close: pd.Series, period: int, oversold: float, overbought: float) -> float:
    signal = pd.Series(0, index=close.index)
    rsi_series = rsi(close, period)
    signal[rsi_series <= oversold] = 1
    signal[rsi_series >= overbought] = -1
    returns = close.pct_change().fillna(0.0)
    pnl = (signal.shift(1).fillna(0) * returns).cumsum()
    return pnl.iloc[-1]


def grid(periods: Iterable[int], oversold_vals: Iterable[float], overbought_vals: Iterable[float]):
    for period in periods:
        for oversold in oversold_vals:
            for overbought in overbought_vals:
                if oversold >= overbought:
                    continue
                yield period, oversold, overbought


def main():
    parser = argparse.ArgumentParser(description="Optimize RSI strategy parameters")
    parser.add_argument("--data", type=Path, required=True)
    parser.add_argument("--symbol", default="BTCUSDT")
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--periods", default="8,14,21,28", help="Comma-separated RSI periods")
    parser.add_argument(
        "--oversold",
        default="20,25,30,35",
        help="Comma-separated oversold thresholds",
    )
    parser.add_argument(
        "--overbought",
        default="65,70,75,80",
        help="Comma-separated overbought thresholds",
    )
    args = parser.parse_args()

    close = load_csv(args.data)
    periods = [int(x) for x in args.periods.split(",")]
    oversold_vals = [float(x) for x in args.oversold.split(",")]
    overbought_vals = [float(x) for x in args.overbought.split(",")]

    best_score = float("-inf")
    best_params: Tuple[int, float, float] | None = None
    for period, oversold, overbought in grid(periods, oversold_vals, overbought_vals):
        score = evaluate(close, period, oversold, overbought)
        if score > best_score:
            best_score = score
            best_params = (period, oversold, overbought)

    if best_params is None:
        raise RuntimeError("Grid search produced no valid parameter set")

    period, oversold, overbought = best_params
    config = {
        "strategy_name": "RsiReversion",
        "params": {
            "symbol": args.symbol,
            "period": period,
            "oversold": oversold,
            "overbought": overbought,
            "lookback": 400,
        },
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(toml.dumps(config))
    print(f"Best: period={period}, oversold={oversold}, overbought={overbought}, score={best_score:.4f}")
    print(f"Saved config to {args.output}")


if __name__ == "__main__":
    main()
