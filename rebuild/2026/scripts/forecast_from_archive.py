#!/usr/bin/env python3
"""Generate prospective RRG-style forecasts from a versioned market-data ZIP.

Unlike the final-holdout runner, this is a deployment path. It accepts a fresh
archive, verifies its manifest/raw payload hashes, trains only on labels already
known by the latest completed common week, and emits forward forecasts. It does
not report accuracy or trading performance.
"""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
from io import BytesIO
import json
from pathlib import Path, PurePath
import zipfile

import pandas as pd

from rrg_rebuild.data import resample_weekly_last
from rrg_rebuild.dataset import align_common_weekly_history, build_sector_rrg_panel
from rrg_rebuild.deployment import DeploymentConfig, forecast_panel_latest, make_forecast_table

SYMBOLS = (
    "SPY",
    "XLB", "XLC", "XLE", "XLF", "XLI", "XLK",
    "XLP", "XLRE", "XLU", "XLV", "XLY",
)
SECTORS = tuple(s for s in SYMBOLS if s != "SPY")


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def normalized_member_map(zf: zipfile.ZipFile) -> dict[str, str]:
    return {name.replace("\\", "/"): name for name in zf.namelist()}


def basename_any_platform(value: str) -> str:
    return value.replace("\\", "/").rsplit("/", 1)[-1]


def canonical_daily(raw_csv: bytes, *, symbol: str) -> pd.DataFrame:
    daily = pd.read_csv(BytesIO(raw_csv))
    if {"source_timestamp_utc", "adjusted_close"}.issubset(daily.columns):
        daily = daily.rename(
            columns={
                "source_timestamp_utc": "timestamp",
                "adjusted_close": "adj_close",
            }
        )
    required = {"timestamp", "adj_close"}
    missing = required.difference(daily.columns)
    if missing:
        raise ValueError(f"{symbol}: processed CSV missing columns {sorted(missing)}")
    daily["timestamp"] = pd.to_datetime(daily["timestamp"], utc=True)
    daily["symbol"] = symbol
    return daily


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--input-zip", required=True, type=Path)
    parser.add_argument("--output-dir", required=True, type=Path)
    args = parser.parse_args()

    input_zip = args.input_zip.resolve()
    output_dir = args.output_dir.resolve()
    if not input_zip.is_file():
        raise FileNotFoundError(input_zip)
    if output_dir.exists():
        raise FileExistsError(f"refusing to overwrite existing forecast output: {output_dir}")

    input_hash = sha256_file(input_zip)

    with zipfile.ZipFile(input_zip, "r") as zf:
        members = normalized_member_map(zf)
        manifest_name = members.get("manifest.csv")
        if manifest_name is None:
            raise ValueError("market-data ZIP has no manifest.csv")
        manifest_bytes = zf.read(manifest_name)
        manifest_hash = sha256_bytes(manifest_bytes)
        manifest = pd.read_csv(BytesIO(manifest_bytes))

        required_manifest = {"symbol", "fetched_at_utc", "raw_sha256", "raw_path"}
        missing_manifest = required_manifest.difference(manifest.columns)
        if missing_manifest:
            raise ValueError(f"manifest missing columns: {sorted(missing_manifest)}")
        if set(manifest["symbol"]) != set(SYMBOLS):
            raise ValueError("manifest symbol universe does not match deployment universe")
        if manifest["symbol"].duplicated().any():
            raise ValueError("manifest contains duplicate symbols")

        weekly_by_symbol: dict[str, pd.DataFrame] = {}
        verified_raw = 0
        for symbol in SYMBOLS:
            row = manifest.loc[manifest["symbol"] == symbol].iloc[0]

            raw_basename = basename_any_platform(str(row["raw_path"]))
            raw_key = f"raw/yahoo/{raw_basename}"
            raw_name = members.get(raw_key)
            if raw_name is None:
                raise ValueError(f"missing raw Yahoo payload for {symbol}")
            raw_payload = zf.read(raw_name)
            if sha256_bytes(raw_payload) != str(row["raw_sha256"]):
                raise ValueError(f"raw Yahoo payload hash mismatch for {symbol}")
            verified_raw += 1

            daily_key = f"processed/daily/{symbol}.csv"
            daily_name = members.get(daily_key)
            if daily_name is None:
                raise ValueError(f"missing processed daily data for {symbol}")
            daily = canonical_daily(zf.read(daily_name), symbol=symbol)
            fetched_at = pd.Timestamp(row["fetched_at_utc"])
            if fetched_at.tzinfo is None:
                raise ValueError(f"{symbol}: fetched_at_utc must be timezone-aware")
            weekly_by_symbol[symbol] = resample_weekly_last(
                daily,
                as_of_utc=fetched_at,
            )

    prices, _, history = align_common_weekly_history(
        weekly_by_symbol,
        required_symbols=SYMBOLS,
    )
    panel = build_sector_rrg_panel(
        prices,
        benchmark_symbol="SPY",
        sector_symbols=SECTORS,
    )
    config = DeploymentConfig(horizons=(1, 2, 4, 8), lookback=20)
    forecasts = forecast_panel_latest(panel, sector_symbols=SECTORS, config=config)
    table = make_forecast_table(forecasts)

    output_dir.mkdir(parents=True)
    forecasts.to_csv(output_dir / "forecast_long.csv", index=False)
    table.to_csv(output_dir / "forecast_table.csv", index=False)

    marker = {
        "mode": "prospective_deployment_forecast",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "input_zip_sha256": input_hash,
        "manifest_sha256": manifest_hash,
        "raw_payload_hashes_verified": verified_raw,
        "common_history_start": str(history.common_start),
        "common_history_end": str(history.common_end),
        "decision_timestamp": str(forecasts["decision_timestamp"].iloc[0]),
        "horizons": list(config.horizons),
        "lookback": config.lookback,
        "model": "StandardScaler -> LinearRegression",
        "target": "future RRG-style RS-Ratio and RS-Momentum coordinates",
        "confidence_output": False,
        "performance_claim": False,
    }
    (output_dir / "RUN-MARKER.json").write_text(
        json.dumps(marker, indent=2), encoding="utf-8"
    )

    print(table.to_string(index=False))
    print(f"\nFORECAST WRITTEN: {output_dir}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
