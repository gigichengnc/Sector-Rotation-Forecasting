import numpy as np
import pandas as pd
import pytest

from rrg_rebuild.holdout import (
    FINAL_HOLDOUT_AUTHORIZATION_TOKEN,
    FinalHoldoutProtocol,
    make_timestamped_supervised_dataset,
    require_final_holdout_authorization,
    split_fixed_final_holdout,
)


def synthetic_rrg(n=90):
    index = pd.date_range("2024-01-05", periods=n, freq="W-FRI", tz="UTC")
    return pd.DataFrame(
        {
            "rs_ratio": 100 + np.linspace(-5, 8, n),
            "rs_momentum": 100 + np.sin(np.linspace(0, 8, n)) * 4,
        },
        index=index,
    )


def test_guard_refuses_without_explicit_open_flag():
    with pytest.raises(PermissionError, match="closed"):
        require_final_holdout_authorization(
            open_final_holdout=False,
            authorization_token=FINAL_HOLDOUT_AUTHORIZATION_TOKEN,
        )


def test_guard_refuses_wrong_token():
    with pytest.raises(PermissionError, match="token"):
        require_final_holdout_authorization(
            open_final_holdout=True,
            authorization_token="WRONG",
        )


def test_guard_accepts_both_deliberate_gates():
    require_final_holdout_authorization(
        open_final_holdout=True,
        authorization_token=FINAL_HOLDOUT_AUTHORIZATION_TOKEN,
    )


def test_timestamped_dataset_records_future_target_timestamp():
    rrg = synthetic_rrg()
    frame = make_timestamped_supervised_dataset(rrg, horizon=4, lookback=5)
    delta = frame["target_timestamp"] - frame["decision_timestamp"]
    assert (delta == pd.Timedelta(weeks=4)).all()


def test_split_uses_target_window_and_strict_training_cutoff():
    rrg = synthetic_rrg()
    frame = make_timestamped_supervised_dataset(rrg, horizon=4, lookback=5)
    targets = pd.to_datetime(frame["target_timestamp"], utc=True)
    start = targets.iloc[-12]
    end = targets.iloc[-3]
    protocol = FinalHoldoutProtocol(
        target_start=start,
        target_end=end,
        horizons=(4,),
        lookback=5,
    )
    split = split_fixed_final_holdout(frame, protocol=protocol)

    holdout_targets = pd.to_datetime(split.holdout["target_timestamp"], utc=True)
    train_targets = pd.to_datetime(split.train["target_timestamp"], utc=True)

    assert holdout_targets.min() == start
    assert holdout_targets.max() == end
    assert train_targets.max() < split.first_holdout_decision


def test_no_holdout_target_enters_training_even_for_long_horizon():
    rrg = synthetic_rrg(120)
    frame = make_timestamped_supervised_dataset(rrg, horizon=8, lookback=20)
    targets = pd.to_datetime(frame["target_timestamp"], utc=True)
    protocol = FinalHoldoutProtocol(
        target_start=targets.iloc[-20],
        target_end=targets.iloc[-1],
        horizons=(8,),
        lookback=20,
    )
    split = split_fixed_final_holdout(frame, protocol=protocol)
    assert (
        pd.to_datetime(split.train["target_timestamp"], utc=True)
        < split.first_holdout_decision
    ).all()
    assert (
        pd.to_datetime(split.holdout["target_timestamp"], utc=True)
        >= protocol.target_start
    ).all()
