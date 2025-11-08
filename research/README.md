# Research Environment

This directory hosts all Python-based research workflows:

- `notebooks/`: Jupyter or VS Code notebooks for exploratory data analysis.
- `scripts/`: Reusable Python scripts (feature generation, parameter sweeps, ML training).
- `strategies/`: Serialized outputs such as TOML parameter files or ONNX models that Rust consumers load.

## Quick Start

```bash
cd research
uv venv
source .venv/bin/activate
uv pip install -e .
```

Once the environment is ready, you can open notebooks or run scripts:

```bash
uv run python scripts/find_optimal_sma.py --data ../data/btc.parquet
```

Store generated strategy configs under `strategies/` (e.g., `strategies/sma_cross_optimal.toml`) so that the Rust CLI can consume them via `--strategy-config`.
