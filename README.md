# RRG Research Reconstruction

[![Tests](https://github.com/gigichengnc/RRG-Research-Reconstruction/actions/workflows/tests.yml/badge.svg)](https://github.com/gigichengnc/RRG-Research-Reconstruction/actions/workflows/tests.yml)
[![Python](https://img.shields.io/badge/python-3.11-blue.svg)](pyproject.toml)
[![Research status](https://img.shields.io/badge/status-research%20toolkit-orange.svg)](#limits)

An installable research toolkit and retrospective reconstruction of a **2025 student sector-rotation prototype** that explored whether Relative Rotation Graph (RRG) states could be forecast rather than used only descriptively.

The repository has two deliberately separate jobs:

- `historical/` preserves a **curated audited subset** of a recovered 137-file historical source archive, plus the complete file/hash inventory;
- `rebuild/2026/` contains the reproducible forecasting reconstruction, validation contracts, recorded results, prospective forecast logic, and a separately declared economic-value backtest.

The full historical archive, original development Git history, and frozen raw market-data archive are not published here.

> **Main research lesson:** better prediction of future RRG-style coordinates did **not** automatically translate into a better trading strategy.

## What can you do with it?

### 1. Generate a fresh prospective RRG-style forecast

After installation, run:

```bash
rrg-research forecast
```

The command:

```text
fetches SPY + 11 US sector ETFs from Yahoo Finance
        ↓
requires adjusted close and records source provenance + SHA-256
        ↓
excludes incomplete weekly observations
        ↓
aligns one common weekly history
        ↓
calculates transparent causal RRG-style coordinates
        ↓
fits the frozen StandardScaler -> LinearRegression deployment workflow
        ↓
forecasts 1 / 2 / 4 / 8-week future RRG-style coordinates and quadrants
        ↓
writes a forecast table, detailed rows, run marker, and portable market-data ZIP
```

A default run creates a timestamped directory under `outputs/` containing:

```text
forecast-YYYYMMDDTHHMMSSZ/
├── market-data/
│   ├── manifest.csv
│   ├── raw/yahoo/
│   └── processed/daily/
├── market-data.zip
├── forecast_table.csv
├── forecast_long.csv
└── RUN-MARKER.json
```

`forecast_table.csv` is the compact human-readable output. It reports the current quadrant and the model's predicted RRG-style coordinates/quadrants at each horizon.

It does **not** report expected returns, calibrated probabilities, buy/sell signals, or a claim of profitable trading performance.

### 2. Fetch a versioned market-data archive without forecasting

```bash
rrg-research fetch
```

This preserves the provider URL, acquisition time, raw-payload hash, relative raw-data path, processed adjusted-close series, and a portable ZIP for later research/reproduction.

### 3. Reuse the core research functions from Python

The public package surface includes the transparent RRG-style calculator and the frozen prospective deployment workflow:

```python
from rrg_rebuild import (
    DeploymentConfig,
    RRGConfig,
    calculate_rrg,
    run_fresh_forecast,
)
```

The import package remains `rrg_rebuild` to preserve the reconstruction lineage; the installable distribution is `rrg-research-reconstruction`, and the user-facing CLI is `rrg-research`.

### 4. Inspect or extend the methodology

Researchers can replace or add candidate models while retaining the same time-aware validation, persistence baseline, target definitions, and research boundaries. More model complexity is not treated as progress unless it improves out-of-sample evidence under the same protocol.

## Quick start

Clone the repository and, from the repository root:

```bash
python -m venv .venv
```

Activate it:

```text
Windows PowerShell: .\.venv\Scripts\Activate.ps1
macOS/Linux:        source .venv/bin/activate
```

For the recorded research environment:

```bash
python -m pip install -r rebuild/2026/requirements-lock.txt
python -m pip install -e . --no-deps --no-build-isolation
pytest
rrg-research version
```

Generate a fresh forecast:

```bash
rrg-research forecast
```

Choose an explicit output directory if desired:

```bash
rrg-research forecast --output-dir outputs/my-forecast
```

The GitHub Actions workflow also builds a wheel, installs that wheel into a clean virtual environment, and smoke-tests the installed `rrg-research` command.

## Main predictive finding

The 2026 reconstruction uses a transparent RRG-style coordinate calculation and a deliberately simple two-output model:

```text
20 weekly RRG-style coordinate observations
        ↓
StandardScaler
        ↓
LinearRegression
        ↓
future (RS-Ratio, RS-Momentum)
```

On a frozen 52-week final holdout across 11 US sector ETFs, LinearRegression outperformed current-state persistence at all four horizons:

| Horizon | Linear quadrant accuracy | Persistence | Coordinate-distance reduction |
|---:|---:|---:|---:|
| 1 week | **80.6%** | 74.0% | 32.8% |
| 2 weeks | **75.5%** | 62.8% | 35.8% |
| 4 weeks | **69.9%** | 50.0% | 40.2% |
| 8 weeks | **51.7%** | 29.5% | 44.3% |

Each model/horizon result contains **572 observations = 52 target weeks × 11 sectors**.

This is evidence about prediction in this project's **RRG-style state space**. It is not a claim of proprietary JdK equivalence, calibrated probability, expected return, or trading profitability.

The final predictive holdout was opened once under a frozen protocol. It is a recorded evaluation artifact, not a benchmark to keep reopening while tuning new models.

## Prediction quality did not automatically become economic value

A separate strategy hypothesis was declared before inspecting its returns:

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

## Research architecture

```text
HISTORICAL 2025 SOURCE
        ↓
retrospective audit
        ↓
2026 research contract
        ↓
adjusted-close market data + provenance
        ↓
completed-week/common-history gate
        ↓
transparent RRG-style coordinates
        ↓
time-aware development validation
        ↓
model selection
        ↓
one-time frozen predictive holdout
        ↓
prospective deployment forecast
        ↓
separate economic-value hypothesis
```

The package CLI exposes the **prospective deployment** path. Historical audit tools, final-holdout guards, model-selection evidence, and strategy experiments remain research artifacts rather than ordinary end-user commands.

## Repository map

```text
.
├── pyproject.toml                    # installable package + rrg-research CLI
├── historical/                       # audited 2025 evidence and source manifest
├── rebuild/2026/
│   ├── src/rrg_rebuild/              # research/library implementation
│   ├── tests/                        # unit/regression tests
│   ├── docs/                         # frozen research and validation contracts
│   ├── scripts/                      # research/reproduction runners
│   ├── results/                      # compact recorded experiment evidence
│   └── requirements-lock.txt         # frozen successful research environment
└── .github/workflows/tests.yml       # tests + clean-wheel validation
```

## Data and reproducibility policy

Fresh CLI runs keep local raw Yahoo payloads, hashes, processed adjusted-close files, and a manifest so the acquisition trail is explicit. Those generated local outputs are ignored by Git and are not meant to be committed automatically.

The historical frozen market-data ZIP used for the recorded 2026 evaluation is not published in this repository. Its hashes and compact result artifacts remain part of the research trail.

Provider endpoints and provider access policies can change independently of this project. A successful historical run does not guarantee that a third-party endpoint will remain available forever.

## Limits

This project is a **research reconstruction and forecasting toolkit**. It is not:

- investment advice;
- a production trading system;
- a claim of proprietary JdK RS-Ratio / RS-Momentum equivalence;
- an expected-return model;
- a calibrated probability model;
- evidence that the first tested trading strategy was profitable or superior;
- proof that improved coordinate prediction will generalise into economic alpha.

The most important retained result is therefore two-sided:

> The frozen LinearRegression workflow improved prediction of future RRG-style states versus persistence, while the first pre-declared top-3 portfolio mapping failed to outperform simpler comparators.
