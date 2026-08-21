from __future__ import annotations

from dataclasses import dataclass
from collections.abc import Sequence

import numpy as np
import pandas as pd

from .deployment import make_latest_feature_row
from .modeling import feature_columns, fit_linear_coordinate_model, make_coordinate_supervised_dataset


@dataclass(frozen=True)
class StrategyBacktestConfig:
    """Pre-declared strategy parameters for the exploratory economic-value test."""

    horizon: int = 1
    lookback: int = 20
    min_training_rows: int = 156
    top_k: int = 3
    transaction_cost_per_dollar: float = 0.001

    def __post_init__(self) -> None:
        if self.horizon != 1:
            raise ValueError("v0.1 strategy is frozen to the 1-week horizon")
        if self.lookback <= 0:
            raise ValueError("lookback must be > 0")
        if self.min_training_rows < 2:
            raise ValueError("min_training_rows must be >= 2")
        if self.top_k <= 0:
            raise ValueError("top_k must be > 0")
        if not 0 <= self.transaction_cost_per_dollar < 1:
            raise ValueError("transaction_cost_per_dollar must be in [0, 1)")


def _canonical_rrg(rrg: pd.DataFrame) -> pd.DataFrame:
    required = {"rs_ratio", "rs_momentum"}
    missing = required.difference(rrg.columns)
    if missing:
        raise ValueError(f"missing RRG columns: {sorted(missing)}")
    out = rrg.copy()
    out.index = pd.DatetimeIndex(out.index)
    if out.index.tz is None:
        raise ValueError("RRG index must be timezone-aware")
    out.index = out.index.tz_convert("UTC")
    out = out.sort_index()
    if out.index.has_duplicates:
        raise ValueError("RRG index must be unique")
    if out[["rs_ratio", "rs_momentum"]].isna().any().any():
        raise ValueError("RRG coordinates cannot be missing")
    return out


def forecast_at_decision(
    rrg: pd.DataFrame,
    *,
    decision_timestamp: pd.Timestamp,
    config: StrategyBacktestConfig,
) -> dict[str, object]:
    """Fit using only labels known by one historical decision timestamp."""
    history = _canonical_rrg(rrg)
    decision = pd.Timestamp(decision_timestamp)
    if decision.tzinfo is None:
        raise ValueError("decision_timestamp must be timezone-aware")
    decision = decision.tz_convert("UTC")
    if decision not in history.index:
        raise ValueError("decision_timestamp is not an RRG observation")

    visible = history.loc[:decision].copy()
    supervised = make_coordinate_supervised_dataset(
        visible,
        horizon=config.horizon,
        lookback=config.lookback,
    )
    if len(supervised) < config.min_training_rows:
        raise ValueError("insufficient point-in-time training rows")

    features = feature_columns(supervised)
    targets = ["target_rs_ratio", "target_rs_momentum"]
    latest = make_latest_feature_row(visible, lookback=config.lookback)[features]
    model = fit_linear_coordinate_model(supervised[features], supervised[targets])
    prediction = np.asarray(model.predict(latest), dtype=float)
    if prediction.shape != (1, 2):
        raise AssertionError("linear forecast must return one coordinate pair")

    current = visible.iloc[-1]
    predicted_ratio = float(prediction[0, 0])
    predicted_momentum = float(prediction[0, 1])

    # The last supervised target must be exactly the decision observation.
    last_train_decision = pd.Timestamp(supervised.index[-1])
    last_position = visible.index.get_loc(last_train_decision)
    last_target = visible.index[last_position + config.horizon]
    if last_target != decision:
        raise AssertionError("training uses a label not aligned to the decision cutoff")

    return {
        "decision_timestamp": decision,
        "training_rows": int(len(supervised)),
        "training_last_decision": last_train_decision,
        "training_last_target": last_target,
        "current_rs_ratio": float(current["rs_ratio"]),
        "current_rs_momentum": float(current["rs_momentum"]),
        "predicted_rs_ratio": predicted_ratio,
        "predicted_rs_momentum": predicted_momentum,
        "current_strength": float(current["rs_ratio"] + current["rs_momentum"]),
        "forecast_strength": float(predicted_ratio + predicted_momentum),
    }


def _ols_equivalent_prediction(
    X_train: pd.DataFrame,
    y_train: pd.DataFrame,
    X_latest: pd.DataFrame,
) -> np.ndarray:
    """Fast OLS prediction equivalent to StandardScaler -> LinearRegression.

    StandardScaler is an invertible affine transform for non-constant columns and
    LinearRegression includes an intercept, so unregularized OLS predictions are
    invariant to that scaling. This direct least-squares path is used only to make
    the repeated expanding historical backtest computationally practical. Tests
    compare it against the frozen sklearn pipeline.
    """
    x = X_train.to_numpy(float)
    y = y_train.to_numpy(float)
    latest = X_latest.to_numpy(float)
    design = np.column_stack([np.ones(len(x)), x])
    beta, *_ = np.linalg.lstsq(design, y, rcond=None)
    latest_design = np.column_stack([np.ones(len(latest)), latest])
    return latest_design @ beta


def generate_historical_signal_panel(
    panel: pd.DataFrame,
    *,
    sector_symbols: Sequence[str],
    config: StrategyBacktestConfig | None = None,
) -> pd.DataFrame:
    """Generate point-in-time 1-week forecasts for every eligible historical week.

    Repeated expanding OLS fits use a direct least-squares solver that is
    prediction-equivalent to the frozen StandardScaler -> LinearRegression
    pipeline. The deployment path itself continues to use sklearn directly.
    """
    cfg = config or StrategyBacktestConfig()
    required = {"symbol", "timestamp", "rs_ratio", "rs_momentum"}
    missing = required.difference(panel.columns)
    if missing:
        raise ValueError(f"panel missing columns: {sorted(missing)}")

    sectors = tuple(sector_symbols)
    if not sectors or len(set(sectors)) != len(sectors):
        raise ValueError("sector_symbols must be non-empty and unique")
    if cfg.top_k > len(sectors):
        raise ValueError("top_k cannot exceed sector count")

    histories: dict[str, pd.DataFrame] = {}
    supervised_by_symbol: dict[str, pd.DataFrame] = {}
    feature_names: dict[str, list[str]] = {}
    timestamp_sets: list[pd.DatetimeIndex] = []
    for symbol in sectors:
        rows = panel.loc[panel["symbol"] == symbol].copy()
        if rows.empty:
            raise ValueError(f"missing panel rows for {symbol}")
        rows["timestamp"] = pd.to_datetime(rows["timestamp"], utc=True)
        rrg = _canonical_rrg(rows.set_index("timestamp").sort_index())
        histories[symbol] = rrg
        supervised = make_coordinate_supervised_dataset(
            rrg,
            horizon=cfg.horizon,
            lookback=cfg.lookback,
        )
        supervised_by_symbol[symbol] = supervised
        feature_names[symbol] = feature_columns(supervised)
        timestamp_sets.append(pd.DatetimeIndex(rrg.index))

    common = timestamp_sets[0]
    for idx in timestamp_sets[1:]:
        common = common.intersection(idx)
    common = common.sort_values()
    if common.empty:
        raise ValueError("sector RRG histories have no common timestamps")

    output: list[dict[str, object]] = []
    targets = ["target_rs_ratio", "target_rs_momentum"]
    for decision in common:
        week_rows: list[dict[str, object]] = []
        eligible = True
        for symbol in sectors:
            rrg = histories[symbol]
            pos = rrg.index.get_loc(decision)
            if not isinstance(pos, (int, np.integer)) or pos < cfg.horizon:
                eligible = False
                break
            last_train_decision = rrg.index[pos - cfg.horizon]
            supervised = supervised_by_symbol[symbol]
            train = supervised.loc[:last_train_decision]
            if len(train) < cfg.min_training_rows:
                eligible = False
                break

            features = feature_names[symbol]
            visible = rrg.iloc[: pos + 1]
            latest = make_latest_feature_row(visible, lookback=cfg.lookback)[features]
            prediction = _ols_equivalent_prediction(
                train[features], train[targets], latest
            )
            if prediction.shape != (1, 2):
                raise AssertionError("historical OLS forecast must return one coordinate pair")

            # Strict point-in-time target check: last train row's target is current decision.
            train_pos = rrg.index.get_loc(pd.Timestamp(train.index[-1]))
            last_train_target = rrg.index[train_pos + cfg.horizon]
            if last_train_target != decision:
                raise AssertionError("historical fit includes an unavailable target")

            current = visible.iloc[-1]
            pred_ratio = float(prediction[0, 0])
            pred_momentum = float(prediction[0, 1])
            week_rows.append(
                {
                    "symbol": symbol,
                    "decision_timestamp": decision,
                    "training_rows": int(len(train)),
                    "training_last_decision": pd.Timestamp(train.index[-1]),
                    "training_last_target": last_train_target,
                    "current_rs_ratio": float(current["rs_ratio"]),
                    "current_rs_momentum": float(current["rs_momentum"]),
                    "predicted_rs_ratio": pred_ratio,
                    "predicted_rs_momentum": pred_momentum,
                    "current_strength": float(current["rs_ratio"] + current["rs_momentum"]),
                    "forecast_strength": float(pred_ratio + pred_momentum),
                }
            )
        if eligible:
            output.extend(week_rows)

    if not output:
        raise ValueError("no historical decision week satisfies the frozen training minimum")
    result = pd.DataFrame.from_records(output)
    if result.duplicated(["symbol", "decision_timestamp"]).any():
        raise AssertionError("duplicate historical signal rows")
    per_week = result.groupby("decision_timestamp")["symbol"].nunique()
    if not (per_week == len(sectors)).all():
        raise AssertionError("eligible decision weeks must contain every sector")
    return result.sort_values(["decision_timestamp", "symbol"]).reset_index(drop=True)


def top_k_equal_weights(
    signal_rows: pd.DataFrame,
    *,
    score_column: str,
    sector_symbols: Sequence[str],
    top_k: int,
) -> pd.DataFrame:
    """Convert one score per sector/week into deterministic equal-weight top-k targets."""
    required = {"decision_timestamp", "symbol", score_column}
    missing = required.difference(signal_rows.columns)
    if missing:
        raise ValueError(f"signal rows missing columns: {sorted(missing)}")
    sectors = tuple(sector_symbols)
    if top_k <= 0 or top_k > len(sectors):
        raise ValueError("invalid top_k")

    records: list[dict[str, object]] = []
    for decision, group in signal_rows.groupby("decision_timestamp", sort=True):
        if set(group["symbol"]) != set(sectors):
            raise ValueError("each decision must contain exactly the configured sectors")
        ranked = group.sort_values([score_column, "symbol"], ascending=[False, True])
        selected = set(ranked.head(top_k)["symbol"])
        for symbol in sectors:
            records.append(
                {
                    "decision_timestamp": pd.Timestamp(decision),
                    "symbol": symbol,
                    "target_weight": (1.0 / top_k) if symbol in selected else 0.0,
                }
            )
    weights = pd.DataFrame.from_records(records)
    sums = weights.groupby("decision_timestamp")["target_weight"].sum()
    if not np.allclose(sums.to_numpy(), 1.0):
        raise AssertionError("target weights must sum to one")
    return weights


def fixed_equal_weight_targets(
    decision_timestamps: Sequence[pd.Timestamp],
    *,
    symbols: Sequence[str],
) -> pd.DataFrame:
    assets = tuple(symbols)
    if not assets:
        raise ValueError("symbols cannot be empty")
    records = []
    weight = 1.0 / len(assets)
    for decision in decision_timestamps:
        for symbol in assets:
            records.append(
                {
                    "decision_timestamp": pd.Timestamp(decision),
                    "symbol": symbol,
                    "target_weight": weight,
                }
            )
    return pd.DataFrame.from_records(records)


def _weights_wide(
    targets: pd.DataFrame,
    *,
    asset_columns: Sequence[str],
) -> pd.DataFrame:
    required = {"decision_timestamp", "symbol", "target_weight"}
    missing = required.difference(targets.columns)
    if missing:
        raise ValueError(f"targets missing columns: {sorted(missing)}")
    wide = targets.pivot(index="decision_timestamp", columns="symbol", values="target_weight")
    wide.index = pd.to_datetime(wide.index, utc=True)
    wide = wide.sort_index().reindex(columns=list(asset_columns), fill_value=0.0).fillna(0.0)
    if not np.allclose(wide.sum(axis=1).to_numpy(), 1.0):
        raise ValueError("portfolio target weights must sum to one")
    return wide


def execution_schedule(
    decision_timestamps: Sequence[pd.Timestamp],
    *,
    common_daily_index: pd.DatetimeIndex,
) -> pd.Series:
    """Map each Friday decision to the first strictly later common daily timestamp."""
    daily = pd.DatetimeIndex(common_daily_index)
    if daily.tz is None:
        raise ValueError("common_daily_index must be timezone-aware")
    daily = daily.tz_convert("UTC").sort_values().unique()
    if len(daily) == 0:
        raise ValueError("common_daily_index cannot be empty")

    decisions = pd.DatetimeIndex(pd.to_datetime(list(decision_timestamps), utc=True)).sort_values()
    mapped: list[pd.Timestamp | pd.NaT] = []
    for decision in decisions:
        pos = daily.searchsorted(decision, side="right")
        mapped.append(daily[pos] if pos < len(daily) else pd.NaT)
    return pd.Series(mapped, index=decisions, name="execution_timestamp")


def simulate_target_weight_strategy(
    targets: pd.DataFrame,
    *,
    common_daily_prices: pd.DataFrame,
    transaction_cost_per_dollar: float,
) -> pd.DataFrame:
    """Simulate next-session-close execution with drift-aware turnover and costs."""
    if not 0 <= transaction_cost_per_dollar < 1:
        raise ValueError("transaction_cost_per_dollar must be in [0, 1)")
    prices = common_daily_prices.copy()
    prices.index = pd.DatetimeIndex(prices.index)
    if prices.index.tz is None:
        raise ValueError("daily price index must be timezone-aware")
    prices.index = prices.index.tz_convert("UTC")
    prices = prices.sort_index()
    if prices.index.has_duplicates:
        raise ValueError("daily price index must be unique")
    if prices.isna().any().any() or (prices <= 0).any().any():
        raise ValueError("daily prices must be complete and strictly positive")

    wide = _weights_wide(targets, asset_columns=prices.columns)
    decisions = wide.index
    executions = execution_schedule(decisions, common_daily_index=prices.index)

    valid_decisions = [d for d in decisions if pd.notna(executions.loc[d])]
    if len(valid_decisions) < 2:
        raise ValueError("at least two executable decision weeks are required")
    # Only intervals with both an entry execution and the following rebalance execution are scored.
    valid_decisions = pd.DatetimeIndex(valid_decisions)

    equity = 1.0
    pretrade = pd.Series(0.0, index=prices.columns, dtype=float)
    records: list[dict[str, object]] = []

    for i in range(len(valid_decisions) - 1):
        decision = valid_decisions[i]
        next_decision = valid_decisions[i + 1]
        entry = pd.Timestamp(executions.loc[decision])
        exit_ = pd.Timestamp(executions.loc[next_decision])
        if not entry < exit_:
            raise AssertionError("execution timestamps must increase")

        target = wide.loc[decision].astype(float)
        gross_traded_weight = float((target - pretrade).abs().sum())
        cost_fraction = transaction_cost_per_dollar * gross_traded_weight
        cost_paid = equity * cost_fraction
        equity_after_cost = equity * (1.0 - cost_fraction)

        asset_returns = prices.loc[exit_] / prices.loc[entry] - 1.0
        gross_return = float((target * asset_returns).sum())
        equity_end = equity_after_cost * (1.0 + gross_return)
        net_return = equity_end / equity - 1.0

        denominator = 1.0 + gross_return
        if denominator <= 0:
            raise ValueError("portfolio lost 100% or more in one interval")
        pretrade = target * (1.0 + asset_returns) / denominator

        records.append(
            {
                "decision_timestamp": decision,
                "execution_timestamp": entry,
                "exit_timestamp": exit_,
                "gross_return": gross_return,
                "net_return": net_return,
                "gross_traded_weight": gross_traded_weight,
                "transaction_cost_fraction": cost_fraction,
                "transaction_cost_paid": cost_paid,
                "equity_start": equity,
                "equity_end": equity_end,
            }
        )
        equity = equity_end

    result = pd.DataFrame.from_records(records)
    if result.empty:
        raise ValueError("strategy produced no scored intervals")
    return result


def performance_summary(periods: pd.DataFrame) -> dict[str, float | int | pd.Timestamp]:
    required = {
        "execution_timestamp", "exit_timestamp", "net_return", "gross_return",
        "gross_traded_weight", "transaction_cost_paid", "equity_end",
    }
    missing = required.difference(periods.columns)
    if missing:
        raise ValueError(f"periods missing columns: {sorted(missing)}")
    returns = periods["net_return"].astype(float)
    if len(returns) < 2:
        raise ValueError("at least two periods are required for performance summary")

    start = pd.Timestamp(periods.iloc[0]["execution_timestamp"])
    end = pd.Timestamp(periods.iloc[-1]["exit_timestamp"])
    elapsed_days = (end - start).total_seconds() / 86400.0
    end_value = float(periods.iloc[-1]["equity_end"])
    cagr = end_value ** (365.25 / elapsed_days) - 1.0 if elapsed_days > 0 else np.nan
    vol = float(returns.std(ddof=1) * np.sqrt(52.0))
    sharpe = float(returns.mean() / returns.std(ddof=1) * np.sqrt(52.0)) if returns.std(ddof=1) > 0 else np.nan

    equity_curve = pd.Series([1.0, *periods["equity_end"].astype(float).tolist()])
    running_max = equity_curve.cummax()
    drawdown = equity_curve / running_max - 1.0

    return {
        "intervals": int(len(periods)),
        "first_execution": start,
        "last_exit": end,
        "ending_value": end_value,
        "cumulative_net_return": end_value - 1.0,
        "cagr": float(cagr),
        "annualized_volatility": vol,
        "sharpe_zero_rf": sharpe,
        "max_drawdown": float(drawdown.min()),
        "mean_weekly_gross_traded_weight": float(periods["gross_traded_weight"].mean()),
        "sum_transaction_cost_fraction": float(periods["transaction_cost_fraction"].sum()),
        "total_transaction_cost_paid_on_initial_1": float(periods["transaction_cost_paid"].sum()),
    }
