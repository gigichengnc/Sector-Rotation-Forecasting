# RRG 2026 Strategy Backtest v0.1

## Status

This is the first pre-declared **exploratory economic-value test**. The rules were frozen before any strategy return was inspected. It is not an untouched confirmatory holdout, because the predictive final holdout had already been opened before this strategy experiment was designed.

Primary strategy: 1-week frozen Linear forecast -> rank sectors by predicted RS-Ratio + predicted RS-Momentum -> equal-weight top 3 -> rebalance weekly at the first common trading-day adjusted close after the Friday decision -> 10 bps per dollar traded.

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

## Decision

**The primary forecast-top-3 strategy is not supported by this backtest.**

It underperformed the persistence top-3 comparator even before transaction costs, and its higher turnover increased the gap after costs. It also underperformed the equal-weight sector portfolio and SPY buy-and-hold over the scored period.

The backtest spans **2022-03-21 13:30:00+00:00 to 2026-08-17 13:30:00+00:00**, with **230** scored weekly holding intervals.

This does **not** invalidate the predictive holdout result. It demonstrates a different point: improved prediction of future RRG-style coordinates does not automatically imply that ranking sectors by those predicted coordinates produces superior next-week investment returns.

## Research consequence

Do not tune top-k, transaction cost, or forecast horizon on this output and then present the tuned result as confirmation. Any new strategy mapping should be treated as a new hypothesis, pre-declared separately, and preferably evaluated prospectively or through a nested development/validation design.
