# Candidate Model Freeze — 2026-08-20 (amended before reading candidate results)

## Reason for amendment

The originally proposed 300-tree Random Forest was removed before candidate performance was summarized.
A timing check showed the fixed design required roughly 34 seconds for one sector across the four
development horizons in the current evaluation environment, making the full 11-sector run impractical here.
The candidate was dropped for computational/reproducibility reasons, not because of observed predictive results.

No Random Forest performance result was reviewed.

## Development/holdout boundary

- Use the same completed weekly dataset and RRG-style configuration as the first benchmark.
- Keep the final 52 completed RRG observations (2025-08-22 through 2026-08-14) untouched.
- Candidate models are evaluated only on the existing development walk-forward folds.
- No hyperparameter search is permitted in this phase.

## Existing baselines

- persistence
- fold-local StandardScaler + two-output LinearRegression

## Nonlinear candidate — Histogram Gradient Boosting

Use one fixed HistGradientBoostingRegressor per output through MultiOutputRegressor.

Fixed parameters:
- max_iter = 150
- learning_rate = 0.05
- max_leaf_nodes = 15
- min_samples_leaf = 20
- l2_regularization = 1.0
- random_state = 2026

No validation-fold information is used for fitting.

## Selection rule

The nonlinear candidate may replace the linear baseline for the final holdout only if all are true:

1. Its sample-weighted mean coordinate distance across 1/2/4/8-week development predictions improves on linear by at least 3%.
2. It improves mean coordinate distance versus linear at at least 3 of the 4 horizons.
3. Across the 44 horizon-sector cells, the median relative coordinate-distance change versus linear is < 0.
4. Its sample-weighted macro F1 is not worse than linear by more than 0.01.

If it does not qualify, linear remains the frozen model for final holdout.

The final holdout must remain unopened until this rule has been applied.
