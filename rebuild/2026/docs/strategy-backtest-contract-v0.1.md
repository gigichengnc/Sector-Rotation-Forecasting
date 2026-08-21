# Strategy Backtest Contract v0.1 — frozen before return inspection

Date: 2026-08-20

## Research status

This is an **exploratory economic-value backtest** performed after the predictive
final holdout was opened. It is not a new untouched confirmatory holdout.

The strategy rules below are fixed before any strategy return, CAGR, Sharpe,
drawdown, or turnover result is inspected. No parameter search is permitted on
this backtest output.

## Universe and data

- Benchmark/reference asset for RRG-style calculation: SPY.
- Tradable sector universe: XLB, XLC, XLE, XLF, XLI, XLK, XLP, XLRE, XLU, XLV, XLY.
- Prices: adjusted-close total-return proxy from the versioned Yahoo dataset.
- Weekly signal state: last available completed trading observation for each Friday week.
- Common-history gate: all required symbols must have genuine observations; no forward fill.

## Signal model

- Frozen model family: `StandardScaler -> LinearRegression`.
- Forecast horizon: **1 weekly observation**.
- Lookback: **20 weekly RRG-style coordinate observations**.
- Minimum supervised training rows before a historical signal is eligible: **156**.
- Historical model fitting is expanding and point-in-time: for each decision week,
  the model may use only labels observable by that decision week.

## Primary forecast strategy

At each eligible completed Friday decision week:

1. forecast next-week RS-Ratio and RS-Momentum for every sector;
2. calculate `forecast_strength = predicted_rs_ratio + predicted_rs_momentum`;
3. rank descending by `forecast_strength`, with symbol ascending as deterministic tie-break;
4. hold the top **3** sectors;
5. target weight is exactly **1/3** in each selected sector and zero in the other sectors.

No cash filter, confidence filter, quadrant filter, volatility filter, or stop-loss is used.

## Persistence comparator

Apply the identical top-3/equal-weight portfolio rule, but rank sectors by
`current_rs_ratio + current_rs_momentum` at the decision week. This tests whether
forecasting adds economic value beyond simply acting on the current RRG-style state.

## Other comparators

- `equal_weight_11_weekly`: rebalance all 11 sector ETFs to equal weight every week.
- `spy_buy_hold`: 100% SPY from the first execution date; no subsequent rebalance.

## Decision and execution convention

- Decision timestamp: completed Friday week-end after the Friday market observation is known.
- Execution timestamp: the **first later daily timestamp common to SPY and all 11 sector ETFs**.
- Therefore the strategy never assumes execution at the same close used to form the signal.
- A position is held from its execution timestamp to the next scheduled common execution timestamp.
- The final signal is not scored unless a later common execution timestamp exists.

## Costs and turnover

- Transaction cost: **10 basis points (0.001) per dollar traded**.
- Gross traded weight at a rebalance is `sum(abs(target_weight - pretrade_weight))`.
- Cost fraction is `0.001 * gross_traded_weight`.
- Pretrade weights include drift from the previous holding-period returns.
- Initial entry from cash incurs trading cost.
- No taxes, bid/ask model beyond the fixed cost, market impact, borrow, leverage, or financing.

## Performance outputs

Report gross and net portfolio values/returns, plus:

- cumulative net return;
- CAGR using actual elapsed calendar days;
- annualized volatility from weekly net returns (sqrt(52));
- annualized Sharpe ratio using **0% risk-free rate**, explicitly as a comparison convention;
- maximum drawdown;
- mean weekly gross traded weight;
- total transaction cost fraction paid;
- number of scored holding intervals.

## Interpretation

Prediction accuracy and investment performance remain separate experiments.
A profitable historical backtest does not make the forecast causal, guarantee
future profitability, or turn the RRG-style coordinates into proprietary JdK metrics.
A poor backtest is retained as a valid negative result.
