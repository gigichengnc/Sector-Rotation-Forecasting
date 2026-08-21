# Sector Rotation Forecasting

[![Tests](https://github.com/gigichengnc/Sector-Rotation-Forecasting/actions/workflows/tests.yml/badge.svg)](https://github.com/gigichengnc/Sector-Rotation-Forecasting/actions/workflows/tests.yml)
[![Python](https://img.shields.io/badge/python-3.11-blue.svg)](pyproject.toml)
[![Research status](https://img.shields.io/badge/status-post--publication%20audit-orange.svg)](#current-research-conclusion)

**A reproducible audit and rebuild of a 2025 RRG-style sector-forecasting prototype.**

The main finding is now methodological, not a forecasting headline: the legacy coordinate target can produce a similar linear-model advantage on synthetic geometric random walks with no serial predictability. The old **80.6% vs 74.0%** result is therefore retained as experiment history but **retired as evidence of market predictability**.

The redesigned research question uses fully future sector return relative to SPY as the target. No new market-signal claim is made yet.

## Run it

```bash
python -m pip install -r rebuild/2026/requirements-lock.txt
python -m pip install -e . --no-deps --no-build-isolation
pytest
python rebuild/2026/scripts/run_synthetic_null.py
```

The package also retains the legacy coordinate-forecast workflow:

```bash
sector-rotation forecast
```

> **Interpretation boundary:** `sector-rotation forecast` produces auditable RRG-style coordinate forecasts, not expected returns, calibrated probabilities, buy/sell signals, or validated evidence of market predictability. The first forecast-driven top-3 strategy was not supported by its backtest.

This project was conceived for an HKSI competition in December 2025 and later reconstructed and audited independently. It is not affiliated with or endorsed by HKSI or RRG Research.

## Current research conclusion

The original 2026 reconstruction reported that a linear model beat current-state persistence when forecasting future RRG-style coordinates. That scoring result is still retained as a historical experiment record, but **the earlier interpretation that it demonstrated market predictability is retired**.

A post-publication structural-null test showed that the same pipeline produces a similar linear-vs-persistence advantage on synthetic geometric random walks with **no serial predictability by construction**. The main reason is target construction: the future rolling RRG-style coordinates share substantial information with the lagged coordinate features.

Across 12 deterministic no-signal trials:

| Horizon | Null Linear accuracy | Null persistence | Null edge |
| ---: | ---: | ---: | ---: |
| 1 week | **81.0%** | 72.8% | +8.2 pp |
| 2 weeks | **73.8%** | 63.2% | +10.6 pp |
| 4 weeks | **63.2%** | 50.5% | +12.8 pp |
| 8 weeks | **47.4%** | 31.9% | +15.5 pp |

The previously reported real-data 1-week result was **80.6% vs 74.0%**. Because a no-signal process reproduces the same order of magnitude, that result should not be presented as evidence of forecastable sector-market dynamics.

The strongest lesson of the repository is therefore methodological:

> **Causal validation is necessary, but it is not sufficient. The target itself also needs a structural null.**

See [`rebuild/2026/docs/v0.2-null-audit.md`](rebuild/2026/docs/v0.2-null-audit.md).

## Project snapshot

| Item | Current project state |
| --- | --- |
| Origin | Conceived and initiated by the repository author for an HKSI competition in December 2025 |
| 2025 source | Retrospectively audited Rust prototype; only a curated evidence subset is public |
| 2026 v0.1 | Time-aware coordinate-forecast reconstruction with a frozen holdout and separate strategy test |
| v0.2 correction | Structural-null benchmark shows much of the coordinate-prediction edge is mechanically induced |
| New primary target | Fully future sector return relative to SPY, with RRG-style history used as features |
| Model policy | Start with transparent regularized baselines; complexity must earn its place |
| Economic-value evidence | First forecast-top-3 strategy was **not supported by its backtest**, but that single path is not a proof of no alpha |
| Public package | Installable research toolkit and legacy coordinate-forecast CLI |
| Licensing | No open-source license is currently granted; see [`NOTICE.md`](NOTICE.md) |

## What changed in v0.2?

### 1. The old coordinate target is no longer a market-signal headline

For the default 10-week momentum period:

```text
rs_momentum_raw(t+h)
    = rs_ratio_raw(t+h) / rs_ratio_raw(t+h-10) - 1
```

For the originally reported horizons `h = 1, 2, 4, 8`, the denominator is already in the past at decision time. Short-horizon RS-Ratio targets also contain heavily overlapping return windows.

This is **not conventional look-ahead leakage**. Every feature can still be causal. The problem is that a model can reconstruct part of the target from information already embedded in its lagged features.

The new synthetic null makes that visible.

### 2. Preprocessing language is now more precise

The previous README said the reconstruction used "fold-local preprocessing." That was too broad.

- `StandardScaler` for the model is fitted inside the training data for each fit.
- RRG-style rolling coordinate normalization is causal at each timestamp, but it is calculated on the chronological series before the model split.

That second step is not future-looking, but it is also not fold-local. The distinction now appears explicitly in the audit documentation.

### 3. The old final holdout is retired for confirmatory reuse

The old 52-week holdout remains an immutable historical record. It is **not** a fresh test for the redesigned target or any new model after this post-publication audit.

The old report described 572 scored rows per horizon as `52 weeks × 11 sectors`. Those are not 572 independent observations: sectors share the same benchmark, cross-sectional returns are correlated, and multi-week targets overlap.

Future claims need dependence-aware uncertainty, such as week-level clustering or moving/block bootstrap intervals with blocks at least as long as the forecast horizon.

### 4. The first strategy result is still negative, but not over-interpreted

The first pre-declared strategy used the 1-week coordinate forecast to rank sectors, held the top three, rebalanced weekly, and charged 10 bps per dollar traded.

It underperformed persistence top-3, equal-weight sectors, and SPY on the tested path. The correct conclusion is:

> **The strategy hypothesis was not supported by that backtest.**

The experiment did not include a formal power/significance analysis, so it should not be strengthened into "there is no alpha."

## The redesigned research question

v0.2 changes the primary target to something that is entirely in the future:

```text
future relative return
    = log(asset[t+h] / asset[t])
    - log(benchmark[t+h] / benchmark[t])
```

RRG-style rotation history remains a candidate feature set. The target asks whether the sector actually outperforms the benchmark **after** the decision timestamp.

The next defensible question is:

> Does recent sector-rotation history contain information about genuinely future relative performance beyond base rates, mechanical indicator structure, and sampling noise?

The first model for that question is deliberately regularized and simple: `StandardScaler -> LogisticRegression(L2)`. More complex models should only be considered if simpler baselines demonstrate a stable out-of-sample signal under the same protocol.

The new target utilities live in:

```text
rebuild/2026/src/rrg_rebuild/future_relative.py
```

## Reproduce the structural null

Install the tested environment from the repository root:

```bash
python -m venv .venv
```

```text
Windows PowerShell: .\.venv\Scripts\Activate.ps1
macOS/Linux:        source .venv/bin/activate
```

```bash
python -m pip install -r rebuild/2026/requirements-lock.txt
python -m pip install -e . --no-deps --no-build-isolation
```

Run the tests and the null benchmark:

```bash
pytest
python rebuild/2026/scripts/run_synthetic_null.py
```

The recorded 12-trial summary is stored at:

```text
rebuild/2026/results/v0.2-null-audit-2026-08-21/synthetic_null_summary.csv
```

## Legacy coordinate forecast CLI

The package still contains the prospective coordinate-forecast workflow developed in v0.1:

```bash
sector-rotation forecast
```

It fetches SPY plus 11 US sector ETFs, records market-data provenance, constructs the transparent RRG-style state history, fits the legacy linear coordinate model, and writes forecast artifacts.

**v0.2 interpretation boundary:** this command is retained as an auditable legacy research workflow. Its predicted RRG-style coordinates are **not** expected returns, calibrated probabilities, buy/sell signals, or validated evidence of market predictability.

A fresh-data archive can also be created without forecasting:

```bash
sector-rotation fetch
```

## Historical audit

The 2025 source audit found genuine implementation work as well as material gaps. Publicly substantiated examples include:

- the web ML dependency was disabled and prediction routes were placeholders;
- a displayed `0.75` model accuracy was hard-coded;
- the historical "LSTM" was an LSTM-like state processor rather than a trained LSTM network;
- the historical feature preprocessing and validation slicing raised leakage/double-counting concerns;
- parts of the historical backtest were placeholders;
- the standalone UI contained random/static outputs.

The first public provenance manifest mistakenly listed three historical source files that were not actually committed. v0.2 corrects that inventory rather than silently claiming they are available.

See:

- [`historical/2025-source-audit.md`](historical/2025-source-audit.md)
- [`historical/PUBLIC-SUBSET.md`](historical/PUBLIC-SUBSET.md)
- [`historical/verify_public_subset.py`](historical/verify_public_subset.py)

Verify the committed historical subset with:

```bash
python historical/verify_public_subset.py
```

The original recovered archive used CRLF line endings while Git stored the public text files with LF endings. The verifier documents and reverses that line-ending transformation before checking the recovered-archive byte counts and SHA-256 values.

## Exact reproducibility boundary

The code, tests, research contracts, null benchmark, and committed result artifacts are auditable and rerunnable.

The **frozen raw market-data archive used for the old v0.1 real-data holdout is not published**. Therefore the exact old 80.6%/74.0% table is not independently reproducible from this public repository alone. v0.2 states that limitation directly instead of using "reproducible" as an unqualified description of every historical result.

Fresh CLI runs preserve their own raw Yahoo payloads, hashes, processed adjusted-close files, manifest, and portable ZIP locally. Provider endpoints and adjustment histories can change independently of this repository.

## Repository map

```text
.
├── README.md
├── CHANGELOG.md
├── CITATION.cff
├── pyproject.toml
├── NOTICE.md
├── historical/
│   ├── 2025-source-audit.md
│   ├── PUBLIC-SUBSET.md
│   ├── verify_public_subset.py
│   └── audited-evidence/
├── rebuild/2026/
│   ├── src/rrg_rebuild/
│   │   ├── null_benchmark.py
│   │   └── future_relative.py
│   ├── tests/
│   ├── docs/
│   │   └── v0.2-null-audit.md
│   ├── scripts/
│   │   └── run_synthetic_null.py
│   └── results/
└── .github/workflows/tests.yml
```

## Research rules from here

1. A causal pipeline still needs a relevant null benchmark.
2. A baseline must have access to the same mechanically available information as the candidate model.
3. Scored rows are not automatically independent observations.
4. Predictive discrimination, calibration, economic value, and statistical significance are separate questions.
5. A failed strategy test means "not supported by this test," not "proved impossible."
6. The old final holdout is retired for new confirmatory claims.
7. Any future headline result should be evaluated on a genuinely prospective window.
8. Negative results and post-publication corrections remain visible rather than being tuned or rewritten away.

## Rights and naming

The software is technically installable and executable, but this repository does **not** currently grant an open-source license. See [`NOTICE.md`](NOTICE.md) before reuse, redistribution, or incorporation into another project.

References to HKSI, Relative Rotation Graph (RRG), and JdK terminology are descriptive historical or methodological references only. This independent repository is not affiliated with or endorsed by HKSI or RRG Research.

## Limits

This project is a research artifact. It is not:

- investment advice;
- a production trading system;
- an official HKSI project or HKSI-endorsed project;
- an official RRG Research product or RRG Research-endorsed project;
- a claim of proprietary JdK equivalence;
- evidence that the legacy 80.6% coordinate result represents market predictability;
- evidence that the first strategy has no possible alpha;
- a guarantee that fresh provider data will reproduce historical adjusted-close values.
