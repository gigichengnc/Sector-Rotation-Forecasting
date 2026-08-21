from __future__ import annotations

from collections.abc import Iterable

import pandas as pd


def add_future_coordinate_targets(
    rrg: pd.DataFrame,
    horizons: Iterable[int] = (1, 2, 4, 8),
) -> pd.DataFrame:
    """Add future coordinate targets without leaking them into current features.

    A horizon is expressed in *observations*. In the planned weekly dataset,
    horizon=4 therefore means four weekly observations, not four calendar days.
    """
    out = rrg.copy()
    for horizon in horizons:
        if horizon <= 0:
            raise ValueError("horizons must be > 0")
        out[f"target_rs_ratio_h{horizon}"] = out["rs_ratio"].shift(-horizon)
        out[f"target_rs_momentum_h{horizon}"] = out["rs_momentum"].shift(-horizon)
        out[f"target_quadrant_h{horizon}"] = out["quadrant"].shift(-horizon)
    return out


def add_persistence_baseline(
    frame: pd.DataFrame,
    horizons: Iterable[int] = (1, 2, 4, 8),
) -> pd.DataFrame:
    """Predict that future RRG coordinates equal the current coordinates."""
    out = frame.copy()
    for horizon in horizons:
        out[f"pred_rs_ratio_persistence_h{horizon}"] = out["rs_ratio"]
        out[f"pred_rs_momentum_persistence_h{horizon}"] = out["rs_momentum"]
    return out
