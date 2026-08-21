from __future__ import annotations

from dataclasses import dataclass
from collections.abc import Sequence

import numpy as np
import pandas as pd

from .modeling import (
    feature_columns,
    fit_linear_coordinate_model,
    make_coordinate_supervised_dataset,
)
from .rrg import classify_quadrant


@dataclass(frozen=True)
class DeploymentConfig:
    """Frozen deployment settings derived from the validated 2026 baseline."""

    horizons: tuple[int, ...] = (1, 2, 4, 8)
    lookback: int = 20

    def __post_init__(self) -> None:
        if self.lookback <= 0:
            raise ValueError("lookback must be > 0")
        if not self.horizons or any(h <= 0 for h in self.horizons):
            raise ValueError("horizons must contain positive integers")
        if len(set(self.horizons)) != len(self.horizons):
            raise ValueError("horizons must be unique")


def _validated_rrg(rrg: pd.DataFrame) -> pd.DataFrame:
    required = {"rs_ratio", "rs_momentum"}
    missing = required.difference(rrg.columns)
    if missing:
        raise ValueError(f"missing required columns: {sorted(missing)}")
    if rrg.empty:
        raise ValueError("rrg history cannot be empty")
    if rrg.index.has_duplicates or not rrg.index.is_monotonic_increasing:
        raise ValueError("rrg index must be unique and chronological")
    index = pd.DatetimeIndex(rrg.index)
    if index.tz is None:
        raise ValueError("rrg index must be timezone-aware")
    if rrg[["rs_ratio", "rs_momentum"]].isna().any().any():
        raise ValueError("rrg coordinates cannot contain missing values")
    out = rrg.copy()
    out.index = index.tz_convert("UTC")
    return out


def make_latest_feature_row(rrg: pd.DataFrame, *, lookback: int = 20) -> pd.DataFrame:
    """Build one causal feature row for the latest observed RRG timestamp."""
    if lookback <= 0:
        raise ValueError("lookback must be > 0")
    history = _validated_rrg(rrg)
    if len(history) < lookback:
        raise ValueError("not enough observations for requested lookback")

    values: dict[str, float] = {}
    for lag in range(lookback):
        row = history.iloc[-1 - lag]
        values[f"rs_ratio_lag{lag}"] = float(row["rs_ratio"])
        values[f"rs_momentum_lag{lag}"] = float(row["rs_momentum"])
    return pd.DataFrame([values], index=pd.DatetimeIndex([history.index[-1]]))


def forecast_latest_for_series(
    rrg: pd.DataFrame,
    *,
    horizon: int,
    lookback: int = 20,
) -> dict[str, object]:
    """Fit on every label currently known, then forecast from the latest state.

    This is a deployment operation, not an out-of-sample evaluation. The
    supervised training frame naturally stops `horizon` observations before the
    latest observation because later labels are not yet known.
    """
    if horizon <= 0:
        raise ValueError("horizon must be > 0")
    history = _validated_rrg(rrg)
    train = make_coordinate_supervised_dataset(
        history,
        horizon=horizon,
        lookback=lookback,
    )
    features = feature_columns(train)
    latest = make_latest_feature_row(history, lookback=lookback)
    latest = latest[features]

    targets = ["target_rs_ratio", "target_rs_momentum"]
    model = fit_linear_coordinate_model(train[features], train[targets])
    predicted = np.asarray(model.predict(latest), dtype=float)
    if predicted.shape != (1, 2):
        raise AssertionError("deployment model must return one coordinate pair")

    decision_timestamp = history.index[-1]
    target_timestamp = decision_timestamp + pd.Timedelta(weeks=horizon)
    current_ratio = float(history.iloc[-1]["rs_ratio"])
    current_momentum = float(history.iloc[-1]["rs_momentum"])
    predicted_ratio = float(predicted[0, 0])
    predicted_momentum = float(predicted[0, 1])

    # The final supervised row must have a target equal to the latest observed
    # RRG timestamp. This proves the deployment fit uses no unavailable label.
    last_train_decision = pd.Timestamp(train.index[-1])
    last_decision_position = history.index.get_loc(last_train_decision)
    last_train_target = history.index[last_decision_position + horizon]
    if last_train_target != decision_timestamp:
        raise AssertionError("deployment training target is not the latest known observation")

    return {
        "decision_timestamp": decision_timestamp,
        "target_timestamp": target_timestamp,
        "horizon": int(horizon),
        "lookback": int(lookback),
        "training_rows": int(len(train)),
        "training_last_decision": last_train_decision,
        "training_last_target": last_train_target,
        "current_rs_ratio": current_ratio,
        "current_rs_momentum": current_momentum,
        "current_quadrant": classify_quadrant(current_ratio, current_momentum),
        "predicted_rs_ratio": predicted_ratio,
        "predicted_rs_momentum": predicted_momentum,
        "predicted_quadrant": classify_quadrant(predicted_ratio, predicted_momentum),
    }


def forecast_panel_latest(
    panel: pd.DataFrame,
    *,
    sector_symbols: Sequence[str],
    config: DeploymentConfig | None = None,
) -> pd.DataFrame:
    """Generate one prospective forecast for each sector/horizon."""
    cfg = config or DeploymentConfig()
    required = {"symbol", "timestamp", "rs_ratio", "rs_momentum"}
    missing = required.difference(panel.columns)
    if missing:
        raise ValueError(f"missing panel columns: {sorted(missing)}")

    sectors = tuple(sector_symbols)
    if not sectors or len(set(sectors)) != len(sectors):
        raise ValueError("sector_symbols must be non-empty and unique")

    rows: list[dict[str, object]] = []
    latest_timestamps: set[pd.Timestamp] = set()
    for symbol in sectors:
        series = panel.loc[panel["symbol"] == symbol].copy()
        if series.empty:
            raise ValueError(f"missing RRG history for {symbol}")
        series["timestamp"] = pd.to_datetime(series["timestamp"], utc=True)
        rrg = series.set_index("timestamp").sort_index()
        latest_timestamps.add(pd.Timestamp(rrg.index[-1]))
        for horizon in cfg.horizons:
            result = forecast_latest_for_series(
                rrg,
                horizon=horizon,
                lookback=cfg.lookback,
            )
            result = {"symbol": symbol, **result}
            rows.append(result)

    if len(latest_timestamps) != 1:
        raise AssertionError("sector forecasts do not share one common decision timestamp")

    output = pd.DataFrame(rows).sort_values(["symbol", "horizon"]).reset_index(drop=True)
    if output.duplicated(["symbol", "horizon"]).any():
        raise AssertionError("duplicate symbol/horizon deployment forecast")
    return output


def make_forecast_table(forecasts: pd.DataFrame) -> pd.DataFrame:
    """Create a compact 11-row human-readable sector outlook table."""
    required = {
        "symbol", "horizon", "decision_timestamp", "current_quadrant",
        "predicted_quadrant", "predicted_rs_ratio", "predicted_rs_momentum",
    }
    missing = required.difference(forecasts.columns)
    if missing:
        raise ValueError(f"missing forecast columns: {sorted(missing)}")

    records: list[dict[str, object]] = []
    for symbol, group in forecasts.groupby("symbol", sort=True):
        group = group.sort_values("horizon")
        record: dict[str, object] = {
            "symbol": symbol,
            "decision_timestamp": group.iloc[0]["decision_timestamp"],
            "current_quadrant": group.iloc[0]["current_quadrant"],
        }
        for _, row in group.iterrows():
            h = int(row["horizon"])
            record[f"{h}w_target_timestamp"] = row["target_timestamp"]
            record[f"{h}w_quadrant"] = row["predicted_quadrant"]
            record[f"{h}w_rs_ratio"] = float(row["predicted_rs_ratio"])
            record[f"{h}w_rs_momentum"] = float(row["predicted_rs_momentum"])
        records.append(record)
    return pd.DataFrame.from_records(records)
