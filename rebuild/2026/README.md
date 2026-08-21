# 2026 rebuild

This directory contains the research-first reconstruction of the sector-rotation forecasting prototype first initiated in late 2025.

The installable package now lives at the **repository root**. From the root checkout, install with:

```bash
python -m pip install -r rebuild/2026/requirements-lock.txt
python -m pip install -e . --no-deps --no-build-isolation
```

The user-facing command is:

```bash
sector-rotation forecast
```

That CLI exposes the prospective deployment path. The final-holdout, model-selection, and strategy scripts remain research/reproduction tools rather than ordinary end-user commands.

## Frozen predictive contract

- weekly completed observations;
- SPY benchmark;
- 11 Select Sector SPDR ETFs;
- adjusted close;
- 20-observation coordinate lookback;
- 1/2/4/8-week horizons;
- future RRG-style coordinate pair as primary target;
- persistence baseline;
- `StandardScaler -> LinearRegression` selected after development model comparison;
- fold-local preprocessing and purged chronological validation;
- one-time 52-week final holdout.

## Layers

- `src/rrg_rebuild/data.py` — Yahoo chart parsing, adjusted-close requirement, provenance, completed-week resampling.
- `src/rrg_rebuild/dataset.py` — common-history alignment and sector panel construction.
- `src/rrg_rebuild/rrg.py` — transparent causal RRG-style coordinate calculation.
- `src/rrg_rebuild/targets.py` — future coordinate targets and persistence baseline.
- `src/rrg_rebuild/validation.py` — chronological folds.
- `src/rrg_rebuild/modeling.py` — lagged features, LinearRegression pipeline, metrics.
- `src/rrg_rebuild/deployment.py` — prospective model fitting/forecast logic.
- `src/rrg_rebuild/workflow.py` — fresh-data acquisition, provenance/archive creation, and one-command prospective workflow.
- `src/rrg_rebuild/cli.py` — `sector-rotation` package CLI.
- `src/rrg_rebuild/holdout.py` / `scripts/run_final_holdout.py` — frozen one-time final evaluation guards.
- `scripts/forecast_from_archive.py` — archived-data prospective reproduction path.
- `src/rrg_rebuild/strategy.py` / `scripts/backtest_strategy.py` — separately declared economic-value experiment.

The first strategy hypothesis was not supported. That negative result is part of the research conclusion, not something to tune away after inspection.
