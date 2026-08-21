# Model Selection Record — 2026-08-20

## Status

This record applies the pre-declared development-set selection rule before the final holdout is opened.

The final 52 completed RRG observations from 2025-08-22 through 2026-08-14 remain untouched.

## Candidate freeze

The nonlinear candidate was fixed before its performance was summarized:

- model: `HistGradientBoostingRegressor` through `MultiOutputRegressor`;
- max_iter: 150;
- learning_rate: 0.05;
- max_leaf_nodes: 15;
- min_samples_leaf: 20;
- l2_regularization: 1.0;
- random_state: 2026;
- no hyperparameter search.

The amended freeze file SHA-256 is:

`18d89f450aca183cfe645d02b081bd198eb3b3447ad03b15993873adf888367f`

A previously proposed 300-tree Random Forest was removed before performance was summarized because a timing check made the full 11-sector evaluation impractical in the current environment. No Random Forest predictive result was reviewed.

## Development results

| Horizon | Linear quadrant accuracy | HGB quadrant accuracy | Linear coordinate distance | HGB coordinate distance |
|---:|---:|---:|---:|---:|
| 1 week | 82.5% | 74.1% | 4.400 | 6.482 |
| 2 week | 75.6% | 64.4% | 5.991 | 8.406 |
| 4 week | 63.9% | 53.8% | 8.257 | 10.635 |
| 8 week | 48.9% | 41.8% | 11.492 | 12.910 |

## Selection-rule outcome

Pre-declared rule:

1. improve sample-weighted mean coordinate distance over linear by at least 3%;
2. improve coordinate distance at at least 3 of 4 horizons;
3. median relative coordinate-distance change across 44 horizon-sector cells < 0;
4. sample-weighted macro F1 no worse than linear by more than 0.01.

Observed:

- sample-weighted coordinate distance:
  - linear: 7.458
  - HGB: 9.544
  - HGB is **28.0% worse**, not better;
- horizons with lower coordinate distance than linear: **0 / 4**;
- horizon-sector cells improved: **2 / 44**;
- median relative coordinate-distance change: **35.3% worse**;
- sample-weighted macro F1:
  - linear: 0.644
  - HGB: 0.551
  - difference: -0.093.

**Decision: HistGradientBoosting fails the selection rule. LinearRegression remains the frozen predictive model for the final holdout.**

## Interpretation

This negative result is useful. A more complex nonlinear model did not improve the development benchmark under a fixed, no-search specification. The reconstruction therefore does not escalate model complexity merely because the original 2025 presentation used AI/ML language.

The final holdout should still remain closed until:
- the linear specification and evaluation code are frozen;
- CI is added and passes;
- the one-time holdout evaluation script is separated from model-selection code.
