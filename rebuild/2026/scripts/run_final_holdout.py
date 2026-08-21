#!/usr/bin/env python3
"""One-time guarded final holdout runner.

IMPORTANT:
- This script exits before opening market-data contents unless both deliberate
  authorization gates are supplied.
- It uses the frozen 2026 dataset hash and frozen protocol.
- It does not fetch new market data.
- It does not tune any model.
"""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
from io import BytesIO
import json
from pathlib import Path
import sys
import zipfile

import pandas as pd

from rrg_rebuild.data import resample_weekly_last
from rrg_rebuild.dataset import align_common_weekly_history, build_sector_rrg_panel
from rrg_rebuild.holdout import (
    FINAL_HOLDOUT_PROTOCOL_ID,
    evaluate_final_holdout_for_series,
    frozen_protocol,
    require_final_holdout_authorization,
)


EXPECTED_INPUT_ZIP_SHA256 = (
    "3838be08d18b238675ea02b9addce983799820abdf74eb40ac0e4ce8481b82bf"
)
EXPECTED_MANIFEST_SHA256 = (
    "4ec4b56f8f14f1c2b21bd60d9e440fd90fa108f83ed1e2c0959cc23ecd6b0c35"
)
SYMBOLS = (
    "SPY",
    "XLB", "XLC", "XLE", "XLF", "XLI", "XLK",
    "XLP", "XLRE", "XLU", "XLV", "XLY",
)
SECTORS = tuple(s for s in SYMBOLS if s != "SPY")


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def normalized_member_map(zf: zipfile.ZipFile) -> dict[str, str]:
    return {name.replace("\\", "/"): name for name in zf.namelist()}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--input-zip", required=True, type=Path)
    parser.add_argument("--output-dir", required=True, type=Path)
    parser.add_argument("--open-final-holdout", action="store_true")
    parser.add_argument("--authorization-token")
    args = parser.parse_args()

    # Deliberate guard BEFORE opening or hashing the market-data archive.
    require_final_holdout_authorization(
        open_final_holdout=args.open_final_holdout,
        authorization_token=args.authorization_token,
    )

    input_zip = args.input_zip.resolve()
    output_dir = args.output_dir.resolve()

    if not input_zip.is_file():
        raise FileNotFoundError(input_zip)
    if output_dir.exists():
        raise FileExistsError(
            f"refusing to overwrite existing final-holdout output: {output_dir}"
        )

    actual_zip_hash = sha256_file(input_zip)
    if actual_zip_hash != EXPECTED_INPUT_ZIP_SHA256:
        raise ValueError(
            "input ZIP hash does not match the frozen 2026 market dataset"
        )

    protocol = frozen_protocol()

    with zipfile.ZipFile(input_zip, "r") as zf:
        members = normalized_member_map(zf)
        manifest_name = members.get("manifest.csv")
        if manifest_name is None:
            raise ValueError("frozen dataset has no manifest.csv")

        manifest_bytes = zf.read(manifest_name)
        manifest_hash = hashlib.sha256(manifest_bytes).hexdigest()
        if manifest_hash != EXPECTED_MANIFEST_SHA256:
            raise ValueError("manifest hash does not match frozen protocol")

        manifest = pd.read_csv(BytesIO(manifest_bytes))
        if set(manifest["symbol"]) != set(SYMBOLS):
            raise ValueError("manifest symbol universe does not match frozen protocol")

        as_of = pd.to_datetime(manifest["fetched_at_utc"], utc=True).max()

        weekly_by_symbol: dict[str, pd.DataFrame] = {}
        for symbol in SYMBOLS:
            key = f"processed/daily/{symbol}.csv"
            actual_name = members.get(key)
            if actual_name is None:
                raise ValueError(f"missing frozen daily data for {symbol}")

            daily = pd.read_csv(BytesIO(zf.read(actual_name)))
            if {"source_timestamp_utc", "adjusted_close"}.issubset(daily.columns):
                daily = daily.rename(
                    columns={
                        "source_timestamp_utc": "timestamp",
                        "adjusted_close": "adj_close",
                    }
                )
                daily["symbol"] = symbol
            daily["timestamp"] = pd.to_datetime(daily["timestamp"], utc=True)
            weekly_by_symbol[symbol] = resample_weekly_last(
                daily,
                as_of_utc=as_of,
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

    prediction_blocks = []
    summary_blocks = []
    for symbol in SECTORS:
        rrg = (
            panel.loc[panel["symbol"] == symbol]
            .set_index("timestamp")
            .sort_index()
        )
        for horizon in protocol.horizons:
            predictions, summary = evaluate_final_holdout_for_series(
                rrg,
                horizon=horizon,
                protocol=protocol,
                open_final_holdout=True,
                authorization_token=args.authorization_token,
            )
            predictions.insert(0, "symbol", symbol)
            summary.insert(0, "symbol", symbol)
            prediction_blocks.append(predictions)
            summary_blocks.append(summary)

    predictions = pd.concat(prediction_blocks, ignore_index=True)
    by_sector = pd.concat(summary_blocks, ignore_index=True)

    aggregate = (
        by_sector.groupby(["horizon", "model"], as_index=False)
        .apply(
            lambda g: pd.Series(
                {
                    "n": int(g["n"].sum()),
                    "weighted_mean_coordinate_distance": float(
                        (g["mean_coordinate_distance"] * g["n"]).sum()
                        / g["n"].sum()
                    ),
                    "weighted_quadrant_accuracy": float(
                        (g["quadrant_accuracy"] * g["n"]).sum()
                        / g["n"].sum()
                    ),
                    "weighted_macro_f1": float(
                        (g["macro_f1"] * g["n"]).sum()
                        / g["n"].sum()
                    ),
                }
            ),
            include_groups=False,
        )
        .reset_index(drop=True)
    )

    output_dir.mkdir(parents=True)
    predictions.to_csv(output_dir / "final_holdout_predictions.csv", index=False)
    by_sector.to_csv(output_dir / "final_holdout_by_sector.csv", index=False)
    aggregate.to_csv(output_dir / "final_holdout_summary.csv", index=False)

    marker = {
        "protocol_id": FINAL_HOLDOUT_PROTOCOL_ID,
        "input_zip_sha256": actual_zip_hash,
        "manifest_sha256": EXPECTED_MANIFEST_SHA256,
        "completed_at_utc": datetime.now(timezone.utc).isoformat(),
        "common_history_start": str(history.common_start),
        "common_history_end": str(history.common_end),
        "holdout_target_start": str(protocol.target_start),
        "holdout_target_end": str(protocol.target_end),
        "horizons": list(protocol.horizons),
        "lookback": protocol.lookback,
        "model": "StandardScaler -> LinearRegression",
        "refit_policy": "fixed_at_first_holdout_decision_per_symbol_horizon",
        "holdout_reuse_allowed": False,
    }
    (output_dir / "RUN-MARKER.json").write_text(
        json.dumps(marker, indent=2),
        encoding="utf-8",
    )

    print(f"FINAL HOLDOUT COMPLETED ONCE: {output_dir}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except PermissionError as exc:
        print(f"REFUSED: {exc}", file=sys.stderr)
        raise SystemExit(4)
