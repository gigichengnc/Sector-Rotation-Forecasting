# RRG 2026 Prospective Forecast Preview

## Status

This is a prospective deployment forecast generated after the final holdout result
was frozen. It is **not** a new holdout evaluation and must not be interpreted as
a trading recommendation.

- decision week: 2026-08-14
- benchmark: SPY
- sectors: 11 Select Sector ETFs
- model: StandardScaler -> LinearRegression
- lookback: 20 completed weekly RRG-style observations
- horizons: 1, 2, 4, 8 weeks
- raw Yahoo payload hashes verified: 12/12
- confidence/probability output: disabled

## Sector quadrant outlook

| Sector | Current | 1W | 2W | 4W | 8W |
|---|---|---|---|---|---|
| XLB | Leading | Improving | Improving | Improving | Lagging |
| XLC | Improving | Improving | Improving | Improving | Leading |
| XLE | Leading | Leading | Leading | Leading | Lagging |
| XLF | Leading | Leading | Leading | Weakening | Weakening |
| XLI | Leading | Leading | Leading | Lagging | Lagging |
| XLK | Lagging | Lagging | Lagging | Lagging | Improving |
| XLP | Leading | Improving | Improving | Lagging | Lagging |
| XLRE | Improving | Improving | Leading | Leading | Lagging |
| XLU | Improving | Improving | Improving | Lagging | Improving |
| XLV | Leading | Leading | Leading | Lagging | Lagging |
| XLY | Improving | Improving | Improving | Improving | Improving |

## Aggregate quadrant counts

- 1W: Leading 4, Improving 6, Weakening 0, Lagging 1
- 2W: Leading 5, Improving 5, Weakening 0, Lagging 1
- 4W: Leading 2, Improving 3, Weakening 1, Lagging 5
- 8W: Leading 1, Improving 3, Weakening 1, Lagging 6

## Interpretation limits

The forecast predicts future coordinates in this project's transparent RRG-style
state space. It does not output a calibrated probability, expected return, buy/sell
signal, portfolio weight, or trading profitability estimate.

Longer-horizon forecasts should not be read as more certain merely because the
project produces a point estimate. The validated final holdout itself showed lower
quadrant accuracy at 8 weeks than at 1–4 weeks.

## Deployment distinction

The final holdout is already frozen and must not be reused for model selection.
For prospective deployment, the selected linear model may be refitted using all
labels that are known by the current decision date, including historical observations
that happened to fall inside the former holdout period. This is operational fitting,
not retrospective re-scoring.
