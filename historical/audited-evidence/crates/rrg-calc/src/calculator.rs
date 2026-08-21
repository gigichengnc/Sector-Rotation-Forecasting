use crate::{RRGError, RRGResult, RRGData, Quadrant, PortfolioAnalysis, AssetContribution};
use rrg_data::{ETFData, OHLCVData, Portfolio, Holding};
use std::collections::HashMap;
use tracing::{info, warn, debug};

/// RRG calculation engine implementing JdK RS-Ratio and RS-Momentum calculations
#[derive(Debug, Clone)]
pub struct RRGCalculator {
    pub rs_ratio_period: usize,
    pub rs_momentum_period: usize,
    pub normalization_period: usize,
}

impl RRGCalculator {
    /// Create a new RRG calculator with standard JdK parameters
    pub fn new() -> Self {
        Self {
            rs_ratio_period: 10,      // 10-period RS-Ratio calculation
            rs_momentum_period: 10,   // 10-period RS-Momentum calculation  
            normalization_period: 100, // 100-period normalization
        }
    }
    
    /// Create a new RRG calculator with custom parameters
    pub fn with_periods(rs_ratio_period: usize, rs_momentum_period: usize, normalization_period: usize) -> Self {
        Self {
            rs_ratio_period,
            rs_momentum_period,
            normalization_period,
        }
    }
    
    /// Calculate RRG data for a single ETF against a benchmark
    pub fn calculate_rrg(&self, etf_data: &ETFData, benchmark_data: &ETFData) -> RRGResult<RRGData> {
        info!("Calculating RRG for {} against benchmark", etf_data.symbol);
        
        // Validate input data
        self.validate_data(etf_data, benchmark_data)?;
        
        // Align data by timestamps
        let (aligned_etf, aligned_benchmark) = self.align_data(etf_data, benchmark_data)?;
        
        if aligned_etf.len() < self.rs_ratio_period + self.rs_momentum_period {
            return Err(RRGError::insufficient_data(
                self.rs_ratio_period + self.rs_momentum_period,
                aligned_etf.len(),
            ));
        }
        
        // Calculate RS-Ratio
        let rs_ratio = self.calculate_rs_ratio(&aligned_etf, &aligned_benchmark)?;
        debug!("Calculated {} RS-Ratio values", rs_ratio.len());
        
        // Calculate RS-Momentum
        let rs_momentum = self.calculate_rs_momentum(&rs_ratio)?;
        debug!("Calculated {} RS-Momentum values", rs_momentum.len());
        
        // Normalize values
        let normalized_rs_ratio = self.normalize_values(&rs_ratio)?;
        let normalized_rs_momentum = self.normalize_values(&rs_momentum)?;
        
        // Create RRG data structure
        let mut rrg_data = RRGData::new(etf_data.symbol.clone(), etf_data.sector.clone());
        
        // Extract timestamps (aligned with RS-Momentum length)
        let start_idx = aligned_etf.len() - rs_momentum.len();
        rrg_data.timestamps = aligned_etf[start_idx..]
            .iter()
            .map(|data| data.timestamp)
            .collect();
        
        rrg_data.rs_ratio = rs_ratio[rs_ratio.len() - rs_momentum.len()..].to_vec();
        rrg_data.rs_momentum = rs_momentum;
        rrg_data.normalized_rs_ratio = normalized_rs_ratio[normalized_rs_ratio.len() - rrg_data.rs_momentum.len()..].to_vec();
        rrg_data.normalized_rs_momentum = normalized_rs_momentum;
        
        // Calculate quadrants
        rrg_data.quadrants = self.calculate_quadrants(&rrg_data.normalized_rs_ratio, &rrg_data.normalized_rs_momentum);
        
        // Set current quadrant and strength
        if let Some(&current_quadrant) = rrg_data.quadrants.last() {
            rrg_data.current_quadrant = current_quadrant;
        }
        
        if let Some(latest_point) = rrg_data.get_latest_point() {
            rrg_data.quadrant_strength = latest_point.strength;
        }
        
        info!("RRG calculation complete for {}: {} data points", etf_data.symbol, rrg_data.rs_ratio.len());
        Ok(rrg_data)
    }
    
    /// Calculate portfolio-weighted RRG data
    /// Aggregates individual asset RRG positions based on portfolio weights
    pub fn calculate_portfolio_rrg(
        &self,
        portfolio: &Portfolio,
        asset_rrg_data: &HashMap<String, RRGData>,
        benchmark_symbol: &str,
    ) -> RRGResult<RRGData> {
        info!("Calculating portfolio-weighted RRG for portfolio: {}", portfolio.name);
        
        if portfolio.holdings.is_empty() {
            return Err(RRGError::calculation("Portfolio has no holdings"));
        }
        
        // Find the minimum number of data points across all assets
        let min_data_points = asset_rrg_data
            .values()
            .map(|data| data.rs_ratio.len())
            .min()
            .unwrap_or(0);
            
        if min_data_points == 0 {
            return Err(RRGError::calculation("No RRG data available for portfolio assets"));
        }
        
        // Initialize portfolio RRG data
        let mut portfolio_rrg = RRGData::new(
            format!("Portfolio_{}", portfolio.name),
            "Portfolio".to_string(),
        );
        
        // Get timestamps from the first available asset (they should be aligned)
        if let Some(first_asset_data) = asset_rrg_data.values().next() {
            let start_idx = first_asset_data.timestamps.len() - min_data_points;
            portfolio_rrg.timestamps = first_asset_data.timestamps[start_idx..].to_vec();
        }
        
        // Calculate weighted averages for each time point
        portfolio_rrg.rs_ratio = Vec::with_capacity(min_data_points);
        portfolio_rrg.rs_momentum = Vec::with_capacity(min_data_points);
        portfolio_rrg.normalized_rs_ratio = Vec::with_capacity(min_data_points);
        portfolio_rrg.normalized_rs_momentum = Vec::with_capacity(min_data_points);
        
        for i in 0..min_data_points {
            let mut weighted_rs_ratio = 0.0;
            let mut weighted_rs_momentum = 0.0;
            let mut weighted_norm_rs_ratio = 0.0;
            let mut weighted_norm_rs_momentum = 0.0;
            let mut total_weight = 0.0;
            
            // Calculate weighted averages across all holdings
            for holding in portfolio.holdings.values() {
                if let Some(asset_data) = asset_rrg_data.get(&holding.symbol) {
                    let data_start_idx = asset_data.rs_ratio.len() - min_data_points;
                    let data_idx = data_start_idx + i;
                    
                    if data_idx < asset_data.rs_ratio.len() {
                        let weight = holding.current_weight;
                        
                        weighted_rs_ratio += asset_data.rs_ratio[data_idx] * weight;
                        weighted_rs_momentum += asset_data.rs_momentum[data_idx] * weight;
                        weighted_norm_rs_ratio += asset_data.normalized_rs_ratio[data_idx] * weight;
                        weighted_norm_rs_momentum += asset_data.normalized_rs_momentum[data_idx] * weight;
                        total_weight += weight;
                    }
                }
            }
            
            // Normalize by total weight (should be close to 1.0)
            if total_weight > 0.0 {
                portfolio_rrg.rs_ratio.push(weighted_rs_ratio / total_weight);
                portfolio_rrg.rs_momentum.push(weighted_rs_momentum / total_weight);
                portfolio_rrg.normalized_rs_ratio.push(weighted_norm_rs_ratio / total_weight);
                portfolio_rrg.normalized_rs_momentum.push(weighted_norm_rs_momentum / total_weight);
            } else {
                return Err(RRGError::calculation("No valid weights found for portfolio calculation"));
            }
        }
        
        // Calculate quadrants for portfolio
        portfolio_rrg.quadrants = self.calculate_quadrants(
            &portfolio_rrg.normalized_rs_ratio,
            &portfolio_rrg.normalized_rs_momentum,
        );
        
        // Set current quadrant and strength
        if let Some(&current_quadrant) = portfolio_rrg.quadrants.last() {
            portfolio_rrg.current_quadrant = current_quadrant;
        }
        
        if let Some(latest_point) = portfolio_rrg.get_latest_point() {
            portfolio_rrg.quadrant_strength = latest_point.strength;
        }
        
        info!(
            "Portfolio RRG calculation complete: {} data points, current quadrant: {:?}",
            portfolio_rrg.rs_ratio.len(),
            portfolio_rrg.current_quadrant
        );
        
        Ok(portfolio_rrg)
    }
    
    /// Calculate individual vs portfolio-weighted analysis
    pub fn calculate_portfolio_analysis(
        &self,
        portfolio: &Portfolio,
        individual_rrg_data: &HashMap<String, RRGData>,
        benchmark_symbol: &str,
    ) -> RRGResult<PortfolioAnalysis> {
        info!("Calculating portfolio analysis for: {}", portfolio.name);
        
        // Calculate portfolio-weighted RRG
        let portfolio_rrg = self.calculate_portfolio_rrg(portfolio, individual_rrg_data, benchmark_symbol)?;
        
        // Calculate contribution analysis
        let mut contributions = HashMap::new();
        
        for holding in portfolio.holdings.values() {
            if let Some(asset_data) = individual_rrg_data.get(&holding.symbol) {
                let contribution = AssetContribution {
                    symbol: holding.symbol.clone(),
                    weight: holding.current_weight,
                    current_quadrant: asset_data.current_quadrant,
                    quadrant_strength: asset_data.quadrant_strength,
                    rs_ratio_contribution: asset_data.normalized_rs_ratio.last().unwrap_or(&100.0) * holding.current_weight,
                    rs_momentum_contribution: asset_data.normalized_rs_momentum.last().unwrap_or(&100.0) * holding.current_weight,
                };
                contributions.insert(holding.symbol.clone(), contribution);
            }
        }
        
        Ok(PortfolioAnalysis {
            portfolio_name: portfolio.name.clone(),
            benchmark_symbol: benchmark_symbol.to_string(),
            portfolio_rrg: portfolio_rrg,
            individual_assets: individual_rrg_data.clone(),
            asset_contributions: contributions,
            total_holdings: portfolio.total_holdings(),
        })
    }
    
    /// Calculate RRG data for multiple ETFs against a benchmark
    pub fn calculate_multiple_rrg(&self, etf_data_list: &[ETFData], benchmark_data: &ETFData) -> Vec<RRGResult<RRGData>> {
        info!("Calculating RRG for {} ETFs", etf_data_list.len());
        
        etf_data_list
            .iter()
            .map(|etf_data| self.calculate_rrg(etf_data, benchmark_data))
            .collect()
    }
    
    /// Calculate RS-Ratio (Relative Strength Ratio)
    /// RS-Ratio = (ETF Price / ETF Price n-periods ago) / (Benchmark Price / Benchmark Price n-periods ago)
    fn calculate_rs_ratio(&self, etf_data: &[OHLCVData], benchmark_data: &[OHLCVData]) -> RRGResult<Vec<f64>> {
        if etf_data.len() != benchmark_data.len() {
            return Err(RRGError::BenchmarkMismatch {
                expected: etf_data.len(),
                actual: benchmark_data.len(),
            });
        }
        
        if etf_data.len() < self.rs_ratio_period {
            return Err(RRGError::insufficient_data(self.rs_ratio_period, etf_data.len()));
        }
        
        let mut rs_ratio = Vec::new();
        
        for i in self.rs_ratio_period..etf_data.len() {
            let etf_current = etf_data[i].close;
            let etf_previous = etf_data[i - self.rs_ratio_period].close;
            let benchmark_current = benchmark_data[i].close;
            let benchmark_previous = benchmark_data[i - self.rs_ratio_period].close;
            
            // Validate prices are positive
            if etf_current <= 0.0 || etf_previous <= 0.0 || benchmark_current <= 0.0 || benchmark_previous <= 0.0 {
                return Err(RRGError::calculation("Invalid price data: prices must be positive"));
            }
            
            // Calculate relative performance
            let etf_return = etf_current / etf_previous;
            let benchmark_return = benchmark_current / benchmark_previous;
            
            if benchmark_return == 0.0 {
                return Err(RRGError::calculation("Benchmark return is zero"));
            }
            
            let rs_value = etf_return / benchmark_return;
            rs_ratio.push(rs_value);
        }
        
        Ok(rs_ratio)
    }
    
    /// Calculate RS-Momentum (Rate of Change of RS-Ratio)
    /// RS-Momentum = (Current RS-Ratio / RS-Ratio n-periods ago) - 1
    fn calculate_rs_momentum(&self, rs_ratio: &[f64]) -> RRGResult<Vec<f64>> {
        if rs_ratio.len() < self.rs_momentum_period {
            return Err(RRGError::insufficient_data(self.rs_momentum_period, rs_ratio.len()));
        }
        
        let mut rs_momentum = Vec::new();
        
        for i in self.rs_momentum_period..rs_ratio.len() {
            let current_rs = rs_ratio[i];
            let previous_rs = rs_ratio[i - self.rs_momentum_period];
            
            if previous_rs <= 0.0 {
                return Err(RRGError::calculation("Invalid RS-Ratio: must be positive"));
            }
            
            let momentum = (current_rs / previous_rs) - 1.0;
            rs_momentum.push(momentum);
        }
        
        Ok(rs_momentum)
    }
    
    /// Normalize values to 0-200 scale with 100 as the center
    /// This is the standard JdK normalization for RRG charts
    fn normalize_values(&self, values: &[f64]) -> RRGResult<Vec<f64>> {
        if values.is_empty() {
            return Ok(Vec::new());
        }
        
        let period = self.normalization_period.min(values.len());
        let mut normalized = Vec::new();
        
        for i in 0..values.len() {
            let start_idx = if i >= period - 1 { i - period + 1 } else { 0 };
            let end_idx = i + 1;
            
            let window = &values[start_idx..end_idx];
            
            // Calculate mean and standard deviation for the window
            let mean = window.iter().sum::<f64>() / window.len() as f64;
            let variance = window.iter()
                .map(|x| (x - mean).powi(2))
                .sum::<f64>() / window.len() as f64;
            let std_dev = variance.sqrt();
            
            // Normalize: 100 + (value - mean) / std_dev * 10
            // This gives a scale where 100 is average, with typical range 70-130
            let normalized_value = if std_dev > 0.0 {
                100.0 + ((values[i] - mean) / std_dev) * 10.0
            } else {
                100.0 // If no variation, use center value
            };
            
            normalized.push(normalized_value);
        }
        
        Ok(normalized)
    }
    
    /// Calculate quadrants for each point
    fn calculate_quadrants(&self, rs_ratio: &[f64], rs_momentum: &[f64]) -> Vec<Quadrant> {
        rs_ratio.iter()
            .zip(rs_momentum.iter())
            .map(|(&ratio, &momentum)| {
                match (ratio >= 100.0, momentum >= 100.0) {
                    (true, true) => Quadrant::Leading,      // High RS-Ratio, High RS-Momentum
                    (true, false) => Quadrant::Weakening,   // High RS-Ratio, Low RS-Momentum
                    (false, false) => Quadrant::Lagging,    // Low RS-Ratio, Low RS-Momentum
                    (false, true) => Quadrant::Improving,   // Low RS-Ratio, High RS-Momentum
                }
            })
            .collect()
    }
    
    /// Validate input data
    fn validate_data(&self, etf_data: &ETFData, benchmark_data: &ETFData) -> RRGResult<()> {
        if etf_data.price_history.is_empty() {
            return Err(RRGError::calculation("ETF data is empty"));
        }
        
        if benchmark_data.price_history.is_empty() {
            return Err(RRGError::calculation("Benchmark data is empty"));
        }
        
        // Validate data quality
        etf_data.validate().map_err(|e| RRGError::calculation(format!("ETF data validation failed: {}", e)))?;
        benchmark_data.validate().map_err(|e| RRGError::calculation(format!("Benchmark data validation failed: {}", e)))?;
        
        Ok(())
    }
    
    /// Align ETF and benchmark data by timestamps
    fn align_data(&self, etf_data: &ETFData, benchmark_data: &ETFData) -> RRGResult<(Vec<OHLCVData>, Vec<OHLCVData>)> {
        let mut etf_map: HashMap<String, &OHLCVData> = HashMap::new();
        let mut benchmark_map: HashMap<String, &OHLCVData> = HashMap::new();
        
        // Create timestamp maps
        for data in &etf_data.price_history {
            let date_key = data.timestamp.format("%Y-%m-%d").to_string();
            etf_map.insert(date_key, data);
        }
        
        for data in &benchmark_data.price_history {
            let date_key = data.timestamp.format("%Y-%m-%d").to_string();
            benchmark_map.insert(date_key, data);
        }
        
        // Find common dates and align data
        let mut aligned_etf = Vec::new();
        let mut aligned_benchmark = Vec::new();
        
        // Get all dates and sort them
        let mut common_dates: Vec<String> = etf_map.keys()
            .filter(|date| benchmark_map.contains_key(*date))
            .cloned()
            .collect();
        common_dates.sort();
        
        for date in common_dates {
            if let (Some(&etf_data), Some(&benchmark_data)) = (etf_map.get(&date), benchmark_map.get(&date)) {
                aligned_etf.push(etf_data.clone());
                aligned_benchmark.push(benchmark_data.clone());
            }
        }
        
        if aligned_etf.is_empty() {
            return Err(RRGError::calculation("No overlapping dates found between ETF and benchmark data"));
        }
        
        info!("Aligned {} data points between ETF and benchmark", aligned_etf.len());
        Ok((aligned_etf, aligned_benchmark))
    }
}

impl Default for RRGCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Utc, Duration};
    use rrg_data::OHLCVData;
    
    fn create_test_data(symbol: &str, days: usize, base_price: f64) -> ETFData {
        let mut etf_data = ETFData::new(symbol.to_string(), format!("{} ETF", symbol), "Technology".to_string());
        
        for i in 0..days {
            let price = base_price + (i as f64 * 0.1); // Slight upward trend
            let ohlcv = OHLCVData {
                timestamp: Utc::now() - Duration::days((days - i - 1) as i64),
                open: price,
                high: price + 1.0,
                low: price - 1.0,
                close: price,
                volume: 1000000,
                adjusted_close: Some(price),
            };
            etf_data.add_price_data(ohlcv);
        }
        
        etf_data
    }
    
    #[test]
    fn test_rrg_calculator_creation() {
        let calc = RRGCalculator::new();
        assert_eq!(calc.rs_ratio_period, 10);
        assert_eq!(calc.rs_momentum_period, 10);
        assert_eq!(calc.normalization_period, 100);
    }
    
    #[test]
    fn test_rrg_calculator_custom_periods() {
        let calc = RRGCalculator::with_periods(5, 5, 50);
        assert_eq!(calc.rs_ratio_period, 5);
        assert_eq!(calc.rs_momentum_period, 5);
        assert_eq!(calc.normalization_period, 50);
    }
    
    #[test]
    fn test_rs_ratio_calculation() {
        let calc = RRGCalculator::new();
        let etf_data = create_test_data("XLK", 50, 100.0);
        let benchmark_data = create_test_data("SPY", 50, 200.0);
        
        let (aligned_etf, aligned_benchmark) = calc.align_data(&etf_data, &benchmark_data).unwrap();
        let rs_ratio = calc.calculate_rs_ratio(&aligned_etf, &aligned_benchmark).unwrap();
        
        assert!(!rs_ratio.is_empty());
        assert_eq!(rs_ratio.len(), aligned_etf.len() - calc.rs_ratio_period);
        
        // All RS-Ratio values should be positive
        for &ratio in &rs_ratio {
            assert!(ratio > 0.0);
        }
    }
    
    #[test]
    fn test_rs_momentum_calculation() {
        let calc = RRGCalculator::new();
        let rs_ratio = vec![1.0, 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 1.8, 1.9, 2.0, 2.1, 2.2];
        
        let rs_momentum = calc.calculate_rs_momentum(&rs_ratio).unwrap();
        
        assert!(!rs_momentum.is_empty());
        assert_eq!(rs_momentum.len(), rs_ratio.len() - calc.rs_momentum_period);
        
        // With increasing RS-Ratio, momentum should be positive
        for &momentum in &rs_momentum {
            assert!(momentum > 0.0);
        }
    }
    
    #[test]
    fn test_normalization() {
        let calc = RRGCalculator::new();
        let values = vec![1.0, 1.1, 0.9, 1.2, 0.8, 1.3, 0.7, 1.4, 0.6, 1.5];
        
        let normalized = calc.normalize_values(&values).unwrap();
        
        assert_eq!(normalized.len(), values.len());
        
        // Check that values are reasonably distributed around 100
        let mean = normalized.iter().sum::<f64>() / normalized.len() as f64;
        assert!((mean - 100.0).abs() < 20.0); // Should be close to 100
    }
    
    #[test]
    fn test_full_rrg_calculation() {
        let calc = RRGCalculator::new();
        let etf_data = create_test_data("XLK", 150, 100.0);
        let benchmark_data = create_test_data("SPY", 150, 200.0);
        
        let rrg_data = calc.calculate_rrg(&etf_data, &benchmark_data).unwrap();
        
        assert_eq!(rrg_data.symbol, "XLK");
        assert!(!rrg_data.rs_ratio.is_empty());
        assert!(!rrg_data.rs_momentum.is_empty());
        assert_eq!(rrg_data.rs_ratio.len(), rrg_data.rs_momentum.len());
        assert_eq!(rrg_data.quadrants.len(), rrg_data.rs_momentum.len());
    }
}