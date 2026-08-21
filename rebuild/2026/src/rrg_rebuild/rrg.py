from __future__ import annotations

from dataclasses import dataclass
from typing import Literal

import numpy as np
import pandas as pd

Quadrant = Literal["Leading", "Weakening", "Lagging", "Improving"]


@dataclass(frozen=True)
class RRGConfig:
    """Transparent parameters for the project's RRG-style calculation.

    These defaults reproduce the *structure* of the surviving 2025 calculator:
    an n-period asset return divided by the benchmark n-period return, followed
    by momentum of that relative-return ratio and causal rolling normalization.

    This implementation deliberately does not claim mathematical equivalence to
    the proprietary JdK RS-Ratio / RS-Momentum formulation.
    """

    ratio_period: int = 10
    momentum_period: int = 10
    normalization_period: int = 100
    normalization_scale: float = 10.0

    def __post_init__(self) -> None:
        for name in ("ratio_period", "momentum_period", "normalization_period"):
            if getattr(self, name) <= 0:
                raise ValueError(f"{name} must be > 0")
        if self.normalization_scale <= 0:
            raise ValueError("normalization_scale must be > 0")


def classify_quadrant(rs_ratio: float, rs_momentum: float) -> Quadrant:
    """Classify one normalized coordinate around the 100/100 crosshair."""
    if rs_ratio >= 100.0 and rs_momentum >= 100.0:
        return "Leading"
    if rs_ratio >= 100.0 and rs_momentum < 100.0:
        return "Weakening"
    if rs_ratio < 100.0 and rs_momentum < 100.0:
        return "Lagging"
    return "Improving"


def _causal_rolling_normalize(
    series: pd.Series,
    *,
    window: int,
    scale: float,
) -> pd.Series:
    """Map a series around 100 using only data available at each timestamp.

    The 2025 Rust source used a rolling/expanding mean and population standard
    deviation and emitted 100 when the standard deviation was zero. `ddof=0`
    reproduces that population-standard-deviation convention.
    """
    mean = series.rolling(window=window, min_periods=1).mean()
    std = series.rolling(window=window, min_periods=1).std(ddof=0)
    z = (series - mean) / std.replace(0.0, np.nan)
    normalized = 100.0 + z * scale
    return normalized.fillna(100.0)


def calculate_rrg(
    asset_price: pd.Series,
    benchmark_price: pd.Series,
    config: RRGConfig | None = None,
) -> pd.DataFrame:
    """Calculate a transparent, causal RRG-style coordinate history."""
    cfg = config or RRGConfig()

    frame = pd.concat(
        [asset_price.rename("asset"), benchmark_price.rename("benchmark")],
        axis=1,
        join="inner",
    ).sort_index()
    frame = frame.dropna()

    if frame.empty:
        raise ValueError("asset and benchmark have no overlapping observations")
    if (frame[["asset", "benchmark"]] <= 0).any().any():
        raise ValueError("prices must be strictly positive")
    if frame.index.has_duplicates:
        raise ValueError("timestamps must be unique")

    asset_return_ratio = frame["asset"] / frame["asset"].shift(cfg.ratio_period)
    benchmark_return_ratio = (
        frame["benchmark"] / frame["benchmark"].shift(cfg.ratio_period)
    )
    frame["rs_ratio_raw"] = asset_return_ratio / benchmark_return_ratio
    frame["rs_momentum_raw"] = (
        frame["rs_ratio_raw"]
        / frame["rs_ratio_raw"].shift(cfg.momentum_period)
        - 1.0
    )

    frame = frame.dropna(subset=["rs_ratio_raw", "rs_momentum_raw"]).copy()
    if frame.empty:
        return frame.assign(
            rs_ratio=pd.Series(dtype=float),
            rs_momentum=pd.Series(dtype=float),
            quadrant=pd.Series(dtype="object"),
        )

    frame["rs_ratio"] = _causal_rolling_normalize(
        frame["rs_ratio_raw"],
        window=cfg.normalization_period,
        scale=cfg.normalization_scale,
    )
    frame["rs_momentum"] = _causal_rolling_normalize(
        frame["rs_momentum_raw"],
        window=cfg.normalization_period,
        scale=cfg.normalization_scale,
    )
    frame["quadrant"] = [
        classify_quadrant(ratio, momentum)
        for ratio, momentum in zip(frame["rs_ratio"], frame["rs_momentum"], strict=True)
    ]
    return frame
