from __future__ import annotations

from dataclasses import dataclass

import numpy as np
import pandas as pd

from .holdout import (
    FinalHoldoutProtocol,
    make_timestamped_supervised_dataset,
    split_fixed_final_holdout,
)
from .modeling import feature_columns, fit_linear_coordinate_model
from .rrg import RRGConfig, calculate_rrg, classify_quadrant


@dataclass(frozen=True)
class SyntheticNullConfig:
    """Configuration for a no-predictability structural null benchmark.

    The synthetic benchmark and sectors are geometric random walks with only
    contemporaneous market beta. Returns are independent through time, so the
    data-generating process contains no forecastable market signal by design.
    """

    n_weeks: int = 424
    n_sectors: int = 11
    n_trials: int = 12
    beta: float = 0.9
    market_vol: float = 0.025
    idiosyncratic_vol: float = 0.020
    holdout_weeks: int = 52
    horizons: tuple[int, ...] = (1, 2, 4, 8)
    lookback: int = 20

    def __post_init__(self) -> None:
        if self.n_weeks <= 0 or self.n_sectors <= 0 or self.n_trials <= 0:
            raise ValueError("week, sector, and trial counts must be positive")
        if not (0 < self.holdout_weeks < self.n_weeks):
            raise ValueError("holdout_weeks must be inside the simulated history")
        if any(h <= 0 for h in self.horizons):
            raise ValueError("horizons must be positive")
        if self.lookback <= 0:
            raise ValueError("lookback must be positive")
        if self.market_vol <= 0 or self.idiosyncratic_vol <= 0:
            raise ValueError("volatilities must be positive")


def simulate_no_signal_prices(
    *,
    seed: int,
    config: SyntheticNullConfig,
) -> tuple[pd.Series, dict[str, pd.Series]]:
    """Generate one benchmark and sector panel with zero serial predictability."""
    rng = np.random.default_rng(seed)
    index = pd.date_range(
        "2018-06-22",
        periods=config.n_weeks,
        freq="W-FRI",
        tz="UTC",
    )

    market_log_return = rng.normal(0.0, config.market_vol, config.n_weeks)
    benchmark = pd.Series(
        100.0 * np.exp(np.cumsum(market_log_return)),
        index=index,
        name="SYNTH_BENCH",
    )

    sectors: dict[str, pd.Series] = {}
    for sector_index in range(config.n_sectors):
        idiosyncratic = rng.normal(
            0.0,
            config.idiosyncratic_vol,
            config.n_weeks,
        )
        sector_log_return = config.beta * market_log_return + idiosyncratic
        symbol = f"SYNTH_{sector_index + 1:02d}"
        sectors[symbol] = pd.Series(
            100.0 * np.exp(np.cumsum(sector_log_return)),
            index=index,
            name=symbol,
        )

    return benchmark, sectors


def _axis_side_accuracy(actual: np.ndarray, predicted: np.ndarray, axis: int) -> float:
    return float(
        np.mean((actual[:, axis] >= 100.0) == (predicted[:, axis] >= 100.0))
    )


def _evaluate_one_series(
    rrg: pd.DataFrame,
    *,
    horizon: int,
    protocol: FinalHoldoutProtocol,
) -> dict[str, float | int]:
    supervised = make_timestamped_supervised_dataset(
        rrg,
        horizon=horizon,
        lookback=protocol.lookback,
    )
    split = split_fixed_final_holdout(supervised, protocol=protocol)
    features = feature_columns(split.train)
    targets = ["target_rs_ratio", "target_rs_momentum"]

    model = fit_linear_coordinate_model(
        split.train[features],
        split.train[targets],
    )
    linear = np.asarray(model.predict(split.holdout[features]), dtype=float)
    persistence = split.holdout[
        ["rs_ratio_lag0", "rs_momentum_lag0"]
    ].to_numpy(float)
    actual = split.holdout[targets].to_numpy(float)

    actual_quadrant = np.asarray(
        [classify_quadrant(float(r), float(m)) for r, m in actual],
        dtype=object,
    )
    linear_quadrant = np.asarray(
        [classify_quadrant(float(r), float(m)) for r, m in linear],
        dtype=object,
    )
    persistence_quadrant = np.asarray(
        [classify_quadrant(float(r), float(m)) for r, m in persistence],
        dtype=object,
    )

    linear_distance = np.linalg.norm(linear - actual, axis=1)
    persistence_distance = np.linalg.norm(persistence - actual, axis=1)
    persistence_mean = float(np.mean(persistence_distance))
    reduction = np.nan
    if persistence_mean > 0:
        reduction = 1.0 - float(np.mean(linear_distance)) / persistence_mean

    return {
        "n": int(len(actual)),
        "linear_quadrant_accuracy": float(
            np.mean(linear_quadrant == actual_quadrant)
        ),
        "persistence_quadrant_accuracy": float(
            np.mean(persistence_quadrant == actual_quadrant)
        ),
        "coordinate_distance_reduction": float(reduction),
        "linear_ratio_side_accuracy": _axis_side_accuracy(actual, linear, 0),
        "persistence_ratio_side_accuracy": _axis_side_accuracy(
            actual, persistence, 0
        ),
        "linear_momentum_side_accuracy": _axis_side_accuracy(actual, linear, 1),
        "persistence_momentum_side_accuracy": _axis_side_accuracy(
            actual, persistence, 1
        ),
    }


def run_synthetic_null_benchmark(
    config: SyntheticNullConfig | None = None,
    *,
    rrg_config: RRGConfig | None = None,
) -> pd.DataFrame:
    """Run the current coordinate-forecast pipeline on no-signal synthetic data.

    A large linear-vs-persistence edge here is evidence of structural/mechanical
    target predictability, not market predictability.
    """
    cfg = config or SyntheticNullConfig()
    rrg_cfg = rrg_config or RRGConfig()
    records: list[dict[str, float | int]] = []

    for trial in range(cfg.n_trials):
        benchmark, sectors = simulate_no_signal_prices(seed=trial, config=cfg)
        index = benchmark.index
        protocol = FinalHoldoutProtocol(
            target_start=index[-cfg.holdout_weeks],
            target_end=index[-1],
            horizons=cfg.horizons,
            lookback=cfg.lookback,
        )

        for horizon in cfg.horizons:
            sector_rows = []
            for sector_price in sectors.values():
                rrg = calculate_rrg(sector_price, benchmark, rrg_cfg)
                sector_rows.append(
                    _evaluate_one_series(
                        rrg,
                        horizon=horizon,
                        protocol=protocol,
                    )
                )

            weights = np.asarray([row["n"] for row in sector_rows], dtype=float)
            record: dict[str, float | int] = {
                "trial": trial,
                "horizon": horizon,
                "n": int(np.sum(weights)),
            }
            for metric in (
                "linear_quadrant_accuracy",
                "persistence_quadrant_accuracy",
                "coordinate_distance_reduction",
                "linear_ratio_side_accuracy",
                "persistence_ratio_side_accuracy",
                "linear_momentum_side_accuracy",
                "persistence_momentum_side_accuracy",
            ):
                values = np.asarray(
                    [row[metric] for row in sector_rows], dtype=float
                )
                record[metric] = float(np.average(values, weights=weights))
            record["quadrant_edge_pp"] = 100.0 * (
                float(record["linear_quadrant_accuracy"])
                - float(record["persistence_quadrant_accuracy"])
            )
            records.append(record)

    return pd.DataFrame.from_records(records)


def summarize_synthetic_null(results: pd.DataFrame) -> pd.DataFrame:
    """Summarize null performance across trials by horizon."""
    required = {
        "horizon",
        "linear_quadrant_accuracy",
        "persistence_quadrant_accuracy",
        "quadrant_edge_pp",
        "coordinate_distance_reduction",
    }
    missing = required.difference(results.columns)
    if missing:
        raise ValueError(f"missing required columns: {sorted(missing)}")

    return (
        results.groupby("horizon", sort=True)
        .agg(
            trials=("trial", "nunique"),
            linear_quadrant_accuracy=("linear_quadrant_accuracy", "mean"),
            persistence_quadrant_accuracy=(
                "persistence_quadrant_accuracy", "mean"
            ),
            quadrant_edge_pp=("quadrant_edge_pp", "mean"),
            coordinate_distance_reduction=(
                "coordinate_distance_reduction", "mean"
            ),
            linear_ratio_side_accuracy=("linear_ratio_side_accuracy", "mean"),
            persistence_ratio_side_accuracy=(
                "persistence_ratio_side_accuracy", "mean"
            ),
            linear_momentum_side_accuracy=(
                "linear_momentum_side_accuracy", "mean"
            ),
            persistence_momentum_side_accuracy=(
                "persistence_momentum_side_accuracy", "mean"
            ),
        )
        .reset_index()
    )
