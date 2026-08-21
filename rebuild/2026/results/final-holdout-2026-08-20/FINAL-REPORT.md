# RRG 2026 Final Holdout Result

## v0.2 post-publication correction

This file preserves the original v0.1 holdout outputs, but **withdraws the earlier interpretation that the result demonstrated market predictability**.

A later structural-null benchmark showed that the same coordinate-forecast pipeline produces a similar linear-vs-persistence advantage on synthetic geometric random walks with no serial predictability. The old result therefore remains a record of what the model scored, not evidence that recent sector rotation contains forecastable market information.

See `rebuild/2026/docs/v0.2-null-audit.md`.

## Original frozen holdout protocol

Protocol: `rrg-final-holdout-v1-2026-08-20`

- target window: 2025-08-22 through 2026-08-14 UTC
- 52 target weeks per sector/horizon
- 11 sector ETFs
- 572 **scored rows** per model/horizon
- lookback: 20 weekly observations
- frozen model: `StandardScaler -> LinearRegression`
- refit policy: `fixed_at_first_holdout_decision_per_symbol_horizon`
- holdout reuse allowed: **false**

The 572 rows are not 572 independent observations. Sectors share the same benchmark, cross-sectional returns are correlated, and multi-week targets overlap through time.

## Recorded aggregate outputs

| Horizon | Linear accuracy | Persistence accuracy | Linear advantage | Linear coordinate distance | Persistence distance | Distance reduction |
|---:|---:|---:|---:|---:|---:|---:|
| 1 week | 80.6% | 74.0% | +6.6 pp | 4.404 | 6.554 | 32.8% |
| 2 weeks | 75.5% | 62.8% | +12.8 pp | 5.983 | 9.317 | 35.8% |
| 4 weeks | 69.9% | 50.0% | +19.9 pp | 8.180 | 13.687 | 40.2% |
| 8 weeks | 51.7% | 29.5% | +22.2 pp | 11.508 | 20.651 | 44.3% |

These are historical scoring outputs in the project's transparent RRG-style state space. They are not trading returns, expected returns, calibrated probabilities, or evidence of proprietary JdK equivalence.

## Structural-null comparison added in v0.2

Across 12 no-signal synthetic trials using the same target mechanics and holdout geometry:

| Horizon | Null Linear accuracy | Null persistence | Null edge |
|---:|---:|---:|---:|
| 1 week | 81.0% | 72.8% | +8.2 pp |
| 2 weeks | 73.8% | 63.2% | +10.6 pp |
| 4 weeks | 63.2% | 50.5% | +12.8 pp |
| 8 weeks | 47.4% | 31.9% | +15.5 pp |

At one week, the no-signal Linear accuracy is approximately the same as the real-data value. This is the material reason the old market-signal interpretation is retired.

The null does not prove that no residual signal exists at longer horizons. It shows that persistence was an inadequate sole benchmark and that any residual edge must be evaluated against structural-null behaviour with dependence-aware uncertainty.

## Development versus final Linear

| Horizon | Development accuracy | Final accuracy | Change | Development distance | Final distance |
|---:|---:|---:|---:|---:|---:|
| 1 week | 82.5% | 80.6% | -1.9 pp | 4.400 | 4.404 |
| 2 weeks | 75.6% | 75.5% | -0.1 pp | 5.991 | 5.983 |
| 4 weeks | 63.9% | 69.9% | +6.1 pp | 8.257 | 8.180 |
| 8 weeks | 48.9% | 51.7% | +2.9 pp | 11.492 | 11.508 |

v0.1 described these values as consistent with the frozen model retaining its development behaviour. **v0.2 withdraws that interpretation.** A holdout being better than development is not evidence of generalisation or stability. These differences may reflect sampling variation, period difficulty, dependence, or regime composition.

No formal confidence interval or dependence-aware hypothesis test was attached to the original table.

## Execution note

The first authorized invocation of the frozen runner stopped before any RRG calculation, target materialization, prediction, or performance metric was produced because the processed CSV schema did not match the runner.

A schema-only adapter then mapped:

- `source_timestamp_utc` -> `timestamp`
- `adjusted_close` -> `adj_close`
- added the known symbol column

No model, target, split, price values, horizon, lookback, RRG formula, training rule, or evaluation metric was changed. No holdout result had been observed before the corrected execution.

This execution detail is kept at the same level as the result rather than only in a deeper appendix.

## Current interpretation

The defensible statement is now:

> The frozen v0.1 linear model outscored current-state persistence on the recorded coordinate holdout, but a post-publication structural-null benchmark reproduced much of that advantage without market predictability. The old holdout therefore does not establish forecastable sector-market signal.

The 2025-08-22 to 2026-08-14 holdout is retired for new confirmatory claims and must not be reused as an untouched test for redesigned targets or new models.
