"""Example research script for optimizing SMA parameters."""

import argparse
from pathlib import Path

import numpy as np
import pandas as pd
import toml


def load_data(path: Path) -> pd.DataFrame:
    if path.suffix == ".parquet":
        return pd.read_parquet(path)
    if path.suffix == ".csv":
        return pd.read_csv(path)
    raise ValueError(f"Unsupported extension: {path.suffix}")


def evaluate(close: pd.Series, fast: int, slow: int) -> float:
    fast_ma = close.rolling(fast).mean()
    slow_ma = close.rolling(slow).mean()
    signal = np.where(fast_ma > slow_ma, 1, -1)
    returns = close.pct_change().fillna(0.0)
    pnl = (signal.shift(1).fillna(0) * returns).cumsum()
    return pnl.iloc[-1]


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--data", type=Path, required=True, help="Path to OHLCV data")
    parser.add_argument("--symbol", default="BTCUSDT")
    parser.add_argument("--output", type=Path, default=Path("strategies/sma_cross_optimal.toml"))
    args = parser.parse_args()

    df = load_data(args.data)
    best_score = float("-inf")
    best_pair = (5, 20)
    close = df["close"]

    for fast in range(5, 25, 5):
        for slow in range(fast + 5, 60, 5):
            score = evaluate(close, fast, slow)
            if score > best_score:
                best_score = score
                best_pair = (fast, slow)

    config = {
        "strategy_name": "SmaCross",
        "params": {
            "symbol": args.symbol,
            "fast_period": best_pair[0],
            "slow_period": best_pair[1],
            "min_samples": best_pair[1] + 5,
        },
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(toml.dumps(config))
    print(f"Saved optimal params to {args.output}")


if __name__ == "__main__":
    main()
