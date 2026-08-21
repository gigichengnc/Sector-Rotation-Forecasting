# Final Holdout Protocol — Frozen before opening

Protocol ID: `rrg-final-holdout-v1-2026-08-20`

## Status

This document freezes the final evaluation procedure **before any final-holdout
performance is computed**.

The purpose is to make the last 52 weekly target observations a genuine
one-time test rather than another model-development surface.

## Frozen data

The final run must use exactly the fresh 2026 market-data archive already used
for the development benchmark.

- ZIP SHA-256:
  `3838be08d18b238675ea02b9addce983799820abdf74eb40ac0e4ce8481b82bf`
- `manifest.csv` SHA-256:
  `4ec4b56f8f14f1c2b21bd60d9e440fd90fa108f83ed1e2c0959cc23ecd6b0c35`
- benchmark: SPY
- sectors: XLB, XLC, XLE, XLF, XLI, XLK, XLP, XLRE, XLU, XLV, XLY
- price field: adjusted close
- incomplete acquisition week remains excluded using the original fetch-time
  `as_of_utc` cutoff.

No fresh download or data replacement is permitted for the one-time result.

## Frozen RRG/model specification

- RRG-style relative-ratio period: 10 weekly observations
- momentum period: 10
- causal normalization period: 100
- normalization scale: 10
- model lookback: 20 weekly coordinate observations
- horizons: 1, 2, 4, 8 weekly observations
- selected model: `StandardScaler -> LinearRegression`
- persistence is reported as the required baseline
- no nonlinear model, LSTM, grid search, or post-holdout tuning is allowed.

## Holdout definition

The final holdout is defined by **target timestamp** so all four horizons are
judged on the same target weeks:

- first target week: 2025-08-22
- last target week: 2026-08-14
- 52 completed Friday target observations per sector/horizon.

For each symbol and horizon, the first holdout target has an earlier decision
timestamp. The final linear model for that symbol/horizon is fitted once using
only training examples whose **target timestamp is strictly earlier than that
first holdout decision timestamp**.

This is intentionally conservative. It prevents labels that were not yet known
at the first holdout decision from entering the fitted model.

The fitted model is **not refit during the holdout**. Earlier holdout labels
therefore cannot improve later holdout predictions.

## One-time output

The guarded runner reports both persistence and linear predictions using:

- coordinate MAE/RMSE;
- mean 2D coordinate distance;
- derived quadrant accuracy;
- macro F1;
- sample count;
- sector/horizon breakdown;
- aggregate summary.

The run also writes a `RUN-MARKER.json` recording protocol ID, dataset hashes,
time of execution, frozen model, and `holdout_reuse_allowed: false`.

## Guard conditions

The runner must refuse to open market-data contents unless both are supplied:

1. `--open-final-holdout`
2. exact token `OPEN_FINAL_HOLDOUT_2026_08_20`

It must also refuse:

- an input ZIP with a different SHA-256;
- a manifest with a different SHA-256;
- an existing output directory.

These controls are not cryptographic access control. Their purpose is to make an
accidental or casual holdout run difficult and visible.

## Rerun rule

The final holdout is intended to be opened once.

If the completed run is disappointing, the result remains the result. The model
must not be tuned and rerun against the same holdout.

A rerun is allowed only if a genuine implementation defect invalidates the run.
In that case:

1. the defect must be documented before reviewing a replacement result;
2. the invalidated run must remain recorded;
3. the correction must be tested and committed;
4. the replacement must be explicitly labelled a corrected rerun, not an
   independent fresh holdout.

## Pre-open gates

The holdout may be opened only after:

1. this protocol and the guarded runner are committed;
2. holdout guard/split tests pass;
3. repository CI is confirmed green on the frozen code commit;
4. explicit user authorization is given to **OPEN FINAL HOLDOUT**.

Until all four conditions are met, no final-holdout metrics should be computed.
