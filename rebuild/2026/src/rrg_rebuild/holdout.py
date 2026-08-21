from __future__ import annotations

from dataclasses import dataclass
from typing import Iterable

import numpy as np
import pandas as pd
from sklearn.metrics import f1_score

from .modeling import feature_columns, fit_linear_coordinate_model
from .rrg import classify_quadrant


FINAL_HOLDOUT_PROTOCOL_ID = "rrg-final-holdout-v1-2026-08-20"
FINAL_HOLDOUT_AUTHORIZATION_TOKEN = "OPEN_FINAL_HOLDOUT_2026_08_20"


@dataclass(frozen=True)
class FinalHoldoutProtocol:
    """Frozen one-time final holdout rules.

    The holdout is defined by *target timestamps*, not decision timestamps.
    This gives every forecast horizon the same 52 target weeks.

    A separate fixed model is fitted for each symbol/horizon. The model is
    frozen at the first decision timestamp required for that horizon and is not
    refit while the holdout is being scored. No holdout target can therefore
    enter training for a later holdout prediction.
    """

    target_start: pd.Timestamp
    target_end: pd.Timestamp
    horizons: tuple[int, ...] = (1, 2, 4, 8)
    lookback: int = 20

    def __post_init__(self) -> None:
        start = _utc(self.target_start)
        end = _utc(self.target_end)
        object.__setattr__(self, "target_start", start)
        object.__setattr__(self, "target_end", end)

        if start > end:
            raise ValueError("target_start must not be after target_end")
        if self.lookback <= 0:
            raise ValueError("lookback must be > 0")
        if not self.horizons or any(h <= 0 for h in self.horizons):
            raise ValueError("horizons must contain positive integers")
        if len(set(self.horizons)) != len(self.horizons):
            raise ValueError("horizons must be unique")


def frozen_protocol() -> FinalHoldoutProtocol:
    return FinalHoldoutProtocol(
        target_start=pd.Timestamp("2025-08-22T23:59:59.999999999Z"),
        target_end=pd.Timestamp("2026-08-14T23:59:59.999999999Z"),
        horizons=(1, 2, 4, 8),
        lookback=20,
    )


def _utc(value: pd.Timestamp | str) -> pd.Timestamp:
    ts = pd.Timestamp(value)
    if ts.tzinfo is None:
        raise ValueError("timestamps must be timezone-aware")
    return ts.tz_convert("UTC")


def require_final_holdout_authorization(
    *,
    open_final_holdout: bool,
    authorization_token: str | None,
) -> None:
    """Refuse to proceed unless both deliberate gates are supplied.

    This function must be called before market-data contents are opened.
    """
    if not open_final_holdout:
        raise PermissionError(
            "final holdout is closed; pass --open-final-holdout only after explicit approval"
        )
    if authorization_token != FINAL_HOLDOUT_AUTHORIZATION_TOKEN:
        raise PermissionError("final holdout authorization token is missing or incorrect")


def make_timestamped_supervised_dataset(
    rrg: pd.DataFrame,
    *,
    horizon: int,
    lookback: int,
) -> pd.DataFrame:
    """Create causal lag features plus explicit decision/target timestamps."""
    if horizon <= 0:
        raise ValueError("horizon must be > 0")
    if lookback <= 0:
        raise ValueError("lookback must be > 0")
    required = {"rs_ratio", "rs_momentum"}
    missing = required.difference(rrg.columns)
    if missing:
        raise ValueError(f"missing required columns: {sorted(missing)}")
    if not rrg.index.is_monotonic_increasing or rrg.index.has_duplicates:
        raise ValueError("rrg index must be unique and chronological")

    index = pd.DatetimeIndex(rrg.index)
    if index.tz is None:
        raise ValueError("rrg index must be timezone-aware")

    out = pd.DataFrame(index=index)
    out["decision_timestamp"] = index

    timestamp_series = pd.Series(index, index=index)
    out["target_timestamp"] = timestamp_series.shift(-horizon)

    for lag in range(lookback):
        out[f"rs_ratio_lag{lag}"] = rrg["rs_ratio"].shift(lag)
        out[f"rs_momentum_lag{lag}"] = rrg["rs_momentum"].shift(lag)

    out["target_rs_ratio"] = rrg["rs_ratio"].shift(-horizon)
    out["target_rs_momentum"] = rrg["rs_momentum"].shift(-horizon)
    out = out.dropna().copy()

    if out.empty:
        raise ValueError("not enough observations for requested lookback/horizon")

    out["decision_timestamp"] = pd.to_datetime(out["decision_timestamp"], utc=True)
    out["target_timestamp"] = pd.to_datetime(out["target_timestamp"], utc=True)

    if (out["target_timestamp"] <= out["decision_timestamp"]).any():
        raise AssertionError("target timestamp must be after decision timestamp")
    return out


@dataclass(frozen=True)
class HoldoutSplit:
    train: pd.DataFrame
    holdout: pd.DataFrame
    first_holdout_decision: pd.Timestamp


def split_fixed_final_holdout(
    supervised: pd.DataFrame,
    *,
    protocol: FinalHoldoutProtocol,
) -> HoldoutSplit:
    """Split a supervised frame without allowing any holdout target into training.

    Holdout membership is defined by target timestamp. For each horizon, the
    model is fitted once using only examples whose target was already strictly
    before the first holdout decision timestamp for that horizon.
    """
    required = {"decision_timestamp", "target_timestamp"}
    missing = required.difference(supervised.columns)
    if missing:
        raise ValueError(f"missing timestamp columns: {sorted(missing)}")

    target_ts = pd.to_datetime(supervised["target_timestamp"], utc=True)
    decision_ts = pd.to_datetime(supervised["decision_timestamp"], utc=True)

    holdout_mask = target_ts.between(
        protocol.target_start,
        protocol.target_end,
        inclusive="both",
    )
    holdout = supervised.loc[holdout_mask].copy()
    if holdout.empty:
        raise ValueError("no rows fall inside the frozen final holdout target window")

    first_decision = pd.to_datetime(
        holdout["decision_timestamp"], utc=True
    ).min()

    # Strictly earlier than the first decision timestamp. This is deliberately
    # conservative: no label from the first holdout decision week can be used.
    train_mask = target_ts < first_decision
    train = supervised.loc[train_mask].copy()
    if train.empty:
        raise ValueError("no eligible pre-holdout training rows")

    if pd.to_datetime(train["target_timestamp"], utc=True).max() >= first_decision:
        raise AssertionError("training target reaches the holdout decision boundary")
    if pd.to_datetime(holdout["target_timestamp"], utc=True).min() < protocol.target_start:
        raise AssertionError("holdout starts before frozen target window")
    if pd.to_datetime(holdout["target_timestamp"], utc=True).max() > protocol.target_end:
        raise AssertionError("holdout ends after frozen target window")

    return HoldoutSplit(
        train=train,
        holdout=holdout,
        first_holdout_decision=first_decision,
    )


def _quadrants(ratio: np.ndarray, momentum: np.ndarray) -> list[str]:
    return [
        classify_quadrant(float(r), float(m))
        for r, m in zip(ratio, momentum, strict=True)
    ]


def evaluate_final_holdout_for_series(
    rrg: pd.DataFrame,
    *,
    horizon: int,
    protocol: FinalHoldoutProtocol,
    open_final_holdout: bool,
    authorization_token: str | None,
) -> tuple[pd.DataFrame, pd.DataFrame]:
    """Run the frozen persistence-vs-linear final holdout for one RRG series.

    Authorization is checked before target values are materialized.
    """
    require_final_holdout_authorization(
        open_final_holdout=open_final_holdout,
        authorization_token=authorization_token,
    )

    if horizon not in protocol.horizons:
        raise ValueError(f"horizon {horizon} is not frozen in the protocol")

    frame = make_timestamped_supervised_dataset(
        rrg,
        horizon=horizon,
        lookback=protocol.lookback,
    )
    split = split_fixed_final_holdout(frame, protocol=protocol)

    features = feature_columns(split.train)
    targets = ["target_rs_ratio", "target_rs_momentum"]

    model = fit_linear_coordinate_model(
        split.train[features],
        split.train[targets],
    )
    linear = np.asarray(model.predict(split.holdout[features]), dtype=float)
    persistence = split.holdout[
        ["rs_ratio_lag0", "rs_momentum_lag0"]
    ].to_numpy(float)
    actual = split.holdout[targets].to_numpy(float)

    rows = []
    for model_name, pred in (
        ("persistence", persistence),
        ("linear", linear),
    ):
        block = pd.DataFrame(
            {
                "decision_timestamp": pd.to_datetime(
                    split.holdout["decision_timestamp"], utc=True
                ).to_numpy(),
                "target_timestamp": pd.to_datetime(
                    split.holdout["target_timestamp"], utc=True
                ).to_numpy(),
                "horizon": horizon,
                "model": model_name,
                "actual_rs_ratio": actual[:, 0],
                "actual_rs_momentum": actual[:, 1],
                "predicted_rs_ratio": pred[:, 0],
                "predicted_rs_momentum": pred[:, 1],
            }
        )
        block["actual_quadrant"] = _quadrants(
            block["actual_rs_ratio"].to_numpy(),
            block["actual_rs_momentum"].to_numpy(),
        )
        block["predicted_quadrant"] = _quadrants(
            block["predicted_rs_ratio"].to_numpy(),
            block["predicted_rs_momentum"].to_numpy(),
        )
        rows.append(block)

    predictions = pd.concat(rows, ignore_index=True)
    summary_rows = []
    for model_name, group in predictions.groupby("model", sort=False):
        ratio_error = group["predicted_rs_ratio"] - group["actual_rs_ratio"]
        momentum_error = (
            group["predicted_rs_momentum"] - group["actual_rs_momentum"]
        )
        distance = np.sqrt(
            ratio_error.to_numpy() ** 2 + momentum_error.to_numpy() ** 2
        )
        summary_rows.append(
            {
                "model": model_name,
                "horizon": horizon,
                "n": len(group),
                "mae_rs_ratio": float(ratio_error.abs().mean()),
                "mae_rs_momentum": float(momentum_error.abs().mean()),
                "rmse_rs_ratio": float(
                    np.sqrt(np.mean(ratio_error.to_numpy() ** 2))
                ),
                "rmse_rs_momentum": float(
                    np.sqrt(np.mean(momentum_error.to_numpy() ** 2))
                ),
                "mean_coordinate_distance": float(np.mean(distance)),
                "quadrant_accuracy": float(
                    (
                        group["actual_quadrant"]
                        == group["predicted_quadrant"]
                    ).mean()
                ),
                "macro_f1": float(
                    f1_score(
                        group["actual_quadrant"],
                        group["predicted_quadrant"],
                        average="macro",
                        zero_division=0,
                    )
                ),
                "first_holdout_decision": split.first_holdout_decision,
                "holdout_target_start": protocol.target_start,
                "holdout_target_end": protocol.target_end,
            }
        )

    summary = pd.DataFrame.from_records(summary_rows)
    return predictions, summary
