#!/usr/bin/env python3
from __future__ import annotations

import argparse
import csv
from dataclasses import dataclass
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import time
from typing import Iterable
from urllib.parse import urlencode
from urllib.request import Request, urlopen

UNIVERSE = ["SPY", "XLB", "XLC", "XLE", "XLF", "XLI", "XLK", "XLP", "XLRE", "XLU", "XLV", "XLY"]
BASE_URL = "https://query1.finance.yahoo.com/v8/finance/chart/{symbol}"
USER_AGENT = "RRG-Research-Reconstruction/2026-rebuild"


@dataclass(frozen=True)
class FetchRecord:
    symbol: str
    fetched_at_utc: str
    source_url: str
    raw_sha256: str
    raw_path: str
    daily_csv_path: str
    row_count: int
    first_timestamp_utc: str
    last_timestamp_utc: str


def parse_date_utc(value: str) -> int:
    dt = datetime.strptime(value, "%Y-%m-%d").replace(tzinfo=timezone.utc)
    return int(dt.timestamp())


def build_url(symbol: str, start: str, end: str) -> str:
    params = {
        "period1": parse_date_utc(start),
        "period2": parse_date_utc(end),
        "interval": "1d",
        "events": "history",
        "includeAdjustedClose": "true",
    }
    return f"{BASE_URL.format(symbol=symbol)}?{urlencode(params)}"


def fetch_bytes(url: str, timeout: int = 30) -> bytes:
    req = Request(url, headers={"User-Agent": USER_AGENT, "Accept": "application/json"})
    with urlopen(req, timeout=timeout) as response:
        if getattr(response, "status", 200) != 200:
            raise RuntimeError(f"HTTP {response.status} for {url}")
        return response.read()


def parse_adjusted_close(raw: bytes, symbol: str) -> list[tuple[int, float]]:
    payload = json.loads(raw)
    chart = payload.get("chart", {})
    error = chart.get("error")
    if error:
        raise RuntimeError(f"Yahoo returned error for {symbol}: {error}")

    results = chart.get("result") or []
    if len(results) != 1:
        raise RuntimeError(f"Expected one chart result for {symbol}; got {len(results)}")

    result = results[0]
    timestamps = result.get("timestamp") or []
    indicators = result.get("indicators") or {}
    adj_blocks = indicators.get("adjclose") or []
    if not adj_blocks or "adjclose" not in adj_blocks[0]:
        raise RuntimeError(
            f"Adjusted close missing for {symbol}. The rebuild does not silently fall back to raw close."
        )

    adjusted = adj_blocks[0]["adjclose"]
    if len(timestamps) != len(adjusted):
        raise RuntimeError(f"Timestamp/adjusted-close length mismatch for {symbol}")

    rows: list[tuple[int, float]] = []
    for ts, price in zip(timestamps, adjusted):
        if ts is None or price is None:
            continue
        price_f = float(price)
        if price_f <= 0:
            raise RuntimeError(f"Non-positive adjusted close for {symbol} at {ts}: {price_f}")
        rows.append((int(ts), price_f))

    if not rows:
        raise RuntimeError(f"No usable adjusted-close rows for {symbol}")

    rows.sort(key=lambda x: x[0])
    return rows


def iso_utc(ts: int) -> str:
    return datetime.fromtimestamp(ts, tz=timezone.utc).isoformat()


def write_daily_csv(path: Path, rows: Iterable[tuple[int, float]]) -> tuple[int, str, str]:
    rows = list(rows)
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", newline="", encoding="utf-8") as f:
        writer = csv.writer(f)
        writer.writerow(["source_timestamp_utc", "adjusted_close"])
        for ts, price in rows:
            writer.writerow([iso_utc(ts), f"{price:.10f}"])
    return len(rows), iso_utc(rows[0][0]), iso_utc(rows[-1][0])


def fetch_symbol(symbol: str, start: str, end: str, output_root: Path) -> FetchRecord:
    symbol = symbol.upper().strip()
    url = build_url(symbol, start, end)
    raw = fetch_bytes(url)
    digest = hashlib.sha256(raw).hexdigest()
    fetched_at = datetime.now(timezone.utc).isoformat()

    raw_dir = output_root / "raw" / "yahoo"
    daily_dir = output_root / "processed" / "daily"
    raw_dir.mkdir(parents=True, exist_ok=True)
    daily_dir.mkdir(parents=True, exist_ok=True)

    stamp = fetched_at.replace(":", "").replace("+00:00", "Z").replace(".", "-")
    raw_path = raw_dir / f"{symbol}_{stamp}_{digest[:12]}.json"
    raw_path.write_bytes(raw)

    rows = parse_adjusted_close(raw, symbol)
    csv_path = daily_dir / f"{symbol}.csv"
    count, first_ts, last_ts = write_daily_csv(csv_path, rows)

    return FetchRecord(
        symbol=symbol,
        fetched_at_utc=fetched_at,
        source_url=url,
        raw_sha256=digest,
        raw_path=str(raw_path),
        daily_csv_path=str(csv_path),
        row_count=count,
        first_timestamp_utc=first_ts,
        last_timestamp_utc=last_ts,
    )


def write_manifest(path: Path, records: list[FetchRecord]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    fields = list(FetchRecord.__dataclass_fields__.keys())
    with path.open("w", newline="", encoding="utf-8") as f:
        writer = csv.DictWriter(f, fieldnames=fields)
        writer.writeheader()
        for record in records:
            writer.writerow(record.__dict__)


def main() -> None:
    parser = argparse.ArgumentParser(description="Fetch fresh, versioned daily market data for the 2026 RRG rebuild.")
    parser.add_argument("--start", default="2018-01-01", help="UTC start date (YYYY-MM-DD)")
    parser.add_argument("--end", default=datetime.now(timezone.utc).strftime("%Y-%m-%d"), help="UTC end date, exclusive")
    parser.add_argument("--output", type=Path, default=Path("data/2026-rebuild"))
    parser.add_argument("--symbols", nargs="*", default=UNIVERSE)
    parser.add_argument("--sleep", type=float, default=0.5, help="Delay between requests in seconds")
    args = parser.parse_args()

    records: list[FetchRecord] = []
    failures: list[tuple[str, str]] = []

    for index, symbol in enumerate(args.symbols):
        try:
            record = fetch_symbol(symbol, args.start, args.end, args.output)
            records.append(record)
            print(f"{symbol}: {record.row_count} rows, {record.first_timestamp_utc} -> {record.last_timestamp_utc}")
        except Exception as exc:  # preserve partial progress and report failures explicitly
            failures.append((symbol, str(exc)))
            print(f"{symbol}: FAILED: {exc}")
        if index < len(args.symbols) - 1:
            time.sleep(max(args.sleep, 0.0))

    manifest = args.output / "manifest.csv"
    write_manifest(manifest, records)
    print(f"\nManifest: {manifest}")
    print(f"Successful symbols: {len(records)}/{len(args.symbols)}")

    if failures:
        print("\nFailures:")
        for symbol, error in failures:
            print(f"- {symbol}: {error}")
        raise SystemExit(1)


if __name__ == "__main__":
    main()
