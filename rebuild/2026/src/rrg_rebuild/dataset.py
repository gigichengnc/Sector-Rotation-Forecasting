from __future__ import annotations

from collections.abc import Mapping, Sequence
from dataclasses import dataclass

import pandas as pd

from .rrg import RRGConfig, calculate_rrg


@dataclass(frozen=True)
class CommonHistory:
    common_start: pd.Timestamp
    common_end: pd.Timestamp
    symbols: tuple[str, ...]
    observations: int


def _canonical_weekly(frame: pd.DataFrame, *, symbol: str) -> pd.DataFrame:
    required = {"week_end", "source_timestamp", "adj_close"}
    missing = required.difference(frame.columns)
    if missing:
        raise ValueError(f"{symbol}: missing required columns: {sorted(missing)}")

    out = frame.copy()
    out["week_end"] = pd.to_datetime(out["week_end"], utc=True)
    out["source_timestamp"] = pd.to_datetime(out["source_timestamp"], utc=True)
    out = out.sort_values("week_end")

    if out["week_end"].duplicated().any():
        raise ValueError(f"{symbol}: duplicate week_end values")
    if (out["source_timestamp"] > out["week_end"]).any():
        raise ValueError(f"{symbol}: source observation occurs after weekly cutoff")
    if (out["adj_close"] <= 0).any():
        raise ValueError(f"{symbol}: adjusted close must be strictly positive")
    return out


def align_common_weekly_history(
    weekly_by_symbol: Mapping[str, pd.DataFrame],
    *,
    required_symbols: Sequence[str],
) -> tuple[pd.DataFrame, pd.DataFrame, CommonHistory]:
    """Align all required symbols on weeks for which every symbol has data."""
    symbols = tuple(required_symbols)
    if not symbols:
        raise ValueError("required_symbols cannot be empty")
    if len(set(symbols)) != len(symbols):
        raise ValueError("required_symbols contains duplicates")

    missing_symbols = [s for s in symbols if s not in weekly_by_symbol]
    if missing_symbols:
        raise ValueError(f"missing required symbols: {missing_symbols}")

    price_series: list[pd.Series] = []
    source_series: list[pd.Series] = []
    for symbol in symbols:
        frame = _canonical_weekly(weekly_by_symbol[symbol], symbol=symbol)
        indexed = frame.set_index("week_end")
        price_series.append(indexed["adj_close"].rename(symbol))
        source_series.append(indexed["source_timestamp"].rename(symbol))

    prices = pd.concat(price_series, axis=1, join="inner").dropna().sort_index()
    sources = pd.concat(source_series, axis=1, join="inner").reindex(prices.index)

    if prices.empty:
        raise ValueError("required symbols have no common weekly history")
    if sources.isna().any().any():
        raise AssertionError("aligned source timestamps contain missing values")

    for symbol in symbols:
        if (sources[symbol] > prices.index).any():
            raise AssertionError(f"{symbol}: aligned source timestamp is in the future")

    history = CommonHistory(
        common_start=prices.index[0],
        common_end=prices.index[-1],
        symbols=symbols,
        observations=len(prices),
    )
    return prices, sources, history


def build_sector_rrg_panel(
    prices: pd.DataFrame,
    *,
    benchmark_symbol: str = "SPY",
    sector_symbols: Sequence[str] | None = None,
    config: RRGConfig | None = None,
) -> pd.DataFrame:
    """Calculate one causal RRG-style history per sector on aligned prices."""
    if benchmark_symbol not in prices.columns:
        raise ValueError(f"benchmark symbol {benchmark_symbol!r} is missing")
    if prices.index.has_duplicates or not prices.index.is_monotonic_increasing:
        raise ValueError("price index must be unique and chronological")

    sectors = tuple(sector_symbols or [c for c in prices.columns if c != benchmark_symbol])
    if not sectors:
        raise ValueError("no sector symbols supplied")
    missing = [symbol for symbol in sectors if symbol not in prices.columns]
    if missing:
        raise ValueError(f"sector symbols missing from prices: {missing}")

    benchmark = prices[benchmark_symbol]
    panels: list[pd.DataFrame] = []
    for symbol in sectors:
        result = calculate_rrg(prices[symbol], benchmark, config=config).copy()
        if result.empty:
            continue
        result.insert(0, "symbol", symbol)
        result.insert(1, "timestamp", result.index)
        panels.append(result.reset_index(drop=True))

    if not panels:
        raise ValueError("RRG calculation produced no sector observations")

    panel = pd.concat(panels, ignore_index=True)
    if panel.duplicated(["symbol", "timestamp"]).any():
        raise AssertionError("duplicate symbol/timestamp RRG rows")
    return panel.sort_values(["symbol", "timestamp"]).reset_index(drop=True)
