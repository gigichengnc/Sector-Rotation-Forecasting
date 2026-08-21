# Public historical evidence subset

The private recovered 2025 source-only archive contains **137 files**. This sanitized public release does **not** publish the complete historical archive.

Only the files below are included because they directly support material findings in the retrospective audit. Their SHA-256 values are calculated from the recovered archive bytes.

| Public evidence file | Bytes | SHA-256 | Main audit purpose |
|---|---:|---|---|
| `historical-root/README.md` | 6381 | `46fb1e63dac583d7cbaa4a32fd94059dcb7877df552f11eec82dbcbf7613046b` | Historical presentation / API claims |
| `config.toml` | 1718 | `4356831f3531caf312c423b9b821dd9f030f8900fe24b21da0822d848ea458cf` | Historical configuration and model settings |
| `crates/data/src/data_fetcher.rs` | 8318 | `c476e50be3ee822f93e66c6337b0dab740756690d6f64c361716d48e35949243` | Yahoo Finance HTTP path, adjusted-close handling |
| `crates/data/src/external_data.rs` | 16523 | `d9987ffd7c96ffb3106d21569da7a61f8c35924e7c168c36c1f6b0795821ed28` | Simulated VIX / interest-rate features |
| `crates/data/src/etf_data.rs` | 5091 | `62281034fda37f5fe83fb5119da31cd4e3db552dae65ffed72a2bedfa49fb2d5` | Sector ETF data structures / universe evidence |
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

The complete private archive is intentionally excluded to reduce disclosure risk and avoid publishing generated, duplicated, obsolete, or nonessential artifacts such as `.kiro/` specifications and `OLD Assets/`.

The complete file-name/hash inventory remains available in [`SOURCE-MANIFEST.md`](SOURCE-MANIFEST.md).
