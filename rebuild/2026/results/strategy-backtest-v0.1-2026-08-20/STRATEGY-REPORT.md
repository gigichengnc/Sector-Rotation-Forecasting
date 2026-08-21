# RRG 2026 Strategy Backtest v0.1

## Status

This is the first pre-declared **exploratory economic-value test**. The rules were frozen before any strategy return was inspected. It is not an untouched confirmatory holdout because the predictive coordinate holdout had already been opened before this strategy experiment was designed.

Primary strategy: 1-week frozen Linear coordinate forecast -> rank sectors by predicted RS-Ratio + predicted RS-Momentum -> equal-weight top 3 -> rebalance weekly at the first common trading-day adjusted close after the Friday decision -> 10 bps per dollar traded.

## Net results

| Strategy | Ending value | Cumulative return | CAGR | Volatility | Sharpe (0% rf) | Max drawdown |
|---|---:|---:|---:|---:|---:|---:|
| forecast_top3 | 1.441 | 44.1% | 8.6% | 15.7% | 0.605 | -26.8% |
| persistence_top3 | 1.491 | 49.1% | 9.5% | 15.5% | 0.662 | -22.4% |
| equal_weight_11_weekly | 1.603 | 60.3% | 11.3% | 14.4% | 0.815 | -17.9% |
| spy_buy_hold | 1.840 | 84.0% | 14.8% | 16.4% | 0.923 | -20.5% |

## Gross-vs-net diagnostic

| Strategy | Gross CAGR | Net CAGR | Mean weekly gross traded weight | Sum cost fractions |
|---|---:|---:|---:|---:|
| forecast_top3 | 11.9% | 8.6% | 55.9% | 12.9% |
| persistence_top3 | 12.5% | 9.5% | 51.6% | 11.9% |
| equal_weight_11_weekly | 11.4% | 11.3% | 1.8% | 0.4% |
| spy_buy_hold | 14.9% | 14.8% | 0.4% | 0.1% |

The scored path spans 2022-03-21 through 2026-08-17 and contains 230 weekly holding intervals.

## Decision

**The primary forecast-top-3 strategy is not supported by this backtest.**

It underperformed the persistence top-3 comparator even before transaction costs, and its higher turnover increased the gap after costs. It also underperformed the equal-weight sector portfolio and SPY buy-and-hold over the scored path.

That is the correct level of conclusion.

This experiment used one historical path and did not include a formal significance, confidence-interval, or power analysis. It therefore does **not** establish that the true strategy effect is exactly zero, and it does not rule out a smaller effect that the experiment lacked power to detect.

## v0.2 relation to the predictive result

v0.1 said this negative strategy result did not invalidate the positive coordinate holdout. v0.2 changes the picture: a later structural-null benchmark showed that much of the coordinate-prediction advantage can arise mechanically from the overlapping target construction even when the underlying returns contain no serial predictability.

The strategy failure is therefore no longer contrasted with a claimed market-predictive success. The two results now say:

1. the old coordinate model outscored persistence, but that advantage is largely explainable under a structural null;
2. the first portfolio mapping did not outperform its comparators on the tested path.

Neither statement proves the existence or absence of economically exploitable sector-rotation signal.

## Research consequence

Do not tune top-k, transaction cost, horizon, target, or model on this output and then present the tuned result as confirmation.

Any new strategy mapping should be treated as a new hypothesis. The v0.2 research path first asks whether RRG-style history predicts a **fully future relative-return target** beyond structural/base-rate benchmarks, with dependence-aware uncertainty. Economic-value testing comes only after that predictive question survives.
