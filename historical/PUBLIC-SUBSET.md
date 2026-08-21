# Public historical evidence subset

The private recovered 2025 source-only archive contains **137 files**. This public repository intentionally exposes only a small curated evidence subset.

## v0.2 provenance correction

The first public manifest incorrectly listed 15 evidence files. Three of those paths were never actually committed to the public repository:

- `crates/data/src/data_fetcher.rs`
- `crates/data/src/external_data.rs`
- `crates/data/src/etf_data.rs`

They are therefore removed from the public subset inventory below. No additional private historical source is being published merely to repair the manifest while the competition/IP boundary remains unresolved.

The 12 files that are actually public were copied from text files whose recovered-archive bytes used CRLF line endings. Git stored the public checkout with LF line endings. Their original archive SHA-256 values therefore do not match the raw Git checkout bytes directly, even though the text content is otherwise equivalent.

To make that transformation explicit and verifiable, this repository now includes:

```bash
python historical/verify_public_subset.py
```

The verifier converts the public text checkout to the recovered archive's CRLF convention and then checks the original byte count and SHA-256. CI runs this verifier. This is a **content-equivalence check under a documented line-ending transformation**, not a claim that the raw Git blob bytes equal the recovered archive bytes.

## Verified public evidence files

| Public evidence file | Recovered archive bytes | Recovered archive SHA-256 | Main audit purpose |
|---|---:|---|---|
| `README.md` | 6381 | `46fb1e63dac583d7cbaa4a32fd94059dcb7877df552f11eec82dbcbf7613046b` | Historical presentation / API claims |
| `config.toml` | 1718 | `4356831f3531caf312c423b9b821dd9f030f8900fe24b21da0822d848ea458cf` | Historical configuration and model settings |
| `crates/rrg-calc/src/calculator.rs` | 22341 | `afebb1e98531495ed833a281437ffd55e8d0cfa2b0a60fb5595b9a608dba251d` | Historical RRG-style calculation |
| `crates/rrg-calc/src/quadrant.rs` | 13243 | `bb0e5df0ae335a14f77dbf3b3981c177265271df77a0b359c29babf0f9a7dc87` | Quadrant assignment logic |
| `crates/ml/src/lstm_predictor.rs` | 21468 | `8a01d6cb1d4c4e2648ab0253ff987fa35b7c2cead483057a7c881d8289807f9c` | LSTM-like state processor and linear baseline |
| `crates/ml/src/feature_engineering.rs` | 40171 | `56ea893b3331df6d0350b3e7e3f726c9ce52056c4420a28919b3b03b8c0bb6ed` | Feature normalization / preprocessing audit |
| `crates/ml/src/training_pipeline.rs` | 17526 | `c547877e9115c8c0a598a24b5cb7ea284e9e81313a0bd08b09bd6efcaaeb876d` | Walk-forward split / validation audit |
| `crates/ml/src/prediction_engine.rs` | 26517 | `536b6d7a3528799bf8c81dd94725a742f2cf594ddda193d8c33d0b03ce0b7fec` | Historical prediction path and horizon semantics |
| `crates/ml/src/backtesting.rs` | 25464 | `023863d7727b691517872a6fd519f11af5fb6e381421eb1ade1be8874adccd4c` | Historical backtest placeholders / accounting audit |
| `crates/web/Cargo.toml` | 1043 | `8c6ebc8c8a2329888e1e2d2929c2ee16448eccc6e2c05fa1d68f29cac45baf28` | ML dependency disabled in web application |
| `crates/web/src/api.rs` | 43865 | `a5661d8339e566102721c0ef7203d0df88c295143dea0b79d7f42980e40f37e9` | Placeholder ML API and hard-coded accuracy |
| `app-standalone/app.js` | 19602 | `5325b05cad5599691e08e20608b7ac5593b19b9a34f792a9cc37130d568a91c1` | Simulated/random standalone UI outputs |

## Claims no longer publicly substantiated by this subset

The private recovered archive/audit record also referred to a concrete Yahoo data fetcher, simulated external VIX/rate features, and sector-universe data structures. Because the three source files listed above are not public, those specific implementation claims should **not** be treated as independently verifiable from this repository's public evidence subset.

The detailed `2025-source-audit.md` is retained as a record of the static audit performed against the recovered archive, but public readers should use this file to distinguish what is and is not substantiated by committed evidence.

The complete private archive remains intentionally excluded. `SOURCE-MANIFEST.md` is a recovered source inventory, not a statement that every listed source file is publicly available.
