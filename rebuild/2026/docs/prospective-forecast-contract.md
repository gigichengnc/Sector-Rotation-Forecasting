# Prospective Forecast Contract — 2026-08-20

## Purpose

This layer turns the validated reconstruction into a prospective research tool.
It is deliberately separate from the development and final-holdout evaluators.

## Frozen deployment model

- model: `StandardScaler -> LinearRegression`;
- target: future RRG-style RS-Ratio and RS-Momentum coordinates;
- lookback: 20 completed weekly observations;
- horizons: 1, 2, 4, 8 weekly observations;
- universe: 11 sector ETFs relative to SPY;
- one model is fitted per sector/horizon;
- training uses every target already observable at the latest completed common week;
- latest forecast features use only the latest/current and previous 19 RRG observations.

After the final holdout was opened once and frozen, those historical labels may be
used for **future deployment fitting** because the deployment decision occurs later.
They must not be used to re-select a model or revise the reported holdout result.

## Output

Each run emits:

- current RRG-style coordinates and quadrant;
- predicted coordinates and quadrant at 1/2/4/8 weeks;
- decision and target timestamps;
- number of labelled training rows used;
- input ZIP + manifest hashes;
- verification that all 12 raw Yahoo payloads match manifest hashes.

No confidence/probability is emitted because the current model is not calibrated.
No forecast is described as a buy/sell recommendation or expected trading return.

## Reproducibility

A prospective run refuses to overwrite an existing output directory. Each weekly
run should therefore be stored in a new timestamped/versioned folder.
