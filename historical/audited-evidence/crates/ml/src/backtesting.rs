use crate::{MLError, MLResult};
use rrg_data::{ETFData, OHLCVData};
use rrg_calc::{RRGData, RRGCalculator, Alert, AlertSystem, AlertConfig};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;
use uuid::Uuid;

/// Backtesting configuration parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestConfig {
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub initial_capital: f64,
    pub rebalance_frequency_days: i64,
    pub transaction_cost_bps: f64, // basis points
    pub benchmark_symbol: String,
    pub max_position_size: f64, // as fraction of portfolio
    pub min_position_size: f64,
}

impl Default for BacktestConfig {
    fn default() -> Self {
        Self {
            start_date: Utc::now() - Duration::days(365),
            end_date: Utc::now(),
            initial_capital: 100000.0,
            rebalance_frequency_days: 30,
            transaction_cost_bps: 5.0,
            benchmark_symbol: "SPY".to_string(),
            max_position_size: 0.2,
            min_position_size: 0.01,
        }
    }
}

/// Portfolio position for backtesting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub symbol: String,
    pub shares: f64,
    pub avg_cost: f64,
    pub current_price: f64,
    pub market_value: f64,
    pub weight: f64,
    pub unrealized_pnl: f64,
}

impl Position {
    pub fn new(symbol: String, shares: f64, price: f64) -> Self {
        let market_value = shares * price;
        Self {
            symbol,
            shares,
            avg_cost: price,
            current_price: price,
            market_value,
            weight: 0.0, // Will be calculated by portfolio
            unrealized_pnl: 0.0,
        }
    }

    pub fn update_price(&mut self, new_price: f64) {
        self.current_price = new_price;
        self.market_value = self.shares * new_price;
        self.unrealized_pnl = (new_price - self.avg_cost) * self.shares;
    }
}

/// Portfolio state at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioSnapshot {
    pub timestamp: DateTime<Utc>,
    pub positions: HashMap<String, Position>,
    pub cash: f64,
    pub total_value: f64,
    pub total_return: f64,
    pub benchmark_return: f64,
    pub alpha: f64,
    pub beta: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
}

/// Trade execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub symbol: String,
    pub action: TradeAction,
    pub shares: f64,
    pub price: f64,
    pub value: f64,
    pub transaction_cost: f64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradeAction {
    Buy,
    Sell,
    Rebalance,
}

/// Backtesting performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestMetrics {
    pub total_return: f64,
    pub annualized_return: f64,
    pub volatility: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub calmar_ratio: f64,
    pub win_rate: f64,
    pub profit_factor: f64,
    pub beta: f64,
    pub alpha: f64,
    pub information_ratio: f64,
    pub tracking_error: f64,
    pub benchmark_return: f64,
    pub excess_return: f64,
}

/// Backtesting results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestResults {
    pub config: BacktestConfig,
    pub metrics: BacktestMetrics,
    pub portfolio_history: Vec<PortfolioSnapshot>,
    pub trades: Vec<Trade>,
    pub alerts_generated: Vec<Alert>,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub duration_days: i64,
}

/// Backtesting engine for historical model performance validation
pub struct BacktestingEngine {
    config: BacktestConfig,
    rrg_calculator: RRGCalculator,
    alert_system: AlertSystem,
    portfolio_history: Vec<PortfolioSnapshot>,
    trades: Vec<Trade>,
    alerts: Vec<Alert>,
}

impl BacktestingEngine {
    pub fn new(config: BacktestConfig) -> Self {
        Self {
            config,
            rrg_calculator: RRGCalculator::new(),
            alert_system: AlertSystem::with_default_config(),
            portfolio_history: Vec::new(),
            trades: Vec::new(),
            alerts: Vec::new(),
        }
    }

    /// Run backtest on historical data
    pub async fn run_backtest(
        &mut self,
        etf_data: &HashMap<String, ETFData>,
        benchmark_data: &ETFData,
    ) -> MLResult<BacktestResults> {
        // Validate input data
        self.validate_data(etf_data, benchmark_data)?;

        // Initialize portfolio
        let mut portfolio = self.initialize_portfolio(etf_data)?;
        let mut current_date = self.config.start_date;

        // Main backtesting loop
        while current_date <= self.config.end_date {
            // Get data for current date
            let current_data = self.get_data_for_date(etf_data, benchmark_data, current_date)?;
            
            // Calculate RRG positions
            let rrg_results = self.calculate_rrg_for_date(&current_data, current_date)?;
            
            // Generate alerts
            let new_alerts = self.generate_alerts(&rrg_results)?;
            self.alerts.extend(new_alerts);
            
            // Update portfolio prices
            self.update_portfolio_prices(&mut portfolio, &current_data)?;
            
            // Check if rebalancing is needed
            if self.should_rebalance(current_date)? {
                self.rebalance_portfolio(&mut portfolio, &rrg_results, current_date)?;
            }
            
            // Create portfolio snapshot
            let snapshot = self.create_portfolio_snapshot(&portfolio, current_date, benchmark_data)?;
            self.portfolio_history.push(snapshot);
            
            // Move to next day
            current_date += Duration::days(1);
        }

        // Calculate final metrics
        let metrics = self.calculate_metrics(benchmark_data)?;

        Ok(BacktestResults {
            config: self.config.clone(),
            metrics,
            portfolio_history: self.portfolio_history.clone(),
            trades: self.trades.clone(),
            alerts_generated: self.alerts.clone(),
            start_date: self.config.start_date,
            end_date: self.config.end_date,
            duration_days: (self.config.end_date - self.config.start_date).num_days(),
        })
    }

    fn validate_data(
        &self,
        etf_data: &HashMap<String, ETFData>,
        benchmark_data: &ETFData,
    ) -> MLResult<()> {
        if etf_data.is_empty() {
            return Err(MLError::feature_error("No ETF data provided for backtesting"));
        }

        // Check date ranges
        for (symbol, data) in etf_data {
            if data.price_history.is_empty() {
                return Err(MLError::feature_error(format!("No price data for {}", symbol)));
            }

            let first_date = data.price_history.first().unwrap().timestamp;
            let last_date = data.price_history.last().unwrap().timestamp;

            if first_date > self.config.start_date || last_date < self.config.end_date {
                return Err(MLError::feature_error(format!(
                    "Insufficient data range for {}: available {} to {}, required {} to {}",
                    symbol, first_date, last_date, self.config.start_date, self.config.end_date
                )));
            }
        }

        // Validate benchmark data
        if benchmark_data.price_history.is_empty() {
            return Err(MLError::feature_error("No benchmark data provided"));
        }

        Ok(())
    }

    fn initialize_portfolio(&self, etf_data: &HashMap<String, ETFData>) -> MLResult<HashMap<String, Position>> {
        let mut portfolio = HashMap::new();
        let num_etfs = etf_data.len() as f64;
        let initial_weight = 1.0 / num_etfs;
        let initial_allocation = self.config.initial_capital * initial_weight;

        for (symbol, data) in etf_data {
            if let Some(first_price) = data.price_history.first() {
                let shares = initial_allocation / first_price.close;
                let position = Position::new(symbol.clone(), shares, first_price.close);
                portfolio.insert(symbol.clone(), position);
            }
        }

        Ok(portfolio)
    }

    fn get_data_for_date(
        &self,
        etf_data: &HashMap<String, ETFData>,
        benchmark_data: &ETFData,
        date: DateTime<Utc>,
    ) -> MLResult<HashMap<String, OHLCVData>> {
        let mut current_data = HashMap::new();

        for (symbol, data) in etf_data {
            if let Some(price_data) = self.find_price_data_for_date(&data.price_history, date) {
                current_data.insert(symbol.clone(), price_data.clone());
            }
        }

        // Add benchmark data
        if let Some(benchmark_price) = self.find_price_data_for_date(&benchmark_data.price_history, date) {
            current_data.insert(self.config.benchmark_symbol.clone(), benchmark_price.clone());
        }

        Ok(current_data)
    }

    fn find_price_data_for_date<'a>(&self, price_data: &'a [OHLCVData], target_date: DateTime<Utc>) -> Option<&'a OHLCVData> {
        // Find the closest price data to the target date (within 7 days)
        price_data
            .iter()
            .filter(|data| (data.timestamp.date_naive() - target_date.date_naive()).num_days().abs() <= 7)
            .min_by_key(|data| (data.timestamp.date_naive() - target_date.date_naive()).num_days().abs())
    }

    fn calculate_rrg_for_date(
        &self,
        current_data: &HashMap<String, OHLCVData>,
        _date: DateTime<Utc>,
    ) -> MLResult<HashMap<String, RRGData>> {
        let mut rrg_results = HashMap::new();

        // This is a simplified version - in practice, you'd need historical data
        // to calculate proper RRG values
        for (symbol, _price_data) in current_data {
            if symbol != &self.config.benchmark_symbol {
                let mut rrg_data = RRGData::new(symbol.clone(), "Unknown".to_string());
                // Placeholder RRG calculation - would need proper implementation
                rrg_data.normalized_rs_ratio = vec![100.0];
                rrg_data.normalized_rs_momentum = vec![100.0];
                rrg_results.insert(symbol.clone(), rrg_data);
            }
        }

        Ok(rrg_results)
    }

    fn generate_alerts(&mut self, rrg_results: &HashMap<String, RRGData>) -> MLResult<Vec<Alert>> {
        let mut all_alerts = Vec::new();

        for rrg_data in rrg_results.values() {
            let alerts = self.alert_system.process_data(rrg_data)
                .map_err(|e| MLError::feature_error(format!("Alert generation failed: {}", e)))?;
            all_alerts.extend(alerts);
        }

        Ok(all_alerts)
    }

    fn update_portfolio_prices(
        &self,
        portfolio: &mut HashMap<String, Position>,
        current_data: &HashMap<String, OHLCVData>,
    ) -> MLResult<()> {
        for (symbol, position) in portfolio.iter_mut() {
            if let Some(price_data) = current_data.get(symbol) {
                position.update_price(price_data.close);
            }
        }
        Ok(())
    }

    fn should_rebalance(&self, current_date: DateTime<Utc>) -> MLResult<bool> {
        if self.portfolio_history.is_empty() {
            return Ok(true); // First rebalance
        }

        let last_rebalance = self.trades
            .iter()
            .filter(|trade| matches!(trade.action, TradeAction::Rebalance))
            .map(|trade| trade.timestamp)
            .max()
            .unwrap_or(self.config.start_date);

        let days_since_rebalance = (current_date - last_rebalance).num_days();
        Ok(days_since_rebalance >= self.config.rebalance_frequency_days)
    }

    fn rebalance_portfolio(
        &mut self,
        portfolio: &mut HashMap<String, Position>,
        rrg_results: &HashMap<String, RRGData>,
        current_date: DateTime<Utc>,
    ) -> MLResult<()> {
        // Calculate total portfolio value
        let total_value: f64 = portfolio.values().map(|p| p.market_value).sum();
        
        // Determine target weights based on RRG positions
        let target_weights = self.calculate_target_weights(rrg_results)?;
        
        // Execute rebalancing trades
        for (symbol, target_weight) in target_weights {
            if let Some(position) = portfolio.get_mut(&symbol) {
                let target_value = total_value * target_weight;
                let current_value = position.market_value;
                let difference = target_value - current_value;
                
                if difference.abs() > total_value * 0.01 { // Only trade if difference > 1%
                    let shares_to_trade = difference / position.current_price;
                    let action = if shares_to_trade > 0.0 { TradeAction::Buy } else { TradeAction::Sell };
                    
                    // Calculate transaction cost
                    let transaction_cost = difference.abs() * self.config.transaction_cost_bps / 10000.0;
                    
                    // Create trade record
                    let trade = Trade {
                        id: Uuid::new_v4(),
                        timestamp: current_date,
                        symbol: symbol.clone(),
                        action,
                        shares: shares_to_trade.abs(),
                        price: position.current_price,
                        value: difference.abs(),
                        transaction_cost,
                        reason: "Rebalancing based on RRG signals".to_string(),
                    };
                    
                    self.trades.push(trade);
                    
                    // Update position
                    position.shares += shares_to_trade;
                    position.market_value = position.shares * position.current_price;
                }
            }
        }
        
        Ok(())
    }

    fn calculate_target_weights(&self, rrg_results: &HashMap<String, RRGData>) -> MLResult<HashMap<String, f64>> {
        let mut weights = HashMap::new();
        let num_etfs = rrg_results.len() as f64;
        
        // Simple equal-weight strategy for now
        // In practice, this would use RRG quadrant positions to determine weights
        for symbol in rrg_results.keys() {
            weights.insert(symbol.clone(), 1.0 / num_etfs);
        }
        
        Ok(weights)
    }

    fn create_portfolio_snapshot(
        &self,
        portfolio: &HashMap<String, Position>,
        timestamp: DateTime<Utc>,
        benchmark_data: &ETFData,
    ) -> MLResult<PortfolioSnapshot> {
        let total_value: f64 = portfolio.values().map(|p| p.market_value).sum();
        let cash = 0.0; // Simplified - assume fully invested
        
        // Calculate returns
        let total_return = (total_value - self.config.initial_capital) / self.config.initial_capital;
        
        // Calculate benchmark return (simplified)
        let benchmark_return = self.calculate_benchmark_return(benchmark_data, timestamp)?;
        
        // Calculate other metrics (simplified)
        let alpha = total_return - benchmark_return;
        let beta = 1.0; // Placeholder
        let sharpe_ratio = self.calculate_sharpe_ratio()?;
        let max_drawdown = self.calculate_max_drawdown()?;
        
        // Update position weights
        let mut positions = portfolio.clone();
        for position in positions.values_mut() {
            position.weight = position.market_value / total_value;
        }
        
        Ok(PortfolioSnapshot {
            timestamp,
            positions,
            cash,
            total_value,
            total_return,
            benchmark_return,
            alpha,
            beta,
            sharpe_ratio,
            max_drawdown,
        })
    }

    fn calculate_benchmark_return(&self, benchmark_data: &ETFData, current_date: DateTime<Utc>) -> MLResult<f64> {
        if let Some(start_price) = self.find_price_data_for_date(&benchmark_data.price_history, self.config.start_date) {
            if let Some(current_price) = self.find_price_data_for_date(&benchmark_data.price_history, current_date) {
                return Ok((current_price.close - start_price.close) / start_price.close);
            }
        }
        Ok(0.0)
    }

    fn calculate_sharpe_ratio(&self) -> MLResult<f64> {
        if self.portfolio_history.len() < 2 {
            return Ok(0.0);
        }
        
        let returns: Vec<f64> = self.portfolio_history
            .windows(2)
            .map(|window| {
                let prev = &window[0];
                let curr = &window[1];
                (curr.total_value - prev.total_value) / prev.total_value
            })
            .collect();
        
        if returns.is_empty() {
            return Ok(0.0);
        }
        
        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns.iter()
            .map(|r| (r - mean_return).powi(2))
            .sum::<f64>() / returns.len() as f64;
        let std_dev = variance.sqrt();
        
        if std_dev == 0.0 {
            Ok(0.0)
        } else {
            Ok(mean_return / std_dev * (252.0_f64).sqrt()) // Annualized
        }
    }

    fn calculate_max_drawdown(&self) -> MLResult<f64> {
        if self.portfolio_history.is_empty() {
            return Ok(0.0);
        }
        
        let mut max_value = self.portfolio_history[0].total_value;
        let mut max_drawdown = 0.0;
        
        for snapshot in &self.portfolio_history {
            if snapshot.total_value > max_value {
                max_value = snapshot.total_value;
            }
            
            let drawdown = (max_value - snapshot.total_value) / max_value;
            if drawdown > max_drawdown {
                max_drawdown = drawdown;
            }
        }
        
        Ok(max_drawdown)
    }

    fn calculate_metrics(&self, benchmark_data: &ETFData) -> MLResult<BacktestMetrics> {
        if self.portfolio_history.is_empty() {
            return Err(MLError::feature_error("No portfolio history available for metrics calculation"));
        }
        
        let first_snapshot = &self.portfolio_history[0];
        let last_snapshot = &self.portfolio_history[self.portfolio_history.len() - 1];
        
        let total_return = (last_snapshot.total_value - first_snapshot.total_value) / first_snapshot.total_value;
        let duration_days = (self.config.end_date - self.config.start_date).num_days() as f64;
        let duration_years = duration_days / 365.25;
        let annualized_return = (1.0 + total_return).powf(1.0 / duration_years) - 1.0;
        
        let benchmark_return = self.calculate_benchmark_return(benchmark_data, self.config.end_date)?;
        let excess_return = total_return - benchmark_return;
        
        let sharpe_ratio = self.calculate_sharpe_ratio()?;
        let max_drawdown = self.calculate_max_drawdown()?;
        
        // Calculate volatility
        let returns: Vec<f64> = self.portfolio_history
            .windows(2)
            .map(|window| (window[1].total_value - window[0].total_value) / window[0].total_value)
            .collect();
        
        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
        let volatility = (returns.iter()
            .map(|r| (r - mean_return).powi(2))
            .sum::<f64>() / returns.len() as f64)
            .sqrt() * (252.0_f64).sqrt(); // Annualized
        
        let calmar_ratio = if max_drawdown > 0.0 { annualized_return / max_drawdown } else { 0.0 };
        
        // Calculate win rate
        let winning_trades = self.trades.iter()
            .filter(|trade| matches!(trade.action, TradeAction::Sell))
            .filter(|_| true) // Simplified - would need to track P&L per trade
            .count();
        let total_trades = self.trades.iter()
            .filter(|trade| matches!(trade.action, TradeAction::Sell))
            .count();
        let win_rate = if total_trades > 0 { winning_trades as f64 / total_trades as f64 } else { 0.0 };
        
        Ok(BacktestMetrics {
            total_return,
            annualized_return,
            volatility,
            sharpe_ratio,
            max_drawdown,
            calmar_ratio,
            win_rate,
            profit_factor: 1.0, // Placeholder
            beta: 1.0, // Placeholder
            alpha: excess_return,
            information_ratio: if volatility > 0.0 { excess_return / volatility } else { 0.0 },
            tracking_error: volatility, // Simplified
            benchmark_return,
            excess_return,
        })
    }

    /// Compare multiple models or strategies
    pub fn compare_strategies(results: &[BacktestResults]) -> MLResult<StrategyComparison> {
        if results.is_empty() {
            return Err(MLError::feature_error("No backtest results provided for comparison"));
        }

        let mut comparison = StrategyComparison {
            strategies: Vec::new(),
            best_return: None,
            best_sharpe: None,
            best_calmar: None,
            lowest_drawdown: None,
        };

        for (i, result) in results.iter().enumerate() {
            let strategy_summary = StrategyPerformance {
                name: format!("Strategy {}", i + 1),
                total_return: result.metrics.total_return,
                annualized_return: result.metrics.annualized_return,
                volatility: result.metrics.volatility,
                sharpe_ratio: result.metrics.sharpe_ratio,
                max_drawdown: result.metrics.max_drawdown,
                calmar_ratio: result.metrics.calmar_ratio,
                win_rate: result.metrics.win_rate,
            };

            // Track best performers
            if comparison.best_return.is_none() || strategy_summary.total_return > comparison.best_return.as_ref().unwrap().total_return {
                comparison.best_return = Some(strategy_summary.clone());
            }

            if comparison.best_sharpe.is_none() || strategy_summary.sharpe_ratio > comparison.best_sharpe.as_ref().unwrap().sharpe_ratio {
                comparison.best_sharpe = Some(strategy_summary.clone());
            }

            if comparison.best_calmar.is_none() || strategy_summary.calmar_ratio > comparison.best_calmar.as_ref().unwrap().calmar_ratio {
                comparison.best_calmar = Some(strategy_summary.clone());
            }

            if comparison.lowest_drawdown.is_none() || strategy_summary.max_drawdown < comparison.lowest_drawdown.as_ref().unwrap().max_drawdown {
                comparison.lowest_drawdown = Some(strategy_summary.clone());
            }

            comparison.strategies.push(strategy_summary);
        }

        Ok(comparison)
    }
}

/// Strategy performance summary for comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyPerformance {
    pub name: String,
    pub total_return: f64,
    pub annualized_return: f64,
    pub volatility: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub calmar_ratio: f64,
    pub win_rate: f64,
}

/// Strategy comparison results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyComparison {
    pub strategies: Vec<StrategyPerformance>,
    pub best_return: Option<StrategyPerformance>,
    pub best_sharpe: Option<StrategyPerformance>,
    pub best_calmar: Option<StrategyPerformance>,
    pub lowest_drawdown: Option<StrategyPerformance>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backtest_config_default() {
        let config = BacktestConfig::default();
        assert_eq!(config.initial_capital, 100000.0);
        assert_eq!(config.rebalance_frequency_days, 30);
        assert_eq!(config.benchmark_symbol, "SPY");
    }

    #[test]
    fn test_position_creation() {
        let position = Position::new("AAPL".to_string(), 100.0, 150.0);
        assert_eq!(position.symbol, "AAPL");
        assert_eq!(position.shares, 100.0);
        assert_eq!(position.avg_cost, 150.0);
        assert_eq!(position.market_value, 15000.0);
    }

    #[test]
    fn test_position_price_update() {
        let mut position = Position::new("AAPL".to_string(), 100.0, 150.0);
        position.update_price(160.0);
        
        assert_eq!(position.current_price, 160.0);
        assert_eq!(position.market_value, 16000.0);
        assert_eq!(position.unrealized_pnl, 1000.0);
    }
}