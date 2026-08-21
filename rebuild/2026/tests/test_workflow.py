from datetime import datetime, timezone
import json
from pathlib import Path
import zipfile

import pandas as pd

from rrg_rebuild.workflow import fetch_market_data


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
