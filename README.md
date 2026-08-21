# Sector Rotation Forecasting Reconstruction

[![Tests](https://github.com/gigichengnc/Sector-Rotation-Forecasting/actions/workflows/tests.yml/badge.svg)](https://github.com/gigichengnc/Sector-Rotation-Forecasting/actions/workflows/tests.yml)
[![Python](https://img.shields.io/badge/python-3.11-blue.svg)](pyproject.toml)
[![Research status](https://img.shields.io/badge/status-research%20toolkit-orange.svg)](#limits)

I first conceived and started this **sector-rotation forecasting project for an HKSI competition in December 2025**. This repository documents how I later audited, reconstructed, and developed the original idea into a more reproducible research workflow.

The original idea was simple:

> Can the future relative-strength and momentum state of US market sectors be forecast from their recent rotation history, rather than used only as a descriptive snapshot?

The historical project was inspired by Relative Rotation Graph (RRG) concepts. In plain language, the reconstructed state space compares each sector with a benchmark using two dimensions — **relative strength** and **momentum** — and maps the result into four states: **Leading, Weakening, Lagging, or Improving**.

This repository implements an independently developed, transparent **RRG-style approximation**. It does **not** claim proprietary JdK RS-Ratio / RS-Momentum equivalence.

> **Main research lesson:** a simple forecasting model improved prediction of future RRG-style sector states versus a persistence baseline, but the first pre-declared trading strategy built from those forecasts did **not** outperform simpler comparators.

References to HKSI, Relative Rotation Graph (RRG), and JdK terminology are descriptive historical or methodological references only. This independent repository is not affiliated with or endorsed by HKSI or RRG Research.

## Project snapshot

| Item | Current project state |
| --- | --- |
| Project origin | Conceived and initiated by the repository author for an HKSI competition in December 2025 |
| Original question | Can future sector relative-strength / momentum states be forecast? |
| Historical implementation | Substantial 2025 Rust prototype preserved as a curated audited evidence subset |
| 2026 reconstruction | Reproducible Python forecasting pipeline with causal, time-aware validation |
| Reference model | 20 weeks of state history → `StandardScaler` → two-output `LinearRegression` |
| Predictive evidence | Outperformed persistence on the frozen 52-week holdout at 1/2/4/8-week horizons |
| Economic-value test | **Not supported** — the first forecast-driven top-3 strategy underperformed simpler comparators |
| Current usable tool | `sector-rotation forecast` |
| Interpretation boundary | Forecasts RRG-style states, **not expected returns, probabilities, or buy/sell signals** |

## If you just want to use it

You do **not** need to open or run the research files manually.

For a normal forecast run, you only need:

- Python 3.11;
- this repository;
- an internet connection for fresh market data.

Start with `README.md`, install the package from the repository root, then use the `sector-rotation` command. The installer handles the package files for you.

| Path | Do you need to open it? | Purpose |
| --- | --- | --- |
| `README.md` | **Yes — start here** | Project overview and usage instructions |
| `pyproject.toml` | No manual editing | Defines the installable package and `sector-rotation` command |
| `rebuild/2026/requirements-lock.txt` | No manual editing | Tested dependency versions used by the recorded research environment |
| `rebuild/2026/src/rrg_rebuild/` | No | Actual Python implementation used by the installed package |
| `historical/` | No | Historical audit evidence and recovered-source inventory |
| `rebuild/2026/docs/` | No | Research methodology and validation contracts |
| `rebuild/2026/results/` | No | Recorded experiment evidence |
| `rebuild/2026/scripts/` | Advanced/reproduction only | Research runners, holdout reproduction, and strategy experiments |
| `rebuild/2026/tests/` | No | Automated validation tests |

Clone and enter the repository:

```bash
git clone https://github.com/gigichengnc/Sector-Rotation-Forecasting.git
cd Sector-Rotation-Forecasting
```

Create and activate a virtual environment:

```bash
python -m venv .venv
```

```text
Windows PowerShell: .\.venv\Scripts\Activate.ps1
macOS/Linux:        source .venv/bin/activate
```

Install the tested dependencies and this package:

```bash
python -m pip install -r rebuild/2026/requirements-lock.txt
python -m pip install -e . --no-deps --no-build-isolation
```

Check the installation and run a forecast:

```bash
sector-rotation version
sector-rotation forecast
```

After installation, normal users interact with the project through `sector-rotation`. You do **not** need to run individual `.py` files.

## From the 2025 prototype to the 2026 reconstruction

The 2025 project contained real implementation work, but the surviving source also exposed important methodological gaps. The reconstruction keeps the history visible rather than silently rewriting it.

| 2025 prototype | 2026 reconstruction |
| --- | --- |
| RRG implementation without evidence of proprietary JdK equivalence | Explicit transparent **RRG-style approximation** with a stated scope |
| Web prediction routes included placeholders | Prospective forecast output comes from an actually fitted model workflow |
| A displayed `0.75` model accuracy was hard-coded | Reported accuracy comes from explicit out-of-sample evaluation |
| Historical “LSTM” was not a trained LSTM network | Model descriptions match the implementation actually evaluated |
| Preprocessing / walk-forward slicing raised leakage concerns | Fold-local preprocessing and chronological validation |
| Surviving VIX/rate features were simulated | Rebuild uses reproducible adjusted-close inputs only |
| Parts of the historical backtest were placeholders | Economic value is tested separately under a pre-declared strategy contract |
| Prediction and trading value could be conflated | Predictive performance and portfolio performance are reported separately |

The full historical archive, original development Git history, and frozen raw market-data archive are not published in this repository. `historical/` instead contains a **curated audited subset** plus the complete recovered source file/hash inventory.

## What can you do with it?

### 1. Generate a fresh prospective sector-state forecast

After installation, run:

```bash
sector-rotation forecast
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
fits the pre-specified StandardScaler -> LinearRegression deployment pipeline
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

`forecast_table.csv` is the compact human-readable output. Conceptually it looks like this:

```text
symbol   current      1w          2w          4w          8w
XLK      Improving    Leading     Leading     Weakening   Weakening
XLE      Lagging      Improving   Improving   Leading     Leading
...
```

The rows above are **illustrative format only**, not a current market forecast.

The real output reports current and predicted RRG-style coordinates/quadrants. It does **not** report expected returns, calibrated probabilities, buy/sell signals, or a claim of profitable trading performance.

### 2. Fetch a versioned market-data archive without forecasting

```bash
sector-rotation fetch
```

This preserves the provider URL, acquisition time, raw-payload hash, relative raw-data path, processed adjusted-close series, and a portable ZIP for later research/reproduction. Repeated default runs use timestamped locations so prior archives are not silently overwritten.

### 3. Use the core research functions from Python

The public package surface includes the transparent RRG-style calculator and the prospective deployment workflow:

```python
from rrg_rebuild import (
    DeploymentConfig,
    RRGConfig,
    calculate_rrg,
    run_fresh_forecast,
)
```

The internal Python import package remains `rrg_rebuild` to preserve the reconstruction lineage. The installable distribution is **`sector-rotation-research`**, and the user-facing CLI is **`sector-rotation`**.

The software is technically installable and executable, but this repository does **not yet grant an open-source license**. See [`NOTICE.md`](NOTICE.md) before reuse, redistribution, or incorporation into another project.

### 4. Inspect or extend the methodology

Researchers can inspect or locally modify candidate models while retaining the same time-aware validation, persistence baseline, target definitions, and research boundaries. More model complexity is not treated as progress unless it improves out-of-sample evidence under the same protocol.

## Quick start for research/reproduction

The section above is enough for normal use. If you also want to run the test suite and reproduce the recorded research environment, from the repository root:

```bash
python -m pip install -r rebuild/2026/requirements-lock.txt
python -m pip install -e . --no-deps --no-build-isolation
pytest
sector-rotation version
```

Generate a fresh forecast:

```bash
sector-rotation forecast
```

Choose an explicit output directory if desired:

```bash
sector-rotation forecast --output-dir outputs/my-forecast
```

The GitHub Actions workflow also builds a wheel, installs that wheel into a clean virtual environment, and smoke-tests the installed `sector-rotation` command.

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
├── README.md                         # start here: explanation + user instructions
├── pyproject.toml                    # installer/package metadata + sector-rotation CLI
├── NOTICE.md                         # rights and third-party naming notice
├── historical/                       # audited 2025 evidence and source manifest
├── rebuild/2026/
│   ├── src/rrg_rebuild/              # research/library implementation
│   ├── tests/                        # unit/regression tests
│   ├── docs/                         # frozen research and validation contracts
│   ├── scripts/                      # advanced research/reproduction runners
│   ├── results/                      # compact recorded experiment evidence
│   └── requirements-lock.txt         # tested research dependency environment
└── .github/workflows/tests.yml       # tests + clean-wheel validation
```

## Data and reproducibility policy

Fresh CLI runs keep local raw Yahoo payloads, hashes, processed adjusted-close files, and a manifest so the acquisition trail is explicit. Those generated local outputs are ignored by Git and are not meant to be committed automatically.

The historical frozen market-data ZIP used for the recorded 2026 evaluation is not published in this repository. Its hashes and compact result artifacts remain part of the research trail.

Provider endpoints and provider access policies can change independently of this project. A successful historical run does not guarantee that a third-party endpoint will remain available forever.

## Limits

This project is a **research reconstruction and sector-state forecasting toolkit**. It is not:

- investment advice;
- a production trading system;
- an official HKSI project or an HKSI-endorsed project;
- an official RRG Research product or an RRG Research-endorsed project;
- a claim of proprietary JdK RS-Ratio / RS-Momentum equivalence;
- an expected-return model;
- a calibrated probability model;
- evidence that the first tested trading strategy was profitable or superior;
- proof that improved coordinate prediction will generalise into economic alpha.

The most important retained result is therefore two-sided:

> The frozen LinearRegression workflow improved prediction of future RRG-style states versus persistence, while the first pre-declared top-3 portfolio mapping failed to outperform simpler comparators.
