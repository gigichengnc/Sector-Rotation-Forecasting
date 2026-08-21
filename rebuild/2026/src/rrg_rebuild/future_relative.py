from __future__ import annotations

import numpy as np
import pandas as pd
from sklearn.linear_model import LogisticRegression
from sklearn.pipeline import Pipeline
from sklearn.preprocessing import StandardScaler

from .modeling import feature_columns


def make_future_relative_return_dataset(
    rrg: pd.DataFrame,
    asset_price: pd.Series,
    benchmark_price: pd.Series,
    *,
    horizon: int,
    lookback: int = 20,
) -> pd.DataFrame:
    """Create a target using only price movement after the decision timestamp.

    Target:
        log(asset[t+h] / asset[t]) - log(benchmark[t+h] / benchmark[t])

    Unlike a future rolling RRG coordinate, this target contains no pre-decision
    segment of the future return window.
    """
    if horizon <= 0:
        raise ValueError("horizon must be > 0")
    if lookback <= 0:
        raise ValueError("lookback must be > 0")
    if not rrg.index.is_monotonic_increasing or rrg.index.has_duplicates:
        raise ValueError("rrg index must be unique and chronological")

    index = pd.DatetimeIndex(rrg.index)
    if index.tz is None:
        raise ValueError("rrg index must be timezone-aware")

    asset = asset_price.reindex(index).astype(float)
    benchmark = benchmark_price.reindex(index).astype(float)
    if asset.isna().any() or benchmark.isna().any():
        raise ValueError("price series must cover every RRG timestamp")
    if (asset <= 0).any() or (benchmark <= 0).any():
        raise ValueError("prices must be strictly positive")

    out = pd.DataFrame(index=index)
    out["decision_timestamp"] = index
    out["target_timestamp"] = pd.Series(index, index=index).shift(-horizon)

    for lag in range(lookback):
        out[f"rs_ratio_lag{lag}"] = rrg["rs_ratio"].shift(lag)
        out[f"rs_momentum_lag{lag}"] = rrg["rs_momentum"].shift(lag)

    future_asset = np.log(asset.shift(-horizon) / asset)
    future_benchmark = np.log(benchmark.shift(-horizon) / benchmark)
    out["future_relative_log_return"] = future_asset - future_benchmark
    out["target_outperform"] = (
        out["future_relative_log_return"] > 0.0
    ).astype(float)
    out = out.dropna().copy()
    out["target_outperform"] = out["target_outperform"].astype(int)
    out["decision_timestamp"] = pd.to_datetime(
        out["decision_timestamp"], utc=True
    )
    out["target_timestamp"] = pd.to_datetime(out["target_timestamp"], utc=True)

    if (out["target_timestamp"] <= out["decision_timestamp"]).any():
        raise AssertionError("target timestamp must be after decision timestamp")
    return out


def fit_l2_outperformance_model(
    X_train: pd.DataFrame,
    y_train: pd.Series,
    *,
    c: float = 1.0,
) -> Pipeline:
    """Fit a regularized logistic model for future relative outperformance."""
    if c <= 0:
        raise ValueError("c must be positive")
    if len(X_train) != len(y_train):
        raise ValueError("X_train and y_train length mismatch")
    if y_train.nunique() < 2:
        raise ValueError("training target must contain both classes")

    model = Pipeline(
        steps=[
            ("scale", StandardScaler()),
            (
                "model",
                LogisticRegression(
                    penalty="l2",
                    C=c,
                    max_iter=5000,
                ),
            ),
        ]
    )
    model.fit(X_train, y_train.astype(int))
    return model


def future_relative_feature_columns(frame: pd.DataFrame) -> list[str]:
    """Return the lagged RRG feature columns used by the v0.2 target."""
    return feature_columns(frame)
