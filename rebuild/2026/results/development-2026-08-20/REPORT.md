# RRG 2026 Development Out-of-Sample Benchmark

## Status

This is the first real-market-data benchmark for the 2026 reconstruction.

It is a **development out-of-sample benchmark**, not the final holdout result.

The final 52 completed weekly RRG observations were reserved untouched:

- holdout start: 2025-08-22
- holdout end: 2026-08-14

The incomplete week ending 2026-08-21 was excluded because the dataset was fetched on 2026-08-20.

## Data

- provider: Yahoo Finance chart endpoint
- price field: adjusted close
- benchmark: SPY
- sectors: XLB, XLC, XLE, XLF, XLI, XLK, XLP, XLRE, XLU, XLV, XLY
- common weekly history starts: 2018-06-22
- complete common weekly observations: 426
- RRG-style observations per sector after calculation warm-up: 406

All 12 raw Yahoo payload hashes matched the SHA-256 values recorded in the supplied manifest.

## Model/validation contract

- weekly observations
- RRG-style config: ratio 10, momentum 10, rolling normalization 100, scale 10
- model lookback: 20 observations
- horizons: 1, 2, 4, 8 observations
- initial training window: 156 model-ready rows
- validation window: 13 rows
- expanding training
- purge gap = forecast horizon
- fold-local StandardScaler for the linear baseline
- validation windows do not overlap
- each model/timestamp/horizon is scored once

## Aggregate development OOS results

| Horizon | Model | n | Quadrant accuracy | Macro F1 | Mean coordinate distance |
|---:|---|---:|---:|---:|---:|
| 1 | linear | 1859 | 82.5% | 0.785 | 4.400 |
| 1 | persistence | 1859 | 74.1% | 0.684 | 6.302 |
| 2 | linear | 1859 | 75.6% | 0.710 | 5.991 |
| 2 | persistence | 1859 | 65.1% | 0.592 | 8.793 |
| 4 | linear | 1859 | 63.9% | 0.597 | 8.257 |
| 4 | persistence | 1859 | 51.2% | 0.449 | 12.913 |
| 8 | linear | 1716 | 48.9% | 0.470 | 11.492 |
| 8 | persistence | 1716 | 29.8% | 0.282 | 18.904 |

## Interpretation rule

These quadrant accuracies are derived from continuous coordinate forecasts.
They are not trading returns and are not directly comparable to an undocumented
historical “65–75% accuracy” claim unless the historical target and validation
procedure are known to be the same.

The persistence baseline is particularly important: short-horizon quadrant
accuracy can be high simply because RRG states are persistent. A complex model
must therefore beat persistence, not merely report a large-looking accuracy.

The untouched holdout should not be opened until the modelling/selection rules
for the next phase are frozen.
