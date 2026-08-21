from datetime import datetime, timezone
import hashlib
import json

import pandas as pd
import pytest

from rrg_rebuild.data import (
    build_yahoo_chart_url,
    parse_yahoo_chart_payload,
    resample_weekly_last,
)


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


def test_parser_requires_and_preserves_adjusted_close_and_provenance() -> None:
    raw = _payload()
    fetched = datetime(2026, 2, 1, 8, 30, tzinfo=timezone.utc)
    frame, provenance = parse_yahoo_chart_payload(
        raw,
        symbol="XLK",
        fetched_at=fetched,
        request_url="https://example.test/chart/XLK",
    )

    assert frame["adj_close"].tolist() == [99.5, 100.5, 101.5, 102.5]
    assert frame["close"].tolist() == [100.5, 101.5, 102.5, 103.5]
    assert provenance.price_field == "adj_close"
    assert provenance.raw_sha256 == hashlib.sha256(raw).hexdigest()
    assert provenance.fetched_at_utc == fetched.isoformat()


def test_parser_does_not_silently_fallback_when_adjusted_close_missing() -> None:
    obj = json.loads(_payload())
    obj["chart"]["result"][0]["indicators"].pop("adjclose")
    raw = json.dumps(obj).encode()

    with pytest.raises(ValueError, match="adjusted close is required"):
        parse_yahoo_chart_payload(raw, symbol="XLK")


def test_weekly_resample_selects_last_available_bar_and_records_actual_date() -> None:
    frame, _ = parse_yahoo_chart_payload(_payload(), symbol="XLK")
    weekly = resample_weekly_last(
        frame,
        as_of_utc=pd.Timestamp("2026-01-17T00:00:00Z"),
    )

    assert len(weekly) == 2
    assert weekly.loc[0, "source_timestamp"] == pd.Timestamp("2026-01-09T21:00:00Z")
    assert weekly.loc[1, "source_timestamp"] == pd.Timestamp("2026-01-15T21:00:00Z")
    assert weekly.loc[1, "week_end"] == pd.Timestamp("2026-01-16T23:59:59.999999999Z")
    assert (weekly["source_timestamp"] <= weekly["week_end"]).all()


def test_weekly_resample_excludes_week_not_complete_at_fetch_time() -> None:
    frame, _ = parse_yahoo_chart_payload(_payload(), symbol="XLK")
    weekly = resample_weekly_last(
        frame,
        as_of_utc=pd.Timestamp("2026-01-15T22:00:00Z"),
    )

    assert len(weekly) == 1
    assert weekly.loc[0, "week_end"] == pd.Timestamp("2026-01-09T23:59:59.999999999Z")
    assert weekly.loc[0, "source_timestamp"] == pd.Timestamp("2026-01-09T21:00:00Z")


def test_weekly_resample_rejects_naive_as_of() -> None:
    frame, _ = parse_yahoo_chart_payload(_payload(), symbol="XLK")
    with pytest.raises(ValueError, match="timezone-aware"):
        resample_weekly_last(frame, as_of_utc=pd.Timestamp("2026-01-17"))


def test_url_contract_is_daily_and_adjusted() -> None:
    url = build_yahoo_chart_url("XLK", 100, 200)
    assert "/XLK?" in url
    assert "interval=1d" in url
    assert "includeAdjustedClose=true" in url


def test_url_rejects_reverse_date_range() -> None:
    with pytest.raises(ValueError, match="start_epoch"):
        build_yahoo_chart_url("XLK", 200, 100)
