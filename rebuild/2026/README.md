# 2026 rebuild

This directory contains the research-first reconstruction of the 2025 RRG prototype.

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

- `data.py` — Yahoo chart parsing, adjusted-close requirement, provenance, completed-week resampling.
- `dataset.py` — common-history alignment and sector panel construction.
- `rrg.py` — transparent causal RRG-style coordinate calculation.
- `targets.py` — future coordinate targets and persistence baseline.
- `validation.py` — chronological folds.
- `modeling.py` — lagged features, LinearRegression pipeline, metrics.
- `holdout.py` / `run_final_holdout.py` — frozen one-time final evaluation guards.
- `deployment.py` / `forecast_from_archive.py` — prospective point forecasts.
- `strategy.py` / `backtest_strategy.py` — separately declared economic-value experiment.

The first strategy hypothesis was not supported. That negative result is part of the research conclusion, not something to tune away after inspection.
