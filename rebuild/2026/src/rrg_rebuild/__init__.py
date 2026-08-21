"""Public package surface for the sector-rotation research reconstruction."""

__version__ = "0.2.0"

from .deployment import (
    DeploymentConfig,
    forecast_latest_for_series,
    forecast_panel_latest,
    make_forecast_table,
)
from .future_relative import (
    fit_l2_outperformance_model,
    future_relative_feature_columns,
    make_future_relative_return_dataset,
)
from .null_benchmark import (
    SyntheticNullConfig,
    run_synthetic_null_benchmark,
    simulate_no_signal_prices,
    summarize_synthetic_null,
)
from .rrg import RRGConfig, calculate_rrg, classify_quadrant
from .workflow import (
    DEFAULT_SYMBOLS,
    FreshForecastRun,
    MarketDataFetch,
    fetch_market_data,
    run_fresh_forecast,
)

__all__ = [
    "__version__",
    "DEFAULT_SYMBOLS",
    "DeploymentConfig",
    "FreshForecastRun",
    "MarketDataFetch",
    "RRGConfig",
    "SyntheticNullConfig",
    "calculate_rrg",
    "classify_quadrant",
    "fetch_market_data",
    "fit_l2_outperformance_model",
    "forecast_latest_for_series",
    "forecast_panel_latest",
    "future_relative_feature_columns",
    "make_forecast_table",
    "make_future_relative_return_dataset",
    "run_fresh_forecast",
    "run_synthetic_null_benchmark",
    "simulate_no_signal_prices",
    "summarize_synthetic_null",
]
