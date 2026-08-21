import numpy as np
import pandas as pd

from rrg_rebuild.deployment import (
    DeploymentConfig,
    forecast_latest_for_series,
    forecast_panel_latest,
    make_forecast_table,
    make_latest_feature_row,
)


def _rrg(n: int = 60) -> pd.DataFrame:
    index = pd.date_range("2025-01-03", periods=n, freq="W-FRI", tz="UTC")
    return pd.DataFrame(
        {
            "rs_ratio": 95.0 + np.arange(n) * 0.15,
            "rs_momentum": 98.0 + np.arange(n) * 0.07,
        },
        index=index,
    )


def test_latest_feature_row_preserves_lag_semantics() -> None:
    rrg = _rrg(30)
    row = make_latest_feature_row(rrg, lookback=4)
    assert row.index[0] == rrg.index[-1]
    assert row.iloc[0]["rs_ratio_lag0"] == rrg.iloc[-1]["rs_ratio"]
    assert row.iloc[0]["rs_momentum_lag0"] == rrg.iloc[-1]["rs_momentum"]
    assert row.iloc[0]["rs_ratio_lag3"] == rrg.iloc[-4]["rs_ratio"]
    assert row.iloc[0]["rs_momentum_lag3"] == rrg.iloc[-4]["rs_momentum"]


def test_deployment_uses_only_labels_known_by_latest_observation() -> None:
    rrg = _rrg(60)
    result = forecast_latest_for_series(rrg, horizon=8, lookback=20)
    assert result["decision_timestamp"] == rrg.index[-1]
    assert result["training_last_target"] == rrg.index[-1]
    assert result["training_last_decision"] == rrg.index[-9]
    assert result["target_timestamp"] == rrg.index[-1] + pd.Timedelta(weeks=8)
    assert result["training_rows"] == 60 - 20 - 8 + 1


def test_deployment_output_has_no_probability_claim() -> None:
    result = forecast_latest_for_series(_rrg(60), horizon=1, lookback=20)
    assert "confidence" not in result
    assert "probability" not in result
    assert result["predicted_quadrant"] in {"Leading", "Weakening", "Lagging", "Improving"}


def test_panel_forecasts_every_symbol_horizon_once() -> None:
    base = _rrg(60)
    blocks = []
    for symbol, offset in [("XLK", 0.0), ("XLE", 0.5), ("XLF", -0.5)]:
        b = base.copy()
        b["rs_ratio"] += offset
        b["symbol"] = symbol
        b["timestamp"] = b.index
        blocks.append(b.reset_index(drop=True))
    panel = pd.concat(blocks, ignore_index=True)
    result = forecast_panel_latest(
        panel,
        sector_symbols=["XLK", "XLE", "XLF"],
        config=DeploymentConfig(horizons=(1, 2, 4, 8), lookback=20),
    )
    assert len(result) == 12
    assert not result.duplicated(["symbol", "horizon"]).any()
    assert result["decision_timestamp"].nunique() == 1


def test_compact_forecast_table_has_one_row_per_symbol() -> None:
    base = _rrg(60)
    blocks = []
    for symbol in ["XLK", "XLE"]:
        b = base.copy()
        b["symbol"] = symbol
        b["timestamp"] = b.index
        blocks.append(b.reset_index(drop=True))
    long = forecast_panel_latest(
        pd.concat(blocks, ignore_index=True),
        sector_symbols=["XLK", "XLE"],
        config=DeploymentConfig(horizons=(1, 4), lookback=20),
    )
    table = make_forecast_table(long)
    assert len(table) == 2
    assert {"1w_quadrant", "4w_quadrant", "1w_rs_ratio", "4w_rs_momentum"}.issubset(table.columns)
