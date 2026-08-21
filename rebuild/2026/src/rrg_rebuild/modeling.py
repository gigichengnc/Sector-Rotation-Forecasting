from __future__ import annotations

from dataclasses import dataclass

import numpy as np
import pandas as pd
from sklearn.linear_model import LinearRegression
from sklearn.pipeline import Pipeline
from sklearn.preprocessing import StandardScaler

from .rrg import classify_quadrant
from .validation import WalkForwardConfig, WalkForwardFold, generate_walk_forward_folds


def make_coordinate_supervised_dataset(
    rrg: pd.DataFrame,
    *,
    horizon: int,
    lookback: int = 20,
) -> pd.DataFrame:
    if horizon <= 0:
        raise ValueError("horizon must be > 0")
    if lookback <= 0:
        raise ValueError("lookback must be > 0")

    required = {"rs_ratio", "rs_momentum"}
    missing = required.difference(rrg.columns)
    if missing:
        raise ValueError(f"missing required columns: {sorted(missing)}")
    if not rrg.index.is_monotonic_increasing:
        raise ValueError("rrg index must be chronological")
    if rrg.index.has_duplicates:
        raise ValueError("rrg index must be unique")

    out = pd.DataFrame(index=rrg.index)
    for lag in range(lookback):
        out[f"rs_ratio_lag{lag}"] = rrg["rs_ratio"].shift(lag)
        out[f"rs_momentum_lag{lag}"] = rrg["rs_momentum"].shift(lag)

    out["target_rs_ratio"] = rrg["rs_ratio"].shift(-horizon)
    out["target_rs_momentum"] = rrg["rs_momentum"].shift(-horizon)
    out = out.dropna().copy()

    if out.empty:
        raise ValueError("not enough observations for requested lookback/horizon")
    return out


def feature_columns(frame: pd.DataFrame) -> list[str]:
    return [
        column
        for column in frame.columns
        if column.startswith("rs_ratio_lag") or column.startswith("rs_momentum_lag")
    ]


def fit_linear_coordinate_model(
    X_train: pd.DataFrame,
    y_train: pd.DataFrame,
) -> Pipeline:
    if len(X_train) != len(y_train):
        raise ValueError("X_train and y_train length mismatch")
    if len(X_train) < 2:
        raise ValueError("at least two training rows are required")

    pipeline = Pipeline(
        steps=[
            ("scale", StandardScaler()),
            ("model", LinearRegression()),
        ]
    )
    pipeline.fit(X_train, y_train)
    return pipeline


def _quadrants(ratio: np.ndarray, momentum: np.ndarray) -> list[str]:
    return [
        classify_quadrant(float(r), float(m))
        for r, m in zip(ratio, momentum, strict=True)
    ]


def _prediction_rows(
    *,
    frame: pd.DataFrame,
    fold: WalkForwardFold,
    horizon: int,
    model_name: str,
    predicted: np.ndarray,
) -> pd.DataFrame:
    validation = frame.iloc[fold.validation_slice]
    actual = validation[["target_rs_ratio", "target_rs_momentum"]].to_numpy(float)

    rows = pd.DataFrame(
        {
            "timestamp": validation.index,
            "fold_id": fold.fold_id,
            "horizon": horizon,
            "model": model_name,
            "actual_rs_ratio": actual[:, 0],
            "actual_rs_momentum": actual[:, 1],
            "predicted_rs_ratio": predicted[:, 0],
            "predicted_rs_momentum": predicted[:, 1],
        }
    )
    rows["actual_quadrant"] = _quadrants(
        rows["actual_rs_ratio"].to_numpy(), rows["actual_rs_momentum"].to_numpy()
    )
    rows["predicted_quadrant"] = _quadrants(
        rows["predicted_rs_ratio"].to_numpy(), rows["predicted_rs_momentum"].to_numpy()
    )
    return rows


@dataclass(frozen=True)
class EvaluationResult:
    predictions: pd.DataFrame
    summary: pd.DataFrame
    folds: tuple[WalkForwardFold, ...]


def summarize_predictions(predictions: pd.DataFrame) -> pd.DataFrame:
    required = {
        "model", "actual_rs_ratio", "actual_rs_momentum",
        "predicted_rs_ratio", "predicted_rs_momentum",
        "actual_quadrant", "predicted_quadrant",
    }
    missing = required.difference(predictions.columns)
    if missing:
        raise ValueError(f"missing prediction columns: {sorted(missing)}")

    records: list[dict[str, float | int | str]] = []
    for model_name, group in predictions.groupby("model", sort=False):
        ratio_error = group["predicted_rs_ratio"] - group["actual_rs_ratio"]
        momentum_error = group["predicted_rs_momentum"] - group["actual_rs_momentum"]
        euclidean = np.sqrt(ratio_error.to_numpy() ** 2 + momentum_error.to_numpy() ** 2)
        records.append({
            "model": str(model_name),
            "n": int(len(group)),
            "mae_rs_ratio": float(ratio_error.abs().mean()),
            "mae_rs_momentum": float(momentum_error.abs().mean()),
            "rmse_rs_ratio": float(np.sqrt(np.mean(ratio_error.to_numpy() ** 2))),
            "rmse_rs_momentum": float(np.sqrt(np.mean(momentum_error.to_numpy() ** 2))),
            "mean_coordinate_distance": float(np.mean(euclidean)),
            "quadrant_accuracy": float((group["actual_quadrant"] == group["predicted_quadrant"]).mean()),
        })
    return pd.DataFrame.from_records(records)


def evaluate_coordinate_baselines(
    rrg: pd.DataFrame,
    *,
    horizon: int,
    lookback: int,
    walk_forward: WalkForwardConfig,
) -> EvaluationResult:
    if walk_forward.gap < horizon:
        raise ValueError("walk-forward gap must be >= horizon to prevent target leakage")
    frame = make_coordinate_supervised_dataset(rrg, horizon=horizon, lookback=lookback)
    folds = generate_walk_forward_folds(len(frame), walk_forward)
    features = feature_columns(frame)
    target_columns = ["target_rs_ratio", "target_rs_momentum"]
    output: list[pd.DataFrame] = []
    for fold in folds:
        train = frame.iloc[fold.train_slice]
        validation = frame.iloc[fold.validation_slice]
        X_train = train[features]
        y_train = train[target_columns]
        X_validation = validation[features]
        persistence = validation[["rs_ratio_lag0", "rs_momentum_lag0"]].to_numpy(float)
        output.append(_prediction_rows(frame=frame, fold=fold, horizon=horizon, model_name="persistence", predicted=persistence))
        linear_model = fit_linear_coordinate_model(X_train, y_train)
        linear = np.asarray(linear_model.predict(X_validation), dtype=float)
        if linear.ndim != 2 or linear.shape[1] != 2:
            raise AssertionError("linear model must return two coordinate predictions")
        output.append(_prediction_rows(frame=frame, fold=fold, horizon=horizon, model_name="linear", predicted=linear))
    predictions = pd.concat(output, ignore_index=True)
    duplicate_key = ["model", "horizon", "timestamp"]
    if predictions.duplicated(duplicate_key).any():
        raise AssertionError("an out-of-sample observation was scored more than once")
    return EvaluationResult(predictions=predictions, summary=summarize_predictions(predictions), folds=tuple(folds))
