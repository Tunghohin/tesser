"""Train a simple logistic regression classifier on candle data."""

import argparse
from pathlib import Path

import numpy as np
import pandas as pd
import toml
from sklearn.linear_model import LogisticRegression


def load_close(path: Path) -> pd.Series:
    df = pd.read_csv(path)
    if "timestamp" not in df or "close" not in df:
        raise ValueError("CSV must contain timestamp and close columns")
    df["timestamp"] = pd.to_datetime(df["timestamp"])
    df = df.sort_values("timestamp")
    return df["close"].astype(float)


def build_dataset(close: pd.Series, lookback: int) -> tuple[np.ndarray, np.ndarray]:
    returns = close.pct_change().fillna(0.0)
    features = []
    labels = []
    for idx in range(lookback, len(returns) - 1):
        window = returns.iloc[idx - lookback : idx].to_numpy()
        if window.size != lookback:
            continue
        features.append(window)
        next_ret = returns.iloc[idx + 1]
        labels.append(1 if next_ret > 0 else 0)
    return np.array(features), np.array(labels)


def main() -> None:
    parser = argparse.ArgumentParser(description="Train ML classifier for Tesser")
    parser.add_argument("--data", type=Path, required=True)
    parser.add_argument("--symbol", default="BTCUSDT")
    parser.add_argument("--lookback", type=int, default=20)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    close = load_close(args.data)
    X, y = build_dataset(close, args.lookback)
    if len(X) == 0:
        raise RuntimeError("Not enough data to build features")

    model = LogisticRegression(max_iter=1000)
    model.fit(X, y)
    weights = model.coef_[0].tolist()
    bias = float(model.intercept_[0])

    artifact = {
        "bias": bias,
        "weights": weights,
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(toml.dumps(artifact))
    config_path = args.output.with_suffix(".strategy.toml")
    config = {
        "strategy_name": "MlClassifier",
        "params": {
            "symbol": args.symbol,
            "model_path": str(args.output),
            "lookback": args.lookback,
            "threshold_long": 0.1,
            "threshold_short": -0.1,
        },
    }
    config_path.write_text(toml.dumps(config))
    print(f"Model saved to {args.output}, strategy config to {config_path}")


if __name__ == "__main__":
    main()
