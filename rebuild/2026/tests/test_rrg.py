import numpy as np
import pandas as pd
import pytest

from rrg_rebuild.rrg import RRGConfig, calculate_rrg, classify_quadrant


def test_quadrant_boundaries_are_explicit() -> None:
    assert classify_quadrant(100, 100) == "Leading"
    assert classify_quadrant(101, 99) == "Weakening"
    assert classify_quadrant(99, 99) == "Lagging"
    assert classify_quadrant(99, 101) == "Improving"


def test_relative_return_ratio_matches_hand_calculation() -> None:
    idx = pd.date_range("2026-01-01", periods=6, freq="D")
    asset = pd.Series([100, 102, 104, 108, 112, 120], index=idx, dtype=float)
    benchmark = pd.Series([100, 101, 102, 103, 104, 105], index=idx, dtype=float)
    cfg = RRGConfig(ratio_period=2, momentum_period=1, normalization_period=3)

    result = calculate_rrg(asset, benchmark, cfg)

    t = idx[3]
    expected_rs_t = (108 / 102) / (103 / 101)
    expected_rs_prev = (104 / 100) / (102 / 100)
    expected_momentum_t = expected_rs_t / expected_rs_prev - 1

    assert result.loc[t, "rs_ratio_raw"] == pytest.approx(expected_rs_t)
    assert result.loc[t, "rs_momentum_raw"] == pytest.approx(expected_momentum_t)


def test_equal_asset_and_benchmark_produce_center_coordinates() -> None:
    idx = pd.date_range("2026-01-01", periods=20, freq="D")
    prices = pd.Series(np.linspace(100, 130, len(idx)), index=idx)
    cfg = RRGConfig(ratio_period=2, momentum_period=2, normalization_period=5)

    result = calculate_rrg(prices, prices, cfg)

    assert (result["rs_ratio"] == 100.0).all()
    assert (result["rs_momentum"] == 100.0).all()
    assert (result["quadrant"] == "Leading").all()


def test_calculation_is_causal_with_respect_to_future_prices() -> None:
    idx = pd.date_range("2026-01-01", periods=40, freq="D")
    asset = pd.Series(np.linspace(100, 140, len(idx)), index=idx)
    benchmark = pd.Series(np.linspace(100, 125, len(idx)), index=idx)
    cfg = RRGConfig(ratio_period=3, momentum_period=2, normalization_period=8)

    original = calculate_rrg(asset, benchmark, cfg)

    cutoff = idx[25]
    modified_asset = asset.copy()
    modified_asset.loc[modified_asset.index > cutoff] *= 20
    modified = calculate_rrg(modified_asset, benchmark, cfg)

    common = original.index.intersection(modified.index)
    pre_cutoff = common[common <= cutoff]
    pd.testing.assert_frame_equal(
        original.loc[pre_cutoff],
        modified.loc[pre_cutoff],
        check_exact=False,
        rtol=1e-12,
        atol=1e-12,
    )


def test_rejects_non_positive_price() -> None:
    idx = pd.date_range("2026-01-01", periods=8, freq="D")
    asset = pd.Series([100, 101, 0, 103, 104, 105, 106, 107], index=idx)
    benchmark = pd.Series([100, 101, 102, 103, 104, 105, 106, 107], index=idx)

    with pytest.raises(ValueError, match="strictly positive"):
        calculate_rrg(asset, benchmark, RRGConfig(2, 1, 3))
