import pandas as pd

from rrg_rebuild.targets import add_future_coordinate_targets, add_persistence_baseline


def test_targets_are_shifted_forward_in_observation_units() -> None:
    idx = pd.date_range("2026-01-02", periods=5, freq="W-FRI")
    frame = pd.DataFrame(
        {
            "rs_ratio": [98.0, 99.0, 100.0, 101.0, 102.0],
            "rs_momentum": [97.0, 98.0, 99.0, 100.0, 101.0],
            "quadrant": ["Lagging", "Lagging", "Weakening", "Leading", "Leading"],
        },
        index=idx,
    )

    out = add_future_coordinate_targets(frame, horizons=(1, 2))

    assert out.loc[idx[0], "target_rs_ratio_h1"] == 99.0
    assert out.loc[idx[0], "target_rs_momentum_h2"] == 99.0
    assert out.loc[idx[1], "target_quadrant_h2"] == "Leading"


def test_persistence_baseline_uses_only_current_coordinate() -> None:
    idx = pd.date_range("2026-01-02", periods=3, freq="W-FRI")
    frame = pd.DataFrame(
        {"rs_ratio": [99.0, 100.0, 101.0], "rs_momentum": [98.0, 99.0, 100.0]},
        index=idx,
    )

    out = add_persistence_baseline(frame, horizons=(1, 4))
    assert out.loc[idx[0], "pred_rs_ratio_persistence_h4"] == 99.0
    assert out.loc[idx[2], "pred_rs_momentum_persistence_h1"] == 100.0
