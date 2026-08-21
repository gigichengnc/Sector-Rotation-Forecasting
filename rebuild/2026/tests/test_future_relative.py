import numpy as np
import pandas as pd

from rrg_rebuild.future_relative import make_future_relative_return_dataset


def test_future_relative_target_uses_only_post_decision_price_change():
    index = pd.date_range("2024-01-05", periods=30, freq="W-FRI", tz="UTC")
    asset = pd.Series(100.0 * np.exp(np.arange(30) * 0.02), index=index)
    benchmark = pd.Series(100.0 * np.exp(np.arange(30) * 0.01), index=index)
    rrg = pd.DataFrame(
        {
            "rs_ratio": np.linspace(95.0, 105.0, 30),
            "rs_momentum": np.linspace(97.0, 103.0, 30),
        },
        index=index,
    )

    frame = make_future_relative_return_dataset(
        rrg,
        asset,
        benchmark,
        horizon=4,
        lookback=3,
    )
    row = frame.iloc[0]
    decision = row["decision_timestamp"]
    target = row["target_timestamp"]

    expected = np.log(asset.loc[target] / asset.loc[decision]) - np.log(
        benchmark.loc[target] / benchmark.loc[decision]
    )
    assert np.isclose(row["future_relative_log_return"], expected)
    assert row["target_outperform"] == int(expected > 0)
    assert target > decision
