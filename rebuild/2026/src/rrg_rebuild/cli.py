from __future__ import annotations

import argparse
from datetime import datetime, timezone
from pathlib import Path
from typing import Sequence

from . import __version__
from .workflow import fetch_market_data, run_fresh_forecast


def _default_output_dir(prefix: str) -> Path:
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    return Path("outputs") / f"{prefix}-{stamp}"


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="rrg-research",
        description="Reproducible sector RRG-style forecasting research toolkit.",
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    subparsers.add_parser("version", help="Print the package version.")

    fetch = subparsers.add_parser(
        "fetch",
        help="Fetch and archive fresh adjusted-close market data with provenance.",
    )
    fetch.add_argument("--output-dir", type=Path)
    fetch.add_argument("--start", default="2018-01-01", help="UTC start date (YYYY-MM-DD).")
    fetch.add_argument("--end", help="UTC exclusive end date (YYYY-MM-DD). Defaults to tomorrow UTC.")
    fetch.add_argument("--sleep", type=float, default=0.5, help="Delay between provider requests.")
    fetch.add_argument("--timeout", type=float, default=30.0, help="Per-request timeout in seconds.")

    forecast = subparsers.add_parser(
        "forecast",
        help="Fetch fresh data and generate 1/2/4/8-week RRG-style forecasts.",
    )
    forecast.add_argument("--output-dir", type=Path)
    forecast.add_argument("--start", default="2018-01-01", help="UTC start date (YYYY-MM-DD).")
    forecast.add_argument("--end", help="UTC exclusive end date (YYYY-MM-DD). Defaults to tomorrow UTC.")
    forecast.add_argument("--sleep", type=float, default=0.5, help="Delay between provider requests.")
    forecast.add_argument("--timeout", type=float, default=30.0, help="Per-request timeout in seconds.")

    return parser


def main(argv: Sequence[str] | None = None) -> int:
    args = build_parser().parse_args(argv)

    if args.command == "version":
        print(__version__)
        return 0

    if args.command == "fetch":
        output_dir = args.output_dir or _default_output_dir("market-data")
        result = fetch_market_data(
            output_dir,
            start_date=args.start,
            end_date=args.end,
            sleep_seconds=args.sleep,
            timeout_seconds=args.timeout,
        )
        print(f"Manifest: {result.manifest_path}")
        print(f"Archive:  {result.archive_path}")
        return 0

    if args.command == "forecast":
        output_dir = args.output_dir or _default_output_dir("forecast")
        result = run_fresh_forecast(
            output_dir,
            start_date=args.start,
            end_date=args.end,
            sleep_seconds=args.sleep,
            timeout_seconds=args.timeout,
        )
        print(result.table.to_string(index=False))
        print(f"\nForecast table: {result.forecast_table_path}")
        print(f"Forecast detail: {result.forecast_long_path}")
        print(f"Run marker:      {result.run_marker_path}")
        print(f"Market archive:  {result.market_data.archive_path}")
        print("\nResearch output only: this command does not estimate expected returns or issue trading signals.")
        return 0

    raise AssertionError(f"unhandled command: {args.command}")


if __name__ == "__main__":
    raise SystemExit(main())
