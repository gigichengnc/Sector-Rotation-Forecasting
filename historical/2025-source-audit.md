# 2025 Source Audit — Static Review

## Scope

This is a static audit of the private recovered Year 2 Autumn source snapshot. The public repository exposes only the curated evidence files required to substantiate the findings below. It is intended to answer a narrow question: **what did the 2025 code actually implement, and where did the competition/presentation framing exceed the implementation?**

The recovered source-only snapshot contains 137 files (~1.94 MiB), including 61 Rust files and the four-crate workspace `rrg-data`, `rrg-calc`, `rrg-ml`, and `rrg-web`. A basic secret scan found no obvious API keys, access tokens, passwords, or private keys.

### Current verification limitation

The present audit environment does not have the Rust toolchain (`cargo`) installed, so a clean compile/test run has **not yet been executed**. Findings below are therefore based on source inspection, not a successful build. Reproducibility remains an open gate.

## High-level status

| Area | Initial finding | Status |
|---|---|---|
| Yahoo Finance data fetching | Concrete HTTP data fetcher and response parsing exist | Substantially implemented |
| 11 SPDR sector universe | Encoded and tested in data crate | Implemented |
| RRG calculation | Timestamp alignment, relative performance, momentum, rolling normalization, quadrant assignment | Substantially implemented, methodology needs validation |
| Web/API layer | Axum server, RRG and alert routes exist | Substantially implemented |
| Standalone UI | Generates RRG coordinates and confidence text using random/static values | Demonstration only |
| ML crate | Feature engineering, linear model, LSTM-like state processor, scenario/backtest structures, tests | Partial / experimental |
| ML integration into web app | `rrg-ml` dependency disabled; ML endpoints explicitly labelled placeholders | Not integrated |
| Model accuracy/confidence | API contains hard-coded `0.75`; UI contains static confidence percentages | Not validated |
| Walk-forward validation | Infrastructure exists, but slicing logic and preprocessing create methodological issues | Partial / unreliable as written |
| Backtesting | Engine structure exists, but core historical RRG step and several statistics are placeholders | Not yet a defensible backtest |

## 1. The RRG/data side is more real than the presentation-only interpretation suggested

The data crate contains a concrete Yahoo Finance chart endpoint (`query1.finance.yahoo.com/v8/finance/chart`), HTTP requests, rate limiting, JSON parsing, adjusted-close handling, and validation. The source also explicitly tests an 11-sector SPDR universe.

The RRG calculator aligns ETF and benchmark timestamps and computes a relative-performance series, a momentum series, rolling normalization around 100, and quadrant assignment. This is meaningful implementation rather than only a mock dashboard.

### Methodology caveat

The code describes its ratio/momentum/normalization as “standard JdK”, but it implements its own formulas:

- relative performance = ETF n-period return / benchmark n-period return;
- momentum = rate of change of that relative-performance series;
- normalization = rolling z-score mapped around 100.

The 2026 rebuild should therefore **verify the mathematical definition against a documented reference** instead of assuming equivalence to the proprietary JdK RRG formulation.

## 2. The web application did not have the ML crate connected

`crates/web/Cargo.toml` contains:

```toml
# rrg-ml = { path = "../ml" }  # Temporarily disabled due to dependency conflicts
```

`crates/web/src/api.rs` separately labels the ML prediction and scenario routes as **placeholder implementations**. The prediction handler returns an empty prediction vector and hard-codes `model_accuracy: 0.75`. Model status also reports `accuracy: 0.75` with a fixed historical date.

**Interpretation:** the 2025 application had an ML research crate, but the production-facing web layer did not provide evidence of a live trained predictor behind the advertised AI panel.

## 3. The “LSTM” was an experimental LSTM-like state processor, not a trained LSTM network

`LSTMPredictor` stores LSTM-style cell and hidden states, but its own comments call the gate calculation **simplified** and state that a full implementation would use proper matrix operations. Gate values are deterministic functions of current inputs and prior state; there are no learned LSTM weight matrices or training routine for those gates.

The same file stores an `Option<FittedLinearRegression<f64>>` and separately implements `LinearPredictor`, whose actual `train()` method fits `linfa_linear::LinearRegression`.

A more accurate historical description is therefore:

> experimental LSTM-like sequential state processing + a trainable linear-regression forecasting baseline

rather than “a fully trained LSTM neural network”.

## 4. The apparent 65–75% accuracy cannot be treated as an experimental result

The web API hard-codes `0.75` and returns no predictions in the placeholder handler. The standalone frontend also contains static confidence values such as 85%, 72%, and 68%. These values are suitable for UI demonstration but are not evidence of model calibration or out-of-sample accuracy.

Those historical values are treated **as placeholders**. The 2026 rebuild replaces them with metrics generated from reproducible evaluation.

## 5. Walk-forward validation exists, but the implementation has important flaws

This is one of the strongest parts conceptually: `TrainingPipeline` defines training windows, validation windows, a gap, fold generation, and an explicit `verify_no_leakage()` check.

However, the actual fold loop does not respect all generated boundaries:

- it ignores `train_start` and trains on `features[..train_end]`;
- it ignores `val_end` and validates on `features[val_start..]`;
- stored fold metadata likewise records validation through the end of the full feature set.

This means later folds use expanding training windows and **overlapping validation tails**, even though the configuration appears to describe fixed windows. Aggregating those validation predictions can double-count later observations.

## 6. Preprocessing leaks future information

`FeatureEngineer::create_sequences_from_single_dataset()` constructs the entire feature history and then calls `normalize_features()` **before** the walk-forward split. Z-score, min-max, and robust normalization all estimate their statistics from the complete feature vector.

Because future validation observations contribute to those statistics, the preprocessing pipeline leaks future distribution information into training features.

**Required rebuild:** fit every scaler/transformer inside each training fold and apply that fitted transform unchanged to its validation/test window.

## 7. The standalone frontend is explicitly simulated

`app-standalone/app.js` generates RRG coordinates using `Math.random()`, displays static “AI insight” confidence percentages, and generates a random data-point count. This should be preserved as a demo artifact, but never presented as live model output.

The full web frontend also contains demo/fallback values and simulated refresh logic, so the rebuild should have an unmistakable `DEMO` versus `LIVE` data mode.

## 8. External macro features are simulated in the surviving data layer

`crates/data/src/external_data.rs` labels VIX and interest-rate fetches as simulated and uses a pseudo-random generator. The feature-engineering layer can accept these features, but this snapshot does not establish a real historical macro-data ingestion pipeline.

## 9. The backtest structure is not yet a valid historical backtest

`crates/ml/src/backtesting.rs` contains portfolio snapshots, trades, returns and strategy-comparison structures, but critical calculations are placeholders:

- historical RRG positions are set to `[100.0, 100.0]` rather than calculated from historical windows;
- beta is hard-coded to 1.0;
- profit factor is hard-coded to 1.0;
- winning sells are effectively counted as wins without per-trade P&L tracking.

Therefore no performance metric from this path should be presented as evidence of a profitable strategy until the engine is rebuilt around point-in-time data and real trade accounting.

## 10. Documentation/API drift exists

The historical root README advertises endpoints such as `/api/rrg/{symbol}` and `/api/predictions/{symbol}`, while the source router uses versioned paths such as `/api/v1/rrg/...` and `/api/v1/ml/predict/:symbol`. This is another reason to treat the old README as an artifact rather than current documentation.

## Claim → implementation → rebuild map

| 2025 framing | Surviving implementation | 2026 requirement |
|---|---|---|
| “AI-powered RRG predictions” | ML research crate exists; web ML dependency disabled | end-to-end tested model service |
| “LSTM prediction” | simplified LSTM-like gates; trainable model is linear regression | baseline-first modelling, then genuine trained sequence model only if justified |
| “65–75% accuracy” | hard-coded/static 0.75 and UI confidences | measured out-of-sample metrics with sample counts and uncertainty |
| “walk-forward validation” | fold/gap infrastructure exists, but slicing overlaps validation tails | strict time-aware folds with fold-local preprocessing |
| “VIX / rates features” | simulated external values in surviving data code | dated, point-in-time historical macro series or remove feature |
| “backtesting” | framework exists; core RRG/trade metrics partly placeholders | point-in-time RRG, realistic execution/accounting, benchmark comparison |
| “real-time/live terminal” | real-data API path plus extensive demo/random fallbacks | explicit LIVE/DEMO modes and provenance shown in UI |

## Rebuild order

1. reproduce the historical build and tests;
2. freeze a tagged historical snapshot;
3. validate the RRG mathematics;
4. define one prediction target and horizon precisely;
5. build naive/persistence and linear/logistic baselines;
6. rebuild feature generation with fold-local preprocessing;
7. implement strict walk-forward evaluation and calibration;
8. rebuild point-in-time backtesting;
9. connect one validated predictor to the API;
10. only then consider a genuine LSTM/GRU/temporal model and compare it against baselines.

## What this audit does **not** conclude yet

This review does not yet say whether sector rotation is predictably exploitable, whether the project can compile unchanged, or whether any model beats a simple baseline. Those are empirical questions for the reconstruction.


## Public evidence scope

The full 137-file recovered archive remains private. See [`PUBLIC-SUBSET.md`](PUBLIC-SUBSET.md) for the exact historical files included in this sanitized public release and the audit claims each file supports.
