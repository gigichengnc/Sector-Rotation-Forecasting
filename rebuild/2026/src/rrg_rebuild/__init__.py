"""Public package surface for the RRG research reconstruction."""

__version__ = "0.1.0"

from .deployment import (
    DeploymentConfig,
    forecast_latest_for_series,
    forecast_panel_latest,
    make_forecast_table,
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
    "calculate_rrg",
    "classify_quadrant",
    "fetch_market_data",
    "forecast_latest_for_series",
    "forecast_panel_latest",
    "make_forecast_table",
    "run_fresh_forecast",
]
