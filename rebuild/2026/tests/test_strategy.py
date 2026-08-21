import numpy as np
import pandas as pd
import pytest

from rrg_rebuild.strategy import (
    StrategyBacktestConfig,
    execution_schedule,
    forecast_at_decision,
    performance_summary,
    simulate_target_weight_strategy,
    top_k_equal_weights,
)


def _rrg(n=190):
    idx = pd.date_range("2020-01-03", periods=n, freq="W-FRI", tz="UTC") + pd.Timedelta(days=1) - pd.Timedelta(nanoseconds=1)
    t = np.arange(n, dtype=float)
    return pd.DataFrame(
        {
            "rs_ratio": 100 + 0.04 * t + np.sin(t / 7),
            "rs_momentum": 100 + 0.02 * t + np.cos(t / 9),
        },
        index=idx,
    )


def test_forecast_at_decision_is_unchanged_by_future_rows():
    frame = _rrg()
    cfg = StrategyBacktestConfig(min_training_rows=156)
    decision = frame.index[180]
    a = forecast_at_decision(frame, decision_timestamp=decision, config=cfg)
    changed = frame.copy()
    changed.loc[changed.index > decision, "rs_ratio"] += 999
    changed.loc[changed.index > decision, "rs_momentum"] -= 999
    b = forecast_at_decision(changed, decision_timestamp=decision, config=cfg)
    assert a["predicted_rs_ratio"] == pytest.approx(b["predicted_rs_ratio"])
    assert a["predicted_rs_momentum"] == pytest.approx(b["predicted_rs_momentum"])
    assert a["training_last_target"] == decision


def test_top_k_weights_are_deterministic_and_sum_to_one():
    d = pd.Timestamp("2026-01-02T23:59:59Z")
    rows = pd.DataFrame(
        {
            "decision_timestamp": [d] * 4,
            "symbol": ["A", "B", "C", "D"],
            "score": [5.0, 5.0, 4.0, 3.0],
        }
    )
    weights = top_k_equal_weights(rows, score_column="score", sector_symbols=("A", "B", "C", "D"), top_k=2)
    selected = weights.loc[weights.target_weight > 0, "symbol"].tolist()
    assert selected == ["A", "B"]
    assert weights.target_weight.sum() == pytest.approx(1.0)


def test_execution_is_strictly_after_decision():
    daily = pd.DatetimeIndex(pd.to_datetime(["2026-01-02T20:00:00Z", "2026-01-05T20:00:00Z", "2026-01-06T20:00:00Z"]))
    decision = pd.Timestamp("2026-01-02T23:59:59Z")
    schedule = execution_schedule([decision], common_daily_index=daily)
    assert schedule.iloc[0] == pd.Timestamp("2026-01-05T20:00:00Z")


def test_simulator_charges_initial_and_rebalance_cost_with_drift():
    decisions = pd.to_datetime(["2026-01-02T23:59:59Z", "2026-01-09T23:59:59Z", "2026-01-16T23:59:59Z"], utc=True)
    targets = pd.DataFrame(
        [
            {"decision_timestamp": decisions[0], "symbol": "A", "target_weight": 1.0},
            {"decision_timestamp": decisions[1], "symbol": "B", "target_weight": 1.0},
            {"decision_timestamp": decisions[2], "symbol": "B", "target_weight": 1.0},
        ]
    )
    daily_idx = pd.to_datetime(["2026-01-05T20:00:00Z", "2026-01-12T20:00:00Z", "2026-01-20T20:00:00Z"], utc=True)
    prices = pd.DataFrame({"A": [100, 110, 110], "B": [100, 100, 105]}, index=daily_idx)
    periods = simulate_target_weight_strategy(targets, common_daily_prices=prices, transaction_cost_per_dollar=0.001)
    assert len(periods) == 2
    assert periods.iloc[0].gross_traded_weight == pytest.approx(1.0)
    # Full A->B switch at the second execution trades one sell and one buy.
    assert periods.iloc[1].gross_traded_weight == pytest.approx(2.0)
    assert periods.iloc[0].net_return < periods.iloc[0].gross_return


def test_performance_summary_has_drawdown_and_cagr():
    idx = pd.to_datetime(["2026-01-05T20:00:00Z", "2026-01-12T20:00:00Z", "2026-01-20T20:00:00Z"], utc=True)
    periods = pd.DataFrame(
        {
            "execution_timestamp": idx[:2],
            "exit_timestamp": idx[1:],
            "net_return": [0.10, -0.05],
            "gross_return": [0.10, -0.05],
            "gross_traded_weight": [1.0, 0.2],
            "transaction_cost_fraction": [0.001, 0.0002],
            "transaction_cost_paid": [0.001, 0.00022],
            "equity_end": [1.10, 1.045],
        }
    )
    s = performance_summary(periods)
    assert s["ending_value"] == pytest.approx(1.045)
    assert s["max_drawdown"] == pytest.approx(-0.05)
    assert np.isfinite(s["cagr"])


def test_fast_ols_matches_frozen_sklearn_pipeline():
    from rrg_rebuild.modeling import fit_linear_coordinate_model
    from rrg_rebuild.strategy import _ols_equivalent_prediction

    rng = np.random.default_rng(2026)
    X = pd.DataFrame(rng.normal(size=(180, 8)), columns=[f"x{i}" for i in range(8)])
    y = pd.DataFrame(rng.normal(size=(180, 2)), columns=["y1", "y2"])
    latest = pd.DataFrame(rng.normal(size=(1, 8)), columns=X.columns)
    sklearn_pred = fit_linear_coordinate_model(X, y).predict(latest)
    fast_pred = _ols_equivalent_prediction(X, y, latest)
    assert np.allclose(fast_pred, sklearn_pred, rtol=1e-10, atol=1e-10)
