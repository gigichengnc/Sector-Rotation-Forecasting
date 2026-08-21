# Validation Contract — 2026 Rebuild

This document defines the minimum evaluation rules that every predictive model in the 2026 reconstruction must follow.

## Unit of time

The Phase 1 research dataset is weekly. A horizon of `h=4` means four weekly observations ahead, not four calendar days.

For a decision row at time `t`:

- features may use observations at `t` and earlier only;
- the target is the RRG-style coordinate at `t + h`;
- the predicted quadrant is derived from the predicted coordinates.

## Purged walk-forward folds

Validation is chronological and non-overlapping. For each fold:

```text
training rows | purge gap | validation rows
```

The purge gap must satisfy:

```text
gap >= prediction horizon
```

This prevents the future target attached to the last training row from extending into the validation period.

The implementation supports expanding and fixed-size training windows, but validation windows are always disjoint. A timestamp can therefore be scored at most once per model/horizon evaluation run.

## Fold-local preprocessing

Any learned preprocessing step must be fitted on training rows only.

The first trainable baseline uses:

```text
StandardScaler -> LinearRegression
```

The scaler is fitted inside each fold. Validation rows do not contribute to its mean or standard deviation.

## Baselines

### Persistence

Predict:

```text
future RS-Ratio    = current RS-Ratio
future RS-Momentum = current RS-Momentum
```

This is the minimum benchmark a more complex model should beat.

### Linear coordinate regression

Use the most recent `lookback` RRG coordinate pairs as lagged features and jointly predict:

- future RS-Ratio;
- future RS-Momentum.

This deliberately fixes a mismatch in the surviving 2025 ML path, where the trainable linear model learned only the first target element while downstream code expected both coordinates.

## Metrics

Every model is evaluated on the same out-of-sample rows using:

- RS-Ratio MAE;
- RS-Momentum MAE;
- RS-Ratio RMSE;
- RS-Momentum RMSE;
- mean Euclidean coordinate distance;
- quadrant accuracy derived from predicted coordinates;
- sample count `n`.

Quadrant accuracy is not treated as investment profitability.

## Current verification status

The validation/modeling layer has been exercised with deterministic synthetic data and unit tests. The synthetic test is a software sanity check only; it is **not market evidence** and is not reported as a model-performance claim.

Before any historical accuracy is quoted, the next gate is to build the deterministic weekly ETF/SPY dataset and run the same evaluator on real point-in-time observations.
