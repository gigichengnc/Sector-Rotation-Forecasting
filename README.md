# RRG Research Reconstruction

A sanitized public research reconstruction of a **2025 student sector-rotation prototype** that explored whether Relative Rotation Graph (RRG) states could be forecast rather than used only descriptively.

This repository is intentionally split into:

- `historical/` — a **curated audited subset** of a private 137-file recovered historical source archive, plus the complete file/hash inventory;
- `rebuild/2026/` — the reproducible Python reconstruction, tests, experiment contracts, compact results, prospective forecast mode, and a separately declared strategy backtest.

The full historical archive, original development repository history, raw market-data archive, private links, and private commit metadata are **not** published here.

## Main finding

The 2026 reconstruction uses a transparent RRG-style coordinate calculation and a deliberately simple two-output model:

`20 weekly RRG-style coordinate observations -> StandardScaler -> LinearRegression -> future (RS-Ratio, RS-Momentum)`

On a frozen 52-week final holdout across 11 US sector ETFs, LinearRegression outperformed current-state persistence at all four horizons:

| Horizon | Linear quadrant accuracy | Persistence | Coordinate-distance reduction |
|---:|---:|---:|---:|
| 1 week | **80.6%** | 74.0% | 32.8% |
| 2 weeks | **75.5%** | 62.8% | 35.8% |
| 4 weeks | **69.9%** | 50.0% | 40.2% |
| 8 weeks | **51.7%** | 29.5% | 44.3% |

Each model/horizon result contains **572 observations = 52 target weeks × 11 sectors**.

This is evidence about prediction in this project's **RRG-style state space**. It is not a claim of proprietary JdK equivalence, calibrated probability, expected return, or trading profitability.

## Prediction quality did not automatically become economic value

A separate strategy hypothesis was declared before inspecting returns:

- use the 1-week Linear forecast;
- rank sectors by predicted `RS-Ratio + RS-Momentum`;
- hold the top 3 equal-weight;
- rebalance weekly;
- execute at the next common trading-day adjusted close after the Friday decision;
- charge 10 bps per dollar traded.

The hypothesis was **not supported**:

| Strategy | Net CAGR | Max drawdown | Sharpe (0% rf) |
|---|---:|---:|---:|
| Forecast top-3 | 8.6% | -26.8% | 0.605 |
| Persistence top-3 | 9.5% | -22.4% | 0.662 |
| Equal-weight 11 sectors | 11.3% | -17.9% | 0.815 |
| SPY buy-and-hold | **14.8%** | -20.5% | **0.923** |

That negative result is deliberately retained. Better future RRG-state prediction did not, under this first mapping, imply better next-week sector-return ranking or a superior portfolio.

## Why the historical audit matters

The recovered 2025 project contained substantial Rust implementation: a Yahoo Finance data path, an RRG calculator, ML research modules, an API/web layer, backtest structures, and a demo frontend. The audit also found material gaps:

- the web ML dependency was disabled and prediction routes were placeholders;
- a displayed `0.75` model accuracy was hard-coded rather than a reproduced experiment;
- the historical “LSTM” was an LSTM-like state processor, not a trained LSTM network;
- preprocessing and walk-forward slicing created validation concerns;
- VIX/rate features were simulated in the surviving source;
- parts of the historical backtest were placeholders;
- demo UI outputs included random/static values.

See [`historical/2025-source-audit.md`](historical/2025-source-audit.md) and [`historical/PUBLIC-SUBSET.md`](historical/PUBLIC-SUBSET.md).

## Reproduce the software tests

Use Python 3.11. The successful private PR CI environment resolved to Python 3.11.16 and the package versions recorded in `requirements-lock.txt`.

```bash
cd rebuild/2026
python -m venv .venv
# activate the environment
python -m pip install -r requirements-lock.txt
python -m pip install -e . --no-deps --no-build-isolation
pytest
```

The private integration PR recorded **44 passing tests** before this sanitized export. A new public repository should run the included GitHub Actions workflow again from a clean history.

## Data policy

Raw Yahoo Finance payloads and the frozen market-data ZIP are not committed to this public bundle. The fetcher, data schema/provenance logic, run markers, result tables, and hashes are retained so that the research trail is explicit without publishing a large data archive.

## Repository status

This bundle is prepared for creation of a **new clean Git repository**. Do not copy `.git/` history from the private development repository.

Before the first public commit, use a GitHub `users.noreply.github.com` commit identity and complete [`PUBLIC-RELEASE-CHECKLIST.md`](PUBLIC-RELEASE-CHECKLIST.md).

## Limits

This project is a research reconstruction, not investment advice, not a production trading system, and not a promise of investment performance.
