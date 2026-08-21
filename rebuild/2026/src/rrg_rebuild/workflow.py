from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime, timedelta, timezone
import hashlib
import json
from pathlib import Path
import time
from typing import Callable, Sequence
from urllib.request import Request, urlopen
import zipfile

import pandas as pd

from .data import build_yahoo_chart_url, parse_yahoo_chart_payload, resample_weekly_last
from .dataset import CommonHistory, align_common_weekly_history, build_sector_rrg_panel
from .deployment import DeploymentConfig, forecast_panel_latest, make_forecast_table

DEFAULT_SYMBOLS: tuple[str, ...] = (
    "SPY",
    "XLB", "XLC", "XLE", "XLF", "XLI", "XLK",
    "XLP", "XLRE", "XLU", "XLV", "XLY",
)
DEFAULT_BENCHMARK = "SPY"

PayloadFetcher = Callable[[str, int, int, float], tuple[bytes, str, datetime]]


@dataclass(frozen=True)
class MarketDataFetch:
    root: Path
    manifest_path: Path
    archive_path: Path
    weekly_by_symbol: dict[str, pd.DataFrame]


@dataclass(frozen=True)
class FreshForecastRun:
    output_dir: Path
    market_data: MarketDataFetch
    forecast_long_path: Path
    forecast_table_path: Path
    run_marker_path: Path
    table: pd.DataFrame
    history: CommonHistory


def _sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def _sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def _date_to_epoch(value: str) -> int:
    dt = datetime.strptime(value, "%Y-%m-%d").replace(tzinfo=timezone.utc)
    return int(dt.timestamp())


def default_end_date() -> str:
    """Return tomorrow in UTC because Yahoo period2 is exclusive."""
    return (datetime.now(timezone.utc).date() + timedelta(days=1)).isoformat()


def _network_fetch_payload(
    symbol: str,
    start_epoch: int,
    end_epoch: int,
    timeout_seconds: float,
) -> tuple[bytes, str, datetime]:
    url = build_yahoo_chart_url(symbol, start_epoch, end_epoch)
    request = Request(
        url,
        headers={"User-Agent": "RRG-Research-Reconstruction/0.1"},
    )
    with urlopen(request, timeout=timeout_seconds) as response:
        raw = response.read()
    fetched_at = datetime.now(timezone.utc)
    return raw, url, fetched_at


def archive_market_data(root: Path, archive_path: Path) -> Path:
    """Create a deterministic-layout ZIP expected by the archive forecast runner."""
    root = Path(root)
    archive_path = Path(archive_path)
    if not root.is_dir():
        raise FileNotFoundError(root)
    archive_path.parent.mkdir(parents=True, exist_ok=True)
    if archive_path.exists():
        raise FileExistsError(archive_path)

    files = sorted(path for path in root.rglob("*") if path.is_file())
    if not files:
        raise ValueError("market-data directory is empty")

    with zipfile.ZipFile(archive_path, "w", compression=zipfile.ZIP_DEFLATED) as zf:
        for path in files:
            zf.write(path, arcname=path.relative_to(root).as_posix())
    return archive_path


def fetch_market_data(
    root: Path,
    *,
    start_date: str = "2018-01-01",
    end_date: str | None = None,
    symbols: Sequence[str] = DEFAULT_SYMBOLS,
    sleep_seconds: float = 0.5,
    timeout_seconds: float = 30.0,
    fetcher: PayloadFetcher | None = None,
) -> MarketDataFetch:
    """Fetch versioned adjusted-close data, provenance, and a portable archive."""
    root = Path(root)
    symbols = tuple(symbol.upper().strip() for symbol in symbols)
    if not symbols or any(not symbol for symbol in symbols):
        raise ValueError("symbols must be non-empty")
    if len(set(symbols)) != len(symbols):
        raise ValueError("symbols must be unique")

    if root.exists() and any(root.iterdir()):
        raise FileExistsError(f"refusing to overwrite non-empty market-data directory: {root}")
    root.mkdir(parents=True, exist_ok=True)

    start_epoch = _date_to_epoch(start_date)
    resolved_end = end_date or default_end_date()
    end_epoch = _date_to_epoch(resolved_end)
    if start_epoch >= end_epoch:
        raise ValueError("start_date must be earlier than end_date")

    payload_fetcher = fetcher or _network_fetch_payload
    raw_dir = root / "raw" / "yahoo"
    daily_dir = root / "processed" / "daily"
    raw_dir.mkdir(parents=True, exist_ok=True)
    daily_dir.mkdir(parents=True, exist_ok=True)

    manifest_rows: list[dict[str, object]] = []
    weekly_by_symbol: dict[str, pd.DataFrame] = {}

    for index, symbol in enumerate(symbols):
        raw, url, fetched_at = payload_fetcher(
            symbol,
            start_epoch,
            end_epoch,
            timeout_seconds,
        )
        if fetched_at.tzinfo is None:
            raise ValueError("payload fetcher must return timezone-aware fetched_at")
        fetched_at = fetched_at.astimezone(timezone.utc)

        daily, provenance = parse_yahoo_chart_payload(
            raw,
            symbol=symbol,
            fetched_at=fetched_at,
            request_url=url,
        )
        digest = provenance.raw_sha256
        raw_rel = Path("raw") / "yahoo" / f"{symbol}_{digest[:12]}.json"
        daily_rel = Path("processed") / "daily" / f"{symbol}.csv"
        (root / raw_rel).write_bytes(raw)

        public_daily = daily[["timestamp", "adj_close"]].rename(
            columns={
                "timestamp": "source_timestamp_utc",
                "adj_close": "adjusted_close",
            }
        )
        public_daily.to_csv(root / daily_rel, index=False)

        weekly_by_symbol[symbol] = resample_weekly_last(
            daily,
            as_of_utc=pd.Timestamp(fetched_at),
        )

        manifest_rows.append(
            {
                "symbol": symbol,
                "fetched_at_utc": provenance.fetched_at_utc,
                "source_url": provenance.request_url,
                "raw_sha256": digest,
                "raw_path": raw_rel.as_posix(),
                "daily_csv_path": daily_rel.as_posix(),
                "row_count": int(len(public_daily)),
                "first_timestamp_utc": str(public_daily.iloc[0]["source_timestamp_utc"]),
                "last_timestamp_utc": str(public_daily.iloc[-1]["source_timestamp_utc"]),
            }
        )

        if index < len(symbols) - 1 and sleep_seconds > 0:
            time.sleep(sleep_seconds)

    manifest_path = root / "manifest.csv"
    pd.DataFrame(manifest_rows).to_csv(manifest_path, index=False)
    archive_path = archive_market_data(root, root.with_suffix(".zip"))

    return MarketDataFetch(
        root=root,
        manifest_path=manifest_path,
        archive_path=archive_path,
        weekly_by_symbol=weekly_by_symbol,
    )


def run_fresh_forecast(
    output_dir: Path,
    *,
    start_date: str = "2018-01-01",
    end_date: str | None = None,
    sleep_seconds: float = 0.5,
    timeout_seconds: float = 30.0,
    symbols: Sequence[str] = DEFAULT_SYMBOLS,
    benchmark_symbol: str = DEFAULT_BENCHMARK,
    deployment_config: DeploymentConfig | None = None,
    fetcher: PayloadFetcher | None = None,
) -> FreshForecastRun:
    """Fetch fresh data and produce the validated-model prospective forecast.

    This is a deployment/research operation. It does not estimate expected
    returns, produce calibrated probabilities, or claim a profitable strategy.
    """
    output_dir = Path(output_dir)
    if output_dir.exists() and any(output_dir.iterdir()):
        raise FileExistsError(f"refusing to overwrite non-empty output directory: {output_dir}")
    output_dir.mkdir(parents=True, exist_ok=True)

    symbols = tuple(symbol.upper().strip() for symbol in symbols)
    if benchmark_symbol not in symbols:
        raise ValueError("benchmark_symbol must be included in symbols")
    sectors = tuple(symbol for symbol in symbols if symbol != benchmark_symbol)
    if not sectors:
        raise ValueError("at least one non-benchmark sector is required")

    market_data = fetch_market_data(
        output_dir / "market-data",
        start_date=start_date,
        end_date=end_date,
        symbols=symbols,
        sleep_seconds=sleep_seconds,
        timeout_seconds=timeout_seconds,
        fetcher=fetcher,
    )

    prices, _, history = align_common_weekly_history(
        market_data.weekly_by_symbol,
        required_symbols=symbols,
    )
    panel = build_sector_rrg_panel(
        prices,
        benchmark_symbol=benchmark_symbol,
        sector_symbols=sectors,
    )
    config = deployment_config or DeploymentConfig()
    forecasts = forecast_panel_latest(
        panel,
        sector_symbols=sectors,
        config=config,
    )
    table = make_forecast_table(forecasts)

    forecast_long_path = output_dir / "forecast_long.csv"
    forecast_table_path = output_dir / "forecast_table.csv"
    run_marker_path = output_dir / "RUN-MARKER.json"
    forecasts.to_csv(forecast_long_path, index=False)
    table.to_csv(forecast_table_path, index=False)

    manifest_hash = _sha256_file(market_data.manifest_path)
    archive_hash = _sha256_file(market_data.archive_path)
    marker = {
        "mode": "fresh_prospective_rrg_forecast",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "benchmark": benchmark_symbol,
        "symbols": list(symbols),
        "common_history_start": str(history.common_start),
        "common_history_end": str(history.common_end),
        "decision_timestamp": str(forecasts["decision_timestamp"].iloc[0]),
        "horizons": list(config.horizons),
        "lookback": config.lookback,
        "model": "StandardScaler -> LinearRegression",
        "target": "future RRG-style RS-Ratio and RS-Momentum coordinates",
        "manifest_sha256": manifest_hash,
        "market_data_zip_sha256": archive_hash,
        "raw_payload_hashes_recorded": True,
        "confidence_output": False,
        "expected_return_output": False,
        "trading_signal_output": False,
        "performance_claim": False,
    }
    run_marker_path.write_text(json.dumps(marker, indent=2), encoding="utf-8")

    return FreshForecastRun(
        output_dir=output_dir,
        market_data=market_data,
        forecast_long_path=forecast_long_path,
        forecast_table_path=forecast_table_path,
        run_marker_path=run_marker_path,
        table=table,
        history=history,
    )