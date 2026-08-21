#!/usr/bin/env python3
"""Run the pre-declared exploratory economic-value strategy backtest."""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
from io import BytesIO
import json
from pathlib import Path
import zipfile

import pandas as pd

from rrg_rebuild.data import resample_weekly_last
from rrg_rebuild.dataset import align_common_weekly_history, build_sector_rrg_panel
from rrg_rebuild.strategy import (
    StrategyBacktestConfig,
    fixed_equal_weight_targets,
    generate_historical_signal_panel,
    performance_summary,
    simulate_target_weight_strategy,
    top_k_equal_weights,
)

SYMBOLS = (
    "SPY",
    "XLB", "XLC", "XLE", "XLF", "XLI", "XLK",
    "XLP", "XLRE", "XLU", "XLV", "XLY",
)
SECTORS = tuple(s for s in SYMBOLS if s != "SPY")
CONTRACT_SHA256 = "a0611e717e1c5f11aec9f0c1d8948c108d844e72e2dd45d548b710fdbfd19cd2"


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
        daily = daily.rename(columns={"source_timestamp_utc": "timestamp", "adjusted_close": "adj_close"})
    required = {"timestamp", "adj_close"}
    missing = required.difference(daily.columns)
    if missing:
        raise ValueError(f"{symbol}: processed CSV missing columns {sorted(missing)}")
    daily["timestamp"] = pd.to_datetime(daily["timestamp"], utc=True)
    daily = daily.sort_values("timestamp")
    if daily["timestamp"].duplicated().any():
        raise ValueError(f"{symbol}: duplicate daily timestamps")
    if (daily["adj_close"] <= 0).any():
        raise ValueError(f"{symbol}: adjusted close must be positive")
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
        raise FileExistsError(f"refusing to overwrite existing backtest output: {output_dir}")

    input_hash = sha256_file(input_zip)
    weekly_by_symbol: dict[str, pd.DataFrame] = {}
    daily_series: dict[str, pd.Series] = {}

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
        if set(manifest["symbol"]) != set(SYMBOLS) or manifest["symbol"].duplicated().any():
            raise ValueError("manifest universe does not match frozen strategy universe")

        verified_raw = 0
        for symbol in SYMBOLS:
            row = manifest.loc[manifest["symbol"] == symbol].iloc[0]
            raw_name = members.get(f"raw/yahoo/{basename_any_platform(str(row['raw_path']))}")
            if raw_name is None:
                raise ValueError(f"missing raw Yahoo payload for {symbol}")
            raw_payload = zf.read(raw_name)
            if sha256_bytes(raw_payload) != str(row["raw_sha256"]):
                raise ValueError(f"raw Yahoo payload hash mismatch for {symbol}")
            verified_raw += 1

            daily_name = members.get(f"processed/daily/{symbol}.csv")
            if daily_name is None:
                raise ValueError(f"missing processed daily data for {symbol}")
            daily = canonical_daily(zf.read(daily_name), symbol=symbol)
            fetched_at = pd.Timestamp(row["fetched_at_utc"])
            if fetched_at.tzinfo is None:
                raise ValueError(f"{symbol}: fetched_at_utc must be timezone-aware")
            weekly_by_symbol[symbol] = resample_weekly_last(daily, as_of_utc=fetched_at)
            daily_series[symbol] = daily.set_index("timestamp")["adj_close"].rename(symbol)

    weekly_prices, _, history = align_common_weekly_history(
        weekly_by_symbol,
        required_symbols=SYMBOLS,
    )
    panel = build_sector_rrg_panel(
        weekly_prices,
        benchmark_symbol="SPY",
        sector_symbols=SECTORS,
    )
    common_daily = pd.concat([daily_series[s] for s in SYMBOLS], axis=1, join="inner").dropna().sort_index()
    if common_daily.empty:
        raise ValueError("no common daily execution history")

    config = StrategyBacktestConfig()
    signals = generate_historical_signal_panel(panel, sector_symbols=SECTORS, config=config)
    decisions = pd.DatetimeIndex(signals["decision_timestamp"].drop_duplicates().sort_values())

    target_sets = {
        "forecast_top3": top_k_equal_weights(
            signals,
            score_column="forecast_strength",
            sector_symbols=SECTORS,
            top_k=config.top_k,
        ),
        "persistence_top3": top_k_equal_weights(
            signals,
            score_column="current_strength",
            sector_symbols=SECTORS,
            top_k=config.top_k,
        ),
        "equal_weight_11_weekly": fixed_equal_weight_targets(decisions, symbols=SECTORS),
        "spy_buy_hold": fixed_equal_weight_targets(decisions, symbols=("SPY",)),
    }

    period_blocks = []
    summary_rows = []
    for strategy_name, targets in target_sets.items():
        periods = simulate_target_weight_strategy(
            targets,
            common_daily_prices=common_daily,
            transaction_cost_per_dollar=config.transaction_cost_per_dollar,
        )
        periods.insert(0, "strategy", strategy_name)
        period_blocks.append(periods)
        summary = performance_summary(periods)
        summary_rows.append({"strategy": strategy_name, **summary})

    all_periods = pd.concat(period_blocks, ignore_index=True)
    summary = pd.DataFrame.from_records(summary_rows)

    output_dir.mkdir(parents=True)
    signals.to_csv(output_dir / "historical_signals.csv", index=False)
    all_periods.to_csv(output_dir / "strategy_periods.csv", index=False)
    summary.to_csv(output_dir / "strategy_summary.csv", index=False)

    marker = {
        "mode": "exploratory_economic_value_backtest",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "strategy_contract_sha256": CONTRACT_SHA256,
        "input_zip_sha256": input_hash,
        "manifest_sha256": manifest_hash,
        "raw_payload_hashes_verified": verified_raw,
        "common_weekly_start": str(history.common_start),
        "common_weekly_end": str(history.common_end),
        "signal_first_decision": str(decisions[0]),
        "signal_last_decision": str(decisions[-1]),
        "horizon": config.horizon,
        "lookback": config.lookback,
        "min_training_rows": config.min_training_rows,
        "top_k": config.top_k,
        "transaction_cost_per_dollar": config.transaction_cost_per_dollar,
        "model": "StandardScaler -> LinearRegression",
        "parameter_search": False,
        "confirmatory_holdout": False,
    }
    (output_dir / "RUN-MARKER.json").write_text(json.dumps(marker, indent=2), encoding="utf-8")
    print(summary.to_string(index=False))
    print(f"\nBACKTEST WRITTEN: {output_dir}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
