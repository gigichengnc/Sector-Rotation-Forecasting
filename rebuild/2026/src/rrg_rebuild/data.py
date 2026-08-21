from __future__ import annotations

from dataclasses import asdict, dataclass
from datetime import datetime, timezone
import hashlib
import json
from typing import Any
from urllib.parse import urlencode
from urllib.request import Request, urlopen

import pandas as pd


@dataclass(frozen=True)
class DataProvenance:
    symbol: str
    provider: str
    interval: str
    price_field: str
    fetched_at_utc: str
    raw_sha256: str
    request_url: str | None = None

    def to_dict(self) -> dict[str, str | None]:
        return asdict(self)


def _sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def parse_yahoo_chart_payload(raw_payload: bytes, *, symbol: str, fetched_at: datetime | None = None, request_url: str | None = None) -> tuple[pd.DataFrame, DataProvenance]:
    fetched_at = fetched_at or datetime.now(timezone.utc)
    if fetched_at.tzinfo is None:
        raise ValueError("fetched_at must be timezone-aware")
    try:
        payload: dict[str, Any] = json.loads(raw_payload)
    except json.JSONDecodeError as exc:
        raise ValueError("invalid JSON market-data payload") from exc
    chart = payload.get("chart")
    if not isinstance(chart, dict):
        raise ValueError("payload has no chart object")
    if chart.get("error") is not None:
        raise ValueError(f"provider returned chart error: {chart['error']}")
    results = chart.get("result")
    if not isinstance(results, list) or not results:
        raise ValueError("payload contains no chart result")
    result = results[0]
    timestamps = result.get("timestamp")
    indicators = result.get("indicators", {})
    quotes = indicators.get("quote") or []
    adjclose_groups = indicators.get("adjclose") or []
    if not timestamps or not quotes:
        raise ValueError("payload has no timestamp/quote observations")
    if not adjclose_groups or "adjclose" not in adjclose_groups[0]:
        raise ValueError("adjusted close is required by the 2026 data contract")
    quote = quotes[0]
    adjusted = adjclose_groups[0]["adjclose"]
    if len(adjusted) != len(timestamps):
        raise ValueError("adjusted-close length does not match timestamps")
    def field(name: str) -> list[Any]:
        values = quote.get(name)
        if values is None:
            return [None] * len(timestamps)
        if len(values) != len(timestamps):
            raise ValueError(f"{name} length does not match timestamps")
        return values
    frame = pd.DataFrame({
        "timestamp": pd.to_datetime(timestamps, unit="s", utc=True),
        "open": field("open"), "high": field("high"), "low": field("low"),
        "close": field("close"), "adj_close": adjusted, "volume": field("volume"),
    })
    frame = frame.dropna(subset=["timestamp", "adj_close"]).copy()
    frame = frame.sort_values("timestamp")
    if frame["timestamp"].duplicated().any():
        raise ValueError("duplicate market-data timestamps")
    if (frame["adj_close"] <= 0).any():
        raise ValueError("adjusted close must be strictly positive")
    frame["symbol"] = symbol
    provenance = DataProvenance(symbol=symbol, provider="Yahoo Finance chart endpoint", interval="1d", price_field="adj_close", fetched_at_utc=fetched_at.astimezone(timezone.utc).isoformat(), raw_sha256=_sha256(raw_payload), request_url=request_url)
    return frame.reset_index(drop=True), provenance


def _as_utc_timestamp(value: datetime | pd.Timestamp) -> pd.Timestamp:
    timestamp = pd.Timestamp(value)
    if timestamp.tzinfo is None:
        raise ValueError("as_of_utc must be timezone-aware")
    return timestamp.tz_convert("UTC")


def resample_weekly_last(frame: pd.DataFrame, *, as_of_utc: datetime | pd.Timestamp) -> pd.DataFrame:
    required = {"timestamp", "symbol", "adj_close"}
    missing = required.difference(frame.columns)
    if missing:
        raise ValueError(f"missing required columns: {sorted(missing)}")
    as_of = _as_utc_timestamp(as_of_utc)
    work = frame.copy()
    work["timestamp"] = pd.to_datetime(work["timestamp"], utc=True)
    work = work.sort_values("timestamp")
    if work["timestamp"].duplicated().any():
        raise ValueError("duplicate market-data timestamps")
    if (work["timestamp"] > as_of).any():
        raise ValueError("market-data frame contains observations after as_of_utc")
    indexed = work.set_index("timestamp")
    rows: list[pd.Series] = []
    labels: list[pd.Timestamp] = []
    for week_end, group in indexed.groupby(pd.Grouper(freq="W-FRI")):
        if group.empty:
            continue
        cutoff = pd.Timestamp(week_end) + pd.Timedelta(days=1) - pd.Timedelta(nanoseconds=1)
        if cutoff > as_of:
            continue
        last_timestamp = group.index[-1]
        last_row = group.iloc[-1].copy()
        last_row["source_timestamp"] = last_timestamp
        rows.append(last_row)
        labels.append(cutoff)
    if not rows:
        return pd.DataFrame(columns=[*work.columns, "source_timestamp", "week_end"])
    weekly = pd.DataFrame(rows).reset_index(drop=True)
    weekly["week_end"] = pd.to_datetime(labels, utc=True)
    weekly["source_timestamp"] = pd.to_datetime(weekly["source_timestamp"], utc=True)
    if (weekly["source_timestamp"] > weekly["week_end"]).any():
        raise AssertionError("weekly resampling selected an observation from the future")
    if (weekly["week_end"] > as_of).any():
        raise AssertionError("weekly resampling emitted an incomplete week")
    return weekly


def build_yahoo_chart_url(symbol: str, start_epoch: int, end_epoch: int) -> str:
    if not symbol:
        raise ValueError("symbol cannot be empty")
    if start_epoch >= end_epoch:
        raise ValueError("start_epoch must be earlier than end_epoch")
    params = urlencode({"period1": int(start_epoch), "period2": int(end_epoch), "interval": "1d", "events": "div,splits", "includeAdjustedClose": "true"})
    return f"https://query1.finance.yahoo.com/v8/finance/chart/{symbol}?{params}"


def fetch_yahoo_chart(symbol: str, *, start_epoch: int, end_epoch: int, timeout_seconds: float = 30.0) -> tuple[pd.DataFrame, DataProvenance]:
    url = build_yahoo_chart_url(symbol, start_epoch, end_epoch)
    request = Request(url, headers={"User-Agent": "RRG-Rebuild/0.1 research"})
    with urlopen(request, timeout=timeout_seconds) as response:
        raw = response.read()
    return parse_yahoo_chart_payload(raw, symbol=symbol, request_url=url)
