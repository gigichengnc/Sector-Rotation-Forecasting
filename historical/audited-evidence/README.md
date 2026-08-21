# RRG Terminal - Professional Relative Rotation Graph Analysis

A professional-grade trading terminal for analyzing sector rotation using Relative Rotation Graphs (RRG) with AI-powered predictions.

## What is RRG?

Relative Rotation Graphs visualize the relative strength and momentum of securities against a benchmark. Securities rotate through four quadrants:

- **Leading** (top-right): Strong relative strength and positive momentum - outperforming
- **Weakening** (bottom-right): Strong relative strength but declining momentum - losing steam  
- **Lagging** (bottom-left): Weak relative strength and negative momentum - underperforming
- **Improving** (top-left): Weak relative strength but improving momentum - potential turnaround

## Quick Start

### Prerequisites

- Rust 1.70+ ([install](https://rustup.rs/))
- Windows, macOS, or Linux

### Run the Application

```bash
# Clone and build
cargo build --release

# Start the server
cargo run --package rrg-web

# Open in browser
# http://localhost:3000
```

Or use the PowerShell script:
```powershell
.\run_demo.ps1
```

## Features

### Main Dashboard

The terminal displays:

| Component | Description |
|-----------|-------------|
| **RRG Chart** | Interactive scatter plot showing sector positions and rotation trails |
| **Sector Watchlist** | Real-time list of tracked ETFs with quadrant status |
| **AI Predictions** | ML-based forecasts for sector movements |
| **Market Alerts** | Notifications for quadrant transitions and market events |
| **Portfolio View** | Track your holdings and allocation |

### View Modes

Toggle between visualization modes using the buttons:
- **CURRENT** - Show current positions only
- **SCATTER** - Standard scatter plot
- **TRAILS** - Show historical rotation paths
- **HEATMAP** - Density visualization

### Quick Actions

- **Run Analysis** - Refresh RRG calculations with latest data
- **Scenario Sim** - Run what-if scenarios with different market conditions
- **Export Data** - Download analysis results
- **Clear Alerts** - Dismiss notifications

## Understanding the Display

### Sector Watchlist

Each sector shows:
- Symbol (e.g., XLK, XLF)
- Current quadrant (Leading, Weakening, Lagging, Improving)
- RS-Ratio value

Color coding:
- 🟢 Green = Leading/Improving (bullish)
- 🔴 Red = Lagging (bearish)
- 🟠 Orange = Weakening (caution)

### AI Predictions Panel

Shows probability forecasts:
```
XLK    65%  Stay Leading
XLE    72%  Move to Leading
XLF    68%  Stay Lagging
```

Higher percentages indicate stronger conviction.

### Market Alerts

Real-time notifications for:
- Quadrant transitions ("XLK crossed into Leading")
- Momentum changes ("XLF momentum turning positive")
- Risk events ("VIX spike detected")

## Tracked Sectors (SPDR ETFs)

| Symbol | Sector |
|--------|--------|
| XLK | Technology |
| XLF | Financials |
| XLE | Energy |
| XLV | Healthcare |
| XLI | Industrials |
| XLP | Consumer Staples |
| XLY | Consumer Discretionary |
| XLB | Materials |
| XLU | Utilities |
| XLRE | Real Estate |
| XLC | Communication Services |

## Configuration

### Change Benchmark

Use the **BENCHMARK** dropdown to select:
- SPY (S&P 500) - default
- QQQ (Nasdaq 100)
- IWM (Russell 2000)

### Adjust Lookback Period

Use the **LOOKBACK** dropdown:
- 30D, 60D, 90D, 180D, 1Y

Longer periods show more stable trends; shorter periods are more responsive.

## Tabs

### POSITIONS
Current sector positions with entry points and P&L.

### PERFORMANCE  
Historical performance metrics and returns.

### CORRELATIONS
Cross-sector correlation matrix.

### BACKTEST
Test strategies against historical data.

### PORTFOLIO
Manage and analyze your portfolio holdings.

## API Endpoints

For programmatic access:

```
GET  /api/rrg/{symbol}           - Get RRG data for symbol
GET  /api/sectors                - List all sectors
POST /api/analyze                - Run full analysis
GET  /api/predictions/{symbol}   - Get AI predictions
WS   /ws                         - Real-time updates
```

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `R` | Refresh data |
| `T` | Toggle trails |
| `H` | Toggle heatmap |
| `Esc` | Clear selection |

## Troubleshooting

**Server won't start:**
```bash
# Check if port 3000 is in use
netstat -an | findstr 3000

# Use different port
cargo run --package rrg-web -- --port 8080
```

**No data showing:**
- Check internet connection (fetches from Yahoo Finance)
- Wait for initial data load (~10 seconds)
- Check browser console for errors

**Slow performance:**
- Reduce lookback period
- Disable trails/heatmap
- Close other browser tabs

## Architecture

```
┌─────────────────────────────────────────────────┐
│                  Web Frontend                    │
│         (HTML/CSS/JavaScript)                   │
└─────────────────┬───────────────────────────────┘
                  │ HTTP/WebSocket
┌─────────────────▼───────────────────────────────┐
│              rrg-web (Axum)                     │
│         REST API + WebSocket Server             │
└─────────────────┬───────────────────────────────┘
                  │
    ┌─────────────┼─────────────┐
    ▼             ▼             ▼
┌────────┐  ┌──────────┐  ┌──────────┐
│rrg-calc│  │  rrg-ml  │  │ rrg-data │
│  RRG   │  │   AI/ML  │  │  Data    │
│ Engine │  │Predictions│  │ Fetcher  │
└────────┘  └──────────┘  └──────────┘
```

## License

MIT License - See LICENSE file for details.

## Support

For issues and feature requests, please open a GitHub issue.
