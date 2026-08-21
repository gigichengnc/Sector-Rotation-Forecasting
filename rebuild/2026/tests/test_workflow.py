from datetime import datetime, timezone
import json
import math
from pathlib import Path
import zipfile

import pandas as pd

from rrg_rebuild.workflow import DEFAULT_SYMBOLS, fetch_market_data, run_fresh_forecast


def _payload() -> bytes:
    timestamps = [
        int(pd.Timestamp("2026-01-05T21:00:00Z").timestamp()),
        int(pd.Timestamp("2026-01-09T21:00:00Z").timestamp()),
        int(pd.Timestamp("2026-01-12T21:00:00Z").timestamp()),
        int(pd.Timestamp("2026-01-15T21:00:00Z").timestamp()),
    ]
    obj = {
        "chart": {
            "result": [
                {
                    "timestamp": timestamps,
                    "indicators": {
                        "quote": [
                            {
                                "open": [100, 101, 102, 103],
                                "high": [101, 102, 103, 104],
                                "low": [99, 100, 101, 102],
                                "close": [100.5, 101.5, 102.5, 103.5],
                                "volume": [1000, 1100, 1200, 1300],
                            }
                        ],
                        "adjclose": [{"adjclose": [99.5, 100.5, 101.5, 102.5]}],
                    },
                }
            ],
            "error": None,
        }
    }
    return json.dumps(obj, separators=(",", ":")).encode()


def _fake_fetch(symbol: str, start_epoch: int, end_epoch: int, timeout: float):
    assert symbol == "SPY"
    assert start_epoch < end_epoch
    assert timeout > 0
    return (
        _payload(),
        "https://example.test/chart/SPY",
        datetime(2026, 1, 17, 0, 0, tzinfo=timezone.utc),
    )


def _universe_payload(symbol: str, periods: int = 90) -> bytes:
    index = pd.date_range("2024-01-05", periods=periods, freq="W-FRI", tz="UTC")
    timestamps = [int((ts + pd.Timedelta(hours=21)).timestamp()) for ts in index]
    offset = 1 + (sum(ord(char) for char in symbol) % 13)
    adjusted = [
        100.0 + 0.22 * i + 0.0009 * offset * i * i + 0.35 * math.sin((i + offset) / 6.0)
        for i in range(periods)
    ]
    obj = {
        "chart": {
            "result": [
                {
                    "timestamp": timestamps,
                    "indicators": {
                        "quote": [
                            {
                                "open": adjusted,
                                "high": [value + 1.0 for value in adjusted],
                                "low": [value - 1.0 for value in adjusted],
                                "close": adjusted,
                                "volume": [1_000_000 + i for i in range(periods)],
                            }
                        ],
                        "adjclose": [{"adjclose": adjusted}],
                    },
                }
            ],
            "error": None,
        }
    }
    return json.dumps(obj, separators=(",", ":")).encode()


def _fake_universe_fetch(symbol: str, start_epoch: int, end_epoch: int, timeout: float):
    assert symbol in DEFAULT_SYMBOLS
    assert start_epoch < end_epoch
    assert timeout > 0
    return (
        _universe_payload(symbol),
        f"https://example.test/chart/{symbol}",
        datetime(2026, 1, 1, 0, 0, tzinfo=timezone.utc),
    )


def test_fetch_manifest_uses_relative_paths_and_portable_archive(tmp_path: Path) -> None:
    root = tmp_path / "run" / "market-data"
    result = fetch_market_data(
        root,
        start_date="2026-01-01",
        end_date="2026-02-01",
        symbols=("SPY",),
        sleep_seconds=0,
        fetcher=_fake_fetch,
    )

    manifest = pd.read_csv(result.manifest_path)
    assert manifest.loc[0, "symbol"] == "SPY"
    assert manifest.loc[0, "raw_path"].startswith("raw/yahoo/")
    assert manifest.loc[0, "daily_csv_path"] == "processed/daily/SPY.csv"
    assert str(tmp_path) not in manifest.loc[0, "raw_path"]
    assert str(tmp_path) not in manifest.loc[0, "daily_csv_path"]
    assert len(manifest.loc[0, "raw_sha256"]) == 64

    assert result.archive_path == tmp_path / "run" / "market-data.zip"
    with zipfile.ZipFile(result.archive_path) as zf:
        names = set(zf.namelist())
    assert "manifest.csv" in names
    assert "processed/daily/SPY.csv" in names
    assert any(name.startswith("raw/yahoo/SPY_") and name.endswith(".json") for name in names)

    weekly = result.weekly_by_symbol["SPY"]
    assert len(weekly) == 2
    assert weekly["week_end"].max() <= pd.Timestamp("2026-01-17T00:00:00Z")


def test_fetch_archive_name_tracks_output_directory(tmp_path: Path) -> None:
    first = fetch_market_data(
        tmp_path / "market-data-20260821T100000Z",
        start_date="2026-01-01",
        end_date="2026-02-01",
        symbols=("SPY",),
        sleep_seconds=0,
        fetcher=_fake_fetch,
    )
    second = fetch_market_data(
        tmp_path / "market-data-20260821T100100Z",
        start_date="2026-01-01",
        end_date="2026-02-01",
        symbols=("SPY",),
        sleep_seconds=0,
        fetcher=_fake_fetch,
    )

    assert first.archive_path == tmp_path / "market-data-20260821T100000Z.zip"
    assert second.archive_path == tmp_path / "market-data-20260821T100100Z.zip"
    assert first.archive_path != second.archive_path
    assert first.archive_path.is_file()
    assert second.archive_path.is_file()


def test_fresh_forecast_workflow_runs_end_to_end_without_network(tmp_path: Path) -> None:
    result = run_fresh_forecast(
        tmp_path / "forecast",
        start_date="2024-01-01",
        end_date="2026-01-02",
        sleep_seconds=0,
        fetcher=_fake_universe_fetch,
    )

    assert len(result.table) == 11
    assert set(result.table["symbol"]) == set(DEFAULT_SYMBOLS) - {"SPY"}
    assert {
        "1w_quadrant",
        "2w_quadrant",
        "4w_quadrant",
        "8w_quadrant",
    }.issubset(result.table.columns)
    assert result.forecast_table_path.is_file()
    assert result.forecast_long_path.is_file()
    assert result.market_data.archive_path.is_file()

    marker = json.loads(result.run_marker_path.read_text(encoding="utf-8"))
    assert marker["mode"] == "fresh_prospective_rrg_forecast"
    assert marker["horizons"] == [1, 2, 4, 8]
    assert marker["confidence_output"] is False
    assert marker["expected_return_output"] is False
    assert marker["trading_signal_output"] is False
    assert marker["performance_claim"] is False
    assert len(marker["manifest_sha256"]) == 64
    assert len(marker["market_data_zip_sha256"]) == 64
