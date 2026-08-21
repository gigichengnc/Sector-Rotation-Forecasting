import pandas as pd
import pytest

from rrg_rebuild.dataset import align_common_weekly_history, build_sector_rrg_panel
from rrg_rebuild.rrg import RRGConfig


def _weekly(symbol: str, start: str, values: list[float]) -> pd.DataFrame:
    week_end = pd.date_range(start, periods=len(values), freq="W-FRI", tz="UTC")
    source = week_end - pd.Timedelta(days=1) + pd.Timedelta(hours=21)
    return pd.DataFrame(
        {
            "symbol": symbol,
            "week_end": week_end + pd.Timedelta(days=1) - pd.Timedelta(nanoseconds=1),
            "source_timestamp": source,
            "adj_close": values,
        }
    )


def test_common_history_starts_when_latest_symbol_becomes_available() -> None:
    spy = _weekly("SPY", "2026-01-02", [100, 101, 102, 103, 104, 105])
    xlk = _weekly("XLK", "2026-01-02", [50, 51, 52, 53, 54, 55])
    xlc = _weekly("XLC", "2026-01-16", [70, 71, 72, 73])

    prices, sources, history = align_common_weekly_history(
        {"SPY": spy, "XLK": xlk, "XLC": xlc},
        required_symbols=["SPY", "XLK", "XLC"],
    )

    assert len(prices) == 4
    assert history.common_start == xlc.loc[0, "week_end"]
    assert prices.index.equals(sources.index)
    assert list(prices.columns) == ["SPY", "XLK", "XLC"]


def test_common_history_does_not_forward_fill_missing_week() -> None:
    spy = _weekly("SPY", "2026-01-02", [100, 101, 102, 103])
    xlk = _weekly("XLK", "2026-01-02", [50, 51, 52, 53]).drop(index=1).reset_index(drop=True)

    prices, _, _ = align_common_weekly_history(
        {"SPY": spy, "XLK": xlk}, required_symbols=["SPY", "XLK"]
    )

    assert len(prices) == 3
    assert spy.loc[1, "week_end"] not in prices.index


def test_common_history_rejects_missing_symbol() -> None:
    with pytest.raises(ValueError, match="missing required symbols"):
        align_common_weekly_history(
            {"SPY": _weekly("SPY", "2026-01-02", [100, 101])},
            required_symbols=["SPY", "XLC"],
        )


def test_common_history_rejects_future_source_bar() -> None:
    spy = _weekly("SPY", "2026-01-02", [100, 101, 102])
    xlk = _weekly("XLK", "2026-01-02", [50, 51, 52])
    xlk.loc[1, "source_timestamp"] = (
        xlk.loc[1, "week_end"].floor("s") + pd.Timedelta(seconds=1)
    )

    with pytest.raises(ValueError, match="after weekly cutoff"):
        align_common_weekly_history(
            {"SPY": spy, "XLK": xlk}, required_symbols=["SPY", "XLK"]
        )


def test_sector_panel_is_unique_by_symbol_timestamp() -> None:
    n = 20
    spy = _weekly("SPY", "2026-01-02", [100 + i for i in range(n)])
    xlk = _weekly("XLK", "2026-01-02", [100 + 1.4 * i for i in range(n)])
    xle = _weekly("XLE", "2026-01-02", [100 + 0.7 * i for i in range(n)])
    prices, _, _ = align_common_weekly_history(
        {"SPY": spy, "XLK": xlk, "XLE": xle},
        required_symbols=["SPY", "XLK", "XLE"],
    )

    panel = build_sector_rrg_panel(
        prices,
        sector_symbols=["XLK", "XLE"],
        config=RRGConfig(ratio_period=2, momentum_period=1, normalization_period=4),
    )

    assert set(panel["symbol"]) == {"XLK", "XLE"}
    assert not panel.duplicated(["symbol", "timestamp"]).any()
