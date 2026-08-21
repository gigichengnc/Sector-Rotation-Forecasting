# RRG 2026 Final Holdout Result

## Status

The frozen final holdout has now been opened and scored.

**Do not use these results for further model selection or tuning.**

Protocol: `rrg-final-holdout-v1-2026-08-20`

- target window: 2025-08-22 23:59:59.999999999+00:00 through 2026-08-14 23:59:59.999999999+00:00
- 52 target weeks per sector/horizon
- 11 sector ETFs
- 572 final observations per model/horizon
- lookback: 20 weekly observations
- frozen model: StandardScaler -> LinearRegression
- refit policy: `fixed_at_first_holdout_decision_per_symbol_horizon`
- holdout reuse allowed: **false**

## Final aggregate results

| Horizon | Linear accuracy | Persistence accuracy | Linear advantage | Linear coordinate distance | Persistence distance | Distance reduction |
|---:|---:|---:|---:|---:|---:|---:|
| 1 week | 80.6% | 74.0% | +6.6 pp | 4.404 | 6.554 | 32.8% |
| 2 week | 75.5% | 62.8% | +12.8 pp | 5.983 | 9.317 | 35.8% |
| 4 week | 69.9% | 50.0% | +19.9 pp | 8.180 | 13.687 | 40.2% |
| 8 week | 51.7% | 29.5% | +22.2 pp | 11.508 | 20.651 | 44.3% |

Across the four horizons, Linear averaged 69.4% quadrant accuracy
versus 54.1% for persistence. Using the aggregate coordinate
distance across horizons, Linear reduced error by
40.1%.

At the sector × horizon level, Linear had lower continuous coordinate error in
**44/44** cells. It had higher quadrant accuracy in **43/44**
cells and lower quadrant accuracy in **1/44** cells. The single
quadrant-accuracy loss was XLU at the 1-week horizon; its continuous coordinate
distance was still lower for Linear.

## Development vs final Linear

| Horizon | Development accuracy | Final accuracy | Change | Development distance | Final distance |
|---:|---:|---:|---:|---:|---:|
| 1 week | 82.5% | 80.6% | -1.9 pp | 4.400 | 4.404 |
| 2 week | 75.6% | 75.5% | -0.1 pp | 5.991 | 5.983 |
| 4 week | 63.9% | 69.9% | +6.1 pp | 8.257 | 8.180 |
| 8 week | 48.9% | 51.7% | +2.9 pp | 11.492 | 11.508 |

The final coordinate errors are extremely close to the development values at 1, 2,
and 8 weeks, and slightly better at 4 weeks. This is consistent with the frozen
linear baseline retaining its development behaviour on the untouched period.

These figures are prediction metrics in the project's transparent RRG-style state
space. They are **not trading returns** and do not validate the historical 2025
“65–75% accuracy” claim because the old target/validation contract was not recovered.

## Execution note

The first authorized invocation of the frozen runner stopped before any RRG
calculation, target materialization, prediction, or performance metric was produced.
The processed CSV schema used `source_timestamp_utc, adjusted_close`, while the runner
expected `timestamp, adj_close`.

A schema-only adapter then mapped:

- `source_timestamp_utc` -> `timestamp`
- `adjusted_close` -> `adj_close`
- added the known symbol column

No model, target, split, price values, horizon, lookback, RRG formula, training rule,
or evaluation metric was changed. The output directory did not exist after the failed
attempt, and no holdout result had been observed before the corrected execution.

The completed run then wrote `RUN-MARKER.json` with
`holdout_reuse_allowed: false`.

## Interpretation

The final result supports a narrower and more defensible conclusion than the 2025
presentation:

> In this 2026 reconstruction, a fixed two-output linear regression using 20 weeks
> of RRG-style coordinate history outperformed a current-state persistence baseline
> on the untouched 52-week holdout at all four forecast horizons.

It does not establish profitability, causality, or a proprietary JdK-equivalent model.
