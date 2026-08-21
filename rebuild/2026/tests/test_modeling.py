import numpy as np
import pandas as pd
import pytest

from rrg_rebuild.modeling import (
    evaluate_coordinate_baselines,
    fit_linear_coordinate_model,
    make_coordinate_supervised_dataset,
)
from rrg_rebuild.validation import WalkForwardConfig


def _trending_rrg(n: int = 90) -> pd.DataFrame:
    idx = pd.date_range("2024-01-05", periods=n, freq="W-FRI", tz="UTC")
    t = np.arange(n, dtype=float)
    ratio = 96.0 + 0.12 * t + 0.2 * np.sin(t / 5.0)
    momentum = 98.0 + 0.08 * t + 0.15 * np.cos(t / 6.0)
    return pd.DataFrame({"rs_ratio": ratio, "rs_momentum": momentum}, index=idx)


def test_supervised_features_are_backward_looking_and_targets_forward() -> None:
    rrg = _trending_rrg(30)
    dataset = make_coordinate_supervised_dataset(rrg, horizon=2, lookback=3)
    ts = dataset.index[0]
    loc = rrg.index.get_loc(ts)

    assert dataset.loc[ts, "rs_ratio_lag0"] == pytest.approx(rrg.iloc[loc]["rs_ratio"])
    assert dataset.loc[ts, "rs_ratio_lag2"] == pytest.approx(rrg.iloc[loc - 2]["rs_ratio"])
    assert dataset.loc[ts, "target_rs_ratio"] == pytest.approx(rrg.iloc[loc + 2]["rs_ratio"])


def test_linear_scaler_is_fit_only_on_training_rows() -> None:
    X_train = pd.DataFrame({"a": [1.0, 2.0, 3.0], "b": [10.0, 11.0, 12.0]})
    y_train = pd.DataFrame({"y1": [2.0, 4.0, 6.0], "y2": [3.0, 5.0, 7.0]})
    model = fit_linear_coordinate_model(X_train, y_train)

    scaler = model.named_steps["scale"]
    assert scaler.mean_[0] == pytest.approx(2.0)
    assert scaler.mean_[1] == pytest.approx(11.0)


def test_evaluation_requires_horizon_sized_purge_gap() -> None:
    with pytest.raises(ValueError, match="gap must be >= horizon"):
        evaluate_coordinate_baselines(
            _trending_rrg(),
            horizon=4,
            lookback=5,
            walk_forward=WalkForwardConfig(30, 5, gap=3),
        )


def test_out_of_sample_rows_are_scored_once_per_model() -> None:
    result = evaluate_coordinate_baselines(
        _trending_rrg(),
        horizon=2,
        lookback=5,
        walk_forward=WalkForwardConfig(30, 8, gap=2),
    )
    assert not result.predictions.duplicated(["model", "horizon", "timestamp"]).any()
    counts = result.predictions.groupby("model").size()
    assert counts["persistence"] == counts["linear"]


def test_linear_baseline_can_beat_persistence_on_known_linear_trend() -> None:
    result = evaluate_coordinate_baselines(
        _trending_rrg(),
        horizon=1,
        lookback=4,
        walk_forward=WalkForwardConfig(35, 10, gap=1),
    )
    summary = result.summary.set_index("model")

    assert summary.loc["linear", "mean_coordinate_distance"] < summary.loc[
        "persistence", "mean_coordinate_distance"
    ]
    assert 0.0 <= summary.loc["linear", "quadrant_accuracy"] <= 1.0
