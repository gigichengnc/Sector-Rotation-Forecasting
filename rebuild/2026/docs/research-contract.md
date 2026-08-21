# 2026 research contract

This reconstruction separates four questions:

1. Can a transparent RRG-style coordinate calculation be reproduced from versioned market data?
2. Does a simple trainable model improve future coordinate prediction over current-state persistence?
3. Does the selected model retain that advantage on a frozen untouched final holdout?
4. Does improved coordinate prediction translate into investment performance under a separately declared strategy mapping?

## Frozen predictive specification

- benchmark: SPY
- sector universe: XLB, XLC, XLE, XLF, XLI, XLK, XLP, XLRE, XLU, XLV, XLY
- data frequency: completed weekly observations
- price convention: adjusted close
- RRG-style parameters: ratio period 10, momentum period 10, normalization period 100, normalization scale 10
- model lookback: 20 weekly coordinate observations
- horizons: 1, 2, 4, 8 weekly observations
- primary target: future `(RS-Ratio, RS-Momentum)` coordinate pair
- derived target: quadrant
- baseline: persistence
- selected trainable model: `StandardScaler -> LinearRegression`
- preprocessing: fit inside training data only
- evaluation: chronological, purged, non-overlapping validation
- final holdout: one-time evaluation only; no reuse for model selection

The coordinate implementation is described as **RRG-style**. It does not claim equivalence to a proprietary JdK implementation.
