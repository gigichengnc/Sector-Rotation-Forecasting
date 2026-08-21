use crate::{MLError, MLResult};
use rrg_data::{ETFData, OHLCVData};
use rrg_calc::{RRGData, RRGPoint, Quadrant};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use tracing::{info, debug, warn};

/// Feature engineering configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureConfig {
    pub sequence_length: usize,
    pub prediction_horizon: usize,
    pub include_technical_indicators: bool,
    pub include_volume_features: bool,
    pub include_external_features: bool,
    pub normalization_method: NormalizationMethod,
}

impl Default for FeatureConfig {
    fn default() -> Self {
        Self {
            sequence_length: 20,        // 20-day sequences
            prediction_horizon: 5,      // Predict 5 days ahead
            include_technical_indicators: true,
            include_volume_features: true,
            include_external_features: false,
            normalization_method: NormalizationMethod::ZScore,
        }
    }
}

// =============================================================================
// External Features (VIX, Interest Rates) - Task 3.5
// Property 8: External Feature Integration
// Validates: Requirements 2.6
// =============================================================================

/// External market features for LSTM input enhancement
/// Includes VIX (volatility index) and interest rate data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalFeatures {
    /// VIX values (volatility index, typically 10-80)
    pub vix: Vec<f64>,
    /// Interest rate values (10-year Treasury yield, typically 0-20%)
    pub interest_rate: Vec<f64>,
    /// Timestamps for the external data
    pub timestamps: Vec<DateTime<Utc>>,
}

impl ExternalFeatures {
    /// Create new external features with validation
    pub fn new(vix: Vec<f64>, interest_rate: Vec<f64>, timestamps: Vec<DateTime<Utc>>) -> MLResult<Self> {
        if vix.len() != interest_rate.len() || vix.len() != timestamps.len() {
            return Err(MLError::feature_error("External feature vectors must have same length"));
        }
        
        // Validate VIX range (5-100 per Property 27)
        for (i, &v) in vix.iter().enumerate() {
            if !v.is_finite() {
                return Err(MLError::feature_error(format!("VIX[{}] is not finite: {}", i, v)));
            }
            if v < 0.0 || v > 150.0 {
                warn!("VIX[{}] value {} outside typical range [5, 100]", i, v);
            }
        }
        
        // Validate interest rate range (0-20% per Property 27)
        for (i, &r) in interest_rate.iter().enumerate() {
            if !r.is_finite() {
                return Err(MLError::feature_error(format!("Interest rate[{}] is not finite: {}", i, r)));
            }
            if r < -5.0 || r > 30.0 {
                warn!("Interest rate[{}] value {} outside typical range [0, 20]", i, r);
            }
        }
        
        Ok(Self { vix, interest_rate, timestamps })
    }
    
    /// Create empty external features (for when external data is not available)
    pub fn empty() -> Self {
        Self {
            vix: Vec::new(),
            interest_rate: Vec::new(),
            timestamps: Vec::new(),
        }
    }
    
    /// Check if external features are available
    pub fn is_empty(&self) -> bool {
        self.vix.is_empty()
    }
    
    /// Get the length of external features
    pub fn len(&self) -> usize {
        self.vix.len()
    }
    
    /// Normalize VIX to standard score (z-score)
    /// Property 29: Feature Normalization Range - values should be in [-3, 3]
    pub fn normalize_vix(&self) -> Vec<f64> {
        if self.vix.is_empty() {
            return Vec::new();
        }
        
        let mean = self.vix.iter().sum::<f64>() / self.vix.len() as f64;
        let variance = self.vix.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / self.vix.len() as f64;
        let std_dev = variance.sqrt();
        
        if std_dev == 0.0 {
            return vec![0.0; self.vix.len()];
        }
        
        self.vix.iter()
            .map(|&x| ((x - mean) / std_dev).clamp(-3.0, 3.0))
            .collect()
    }
    
    /// Normalize interest rate to standard score (z-score)
    /// Property 29: Feature Normalization Range - values should be in [-3, 3]
    pub fn normalize_interest_rate(&self) -> Vec<f64> {
        if self.interest_rate.is_empty() {
            return Vec::new();
        }
        
        let mean = self.interest_rate.iter().sum::<f64>() / self.interest_rate.len() as f64;
        let variance = self.interest_rate.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / self.interest_rate.len() as f64;
        let std_dev = variance.sqrt();
        
        if std_dev == 0.0 {
            return vec![0.0; self.interest_rate.len()];
        }
        
        self.interest_rate.iter()
            .map(|&x| ((x - mean) / std_dev).clamp(-3.0, 3.0))
            .collect()
    }
    
    /// Get normalized external features as a combined vector
    pub fn get_normalized_features(&self) -> (Vec<f64>, Vec<f64>) {
        (self.normalize_vix(), self.normalize_interest_rate())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NormalizationMethod {
    ZScore,
    MinMax,
    Robust,
}

/// Feature engineering system for RRG ML pipeline
#[derive(Debug)]
pub struct FeatureEngineer {
    pub config: FeatureConfig,
    pub technical_indicators: TechnicalIndicators,
    /// External features (VIX, interest rates) for enhanced predictions
    pub external_features: Option<ExternalFeatures>,
}

impl FeatureEngineer {
    pub fn new(config: FeatureConfig) -> Self {
        Self {
            technical_indicators: TechnicalIndicators::new(),
            config,
            external_features: None,
        }
    }
    
    /// Create a new FeatureEngineer with external features
    /// Property 8: External Feature Integration
    pub fn with_external_features(config: FeatureConfig, external_features: ExternalFeatures) -> Self {
        Self {
            technical_indicators: TechnicalIndicators::new(),
            config,
            external_features: Some(external_features),
        }
    }
    
    /// Set external features for the engineer
    pub fn set_external_features(&mut self, external_features: ExternalFeatures) {
        self.external_features = Some(external_features);
    }
    
    /// Check if external features are available
    pub fn has_external_features(&self) -> bool {
        self.external_features.as_ref().map_or(false, |ef| !ef.is_empty())
    }
    
    /// Create features from multiple RRG datasets for training
    pub fn create_features_from_rrg_data(&self, rrg_data: &[RRGData], etf_data: &[ETFData]) -> MLResult<FeatureSet> {
        info!("Creating features from {} RRG datasets", rrg_data.len());
        
        if rrg_data.len() != etf_data.len() {
            return Err(MLError::feature_error("RRG data and ETF data length mismatch"));
        }
        
        let mut all_sequences = Vec::new();
        let mut all_targets = Vec::new();
        let mut metadata = Vec::new();
        
        for (rrg, etf) in rrg_data.iter().zip(etf_data.iter()) {
            let (sequences, targets, meta) = self.create_sequences_from_single_dataset(rrg, etf)?;
            all_sequences.extend(sequences);
            all_targets.extend(targets);
            metadata.extend(meta);
        }
        
        info!("Generated {} training sequences", all_sequences.len());
        
        Ok(FeatureSet {
            sequences: all_sequences,
            targets: all_targets,
            metadata,
            feature_names: self.get_feature_names(),
            config: self.config.clone(),
        })
    }
    
    /// Create features from a single RRG dataset for prediction
    pub fn create_features_from_single_rrg(&self, rrg_data: &RRGData, etf_data: &ETFData) -> MLResult<Vec<Vec<f64>>> {
        debug!("Creating features for prediction from {}", rrg_data.symbol);
        
        let (sequences, _, _) = self.create_sequences_from_single_dataset(rrg_data, etf_data)?;
        
        // For prediction, we typically want the most recent sequence
        if sequences.is_empty() {
            return Err(MLError::feature_error("No sequences generated for prediction"));
        }
        
        // Return the last few sequences for ensemble prediction
        let num_sequences = 3.min(sequences.len());
        let start_idx = sequences.len() - num_sequences;
        
        Ok(sequences[start_idx..].to_vec())
    }
    
    /// Create sequences from a single RRG and ETF dataset
    fn create_sequences_from_single_dataset(&self, rrg_data: &RRGData, etf_data: &ETFData) -> MLResult<(Vec<Vec<f64>>, Vec<Vec<f64>>, Vec<SequenceMetadata>)> {
        if rrg_data.rs_ratio.len() < self.config.sequence_length + self.config.prediction_horizon {
            return Err(MLError::feature_error(format!(
                "Insufficient data: need at least {} points, got {}",
                self.config.sequence_length + self.config.prediction_horizon,
                rrg_data.rs_ratio.len()
            )));
        }
        
        // Create base features from RRG data
        let base_features = self.create_base_rrg_features(rrg_data)?;
        
        // Add technical indicators from price data
        let technical_features = if self.config.include_technical_indicators {
            self.create_technical_features(etf_data)?
        } else {
            vec![vec![0.0; etf_data.price_history.len()]; 0] // Empty technical features
        };
        
        // Add volume features
        let volume_features = if self.config.include_volume_features {
            self.create_volume_features(etf_data)?
        } else {
            vec![vec![0.0; etf_data.price_history.len()]; 0] // Empty volume features
        };
        
        // Combine all features
        let combined_features = self.combine_features(&base_features, &technical_features, &volume_features)?;
        
        // Add external features if available and configured
        // Property 8: External Feature Integration
        let features_with_external = if self.config.include_external_features && self.has_external_features() {
            self.add_external_features(&combined_features)?
        } else {
            combined_features
        };
        
        // Normalize features
        let normalized_features = self.normalize_features(&features_with_external)?;
        
        // Create sequences and targets
        self.create_sequences_and_targets(&normalized_features, rrg_data)
    }
    
    /// Add external features (VIX, interest rates) to the feature set
    /// Property 8: External Feature Integration
    /// Validates: Requirements 2.6
    fn add_external_features(&self, base_features: &[Vec<f64>]) -> MLResult<Vec<Vec<f64>>> {
        let external = self.external_features.as_ref()
            .ok_or_else(|| MLError::feature_error("External features not available"))?;
        
        if base_features.is_empty() {
            return Err(MLError::feature_error("Base features cannot be empty"));
        }
        
        let data_length = base_features[0].len();
        
        // Get normalized external features
        let (normalized_vix, normalized_interest_rate) = external.get_normalized_features();
        
        // Align external features with base features length
        let aligned_vix = self.align_external_feature(&normalized_vix, data_length)?;
        let aligned_interest_rate = self.align_external_feature(&normalized_interest_rate, data_length)?;
        
        // Combine base features with external features
        let mut combined = base_features.to_vec();
        combined.push(aligned_vix);
        combined.push(aligned_interest_rate);
        
        debug!("Added external features: VIX and interest rate (total features: {})", combined.len());
        
        Ok(combined)
    }
    
    /// Align external feature vector to match the required data length
    fn align_external_feature(&self, feature: &[f64], target_length: usize) -> MLResult<Vec<f64>> {
        if feature.is_empty() {
            // Return zeros if no external data available
            return Ok(vec![0.0; target_length]);
        }
        
        if feature.len() == target_length {
            return Ok(feature.to_vec());
        }
        
        if feature.len() > target_length {
            // Truncate from the end (use most recent data)
            let start = feature.len() - target_length;
            return Ok(feature[start..].to_vec());
        }
        
        // Pad with the first value if external data is shorter
        let mut aligned = vec![feature[0]; target_length - feature.len()];
        aligned.extend_from_slice(feature);
        Ok(aligned)
    }
    
    /// Create base RRG features
    fn create_base_rrg_features(&self, rrg_data: &RRGData) -> MLResult<Vec<Vec<f64>>> {
        let mut features = Vec::new();
        
        // RS-Ratio (raw and normalized)
        features.push(rrg_data.rs_ratio.clone());
        features.push(rrg_data.normalized_rs_ratio.clone());
        
        // RS-Momentum (raw and normalized)
        features.push(rrg_data.rs_momentum.clone());
        features.push(rrg_data.normalized_rs_momentum.clone());
        
        // Quadrant encoding (one-hot)
        let quadrant_features = self.encode_quadrants(&rrg_data.quadrants);
        features.extend(quadrant_features);
        
        // Distance from center (strength)
        let strength_features = self.calculate_strength_features(rrg_data);
        features.push(strength_features);
        
        // Velocity features (rate of change)
        let velocity_features = self.calculate_velocity_features(rrg_data)?;
        features.extend(velocity_features);
        
        Ok(features)
    }
    
    /// Create technical indicator features
    fn create_technical_features(&self, etf_data: &ETFData) -> MLResult<Vec<Vec<f64>>> {
        let prices: Vec<f64> = etf_data.price_history.iter().map(|p| p.close).collect();
        let highs: Vec<f64> = etf_data.price_history.iter().map(|p| p.high).collect();
        let lows: Vec<f64> = etf_data.price_history.iter().map(|p| p.low).collect();
        
        let mut features = Vec::new();
        
        // Moving averages
        features.push(self.technical_indicators.sma(&prices, 10)?);
        features.push(self.technical_indicators.sma(&prices, 20)?);
        features.push(self.technical_indicators.ema(&prices, 12)?);
        features.push(self.technical_indicators.ema(&prices, 26)?);
        
        // MACD
        let macd = self.technical_indicators.macd(&prices, 12, 26, 9)?;
        features.push(macd.macd);
        features.push(macd.signal);
        features.push(macd.histogram);
        
        // RSI
        features.push(self.technical_indicators.rsi(&prices, 14)?);
        
        // Bollinger Bands
        let bb = self.technical_indicators.bollinger_bands(&prices, 20, 2.0)?;
        features.push(bb.upper);
        features.push(bb.middle);
        features.push(bb.lower);
        
        // Stochastic
        let stoch = self.technical_indicators.stochastic(&highs, &lows, &prices, 14, 3)?;
        features.push(stoch.k);
        features.push(stoch.d);
        
        Ok(features)
    }
    
    /// Create volume-based features
    fn create_volume_features(&self, etf_data: &ETFData) -> MLResult<Vec<Vec<f64>>> {
        let volumes: Vec<f64> = etf_data.price_history.iter().map(|p| p.volume as f64).collect();
        let prices: Vec<f64> = etf_data.price_history.iter().map(|p| p.close).collect();
        
        let mut features = Vec::new();
        
        // Volume moving averages
        features.push(self.technical_indicators.sma(&volumes, 10)?);
        features.push(self.technical_indicators.sma(&volumes, 20)?);
        
        // Volume rate of change
        features.push(self.technical_indicators.rate_of_change(&volumes, 10)?);
        
        // On-Balance Volume (OBV)
        features.push(self.technical_indicators.obv(&prices, &volumes)?);
        
        // Volume-Price Trend (VPT)
        features.push(self.technical_indicators.vpt(&prices, &volumes)?);
        
        Ok(features)
    }
    
    /// Encode quadrants as one-hot vectors
    fn encode_quadrants(&self, quadrants: &[Quadrant]) -> Vec<Vec<f64>> {
        let mut leading = Vec::new();
        let mut weakening = Vec::new();
        let mut lagging = Vec::new();
        let mut improving = Vec::new();
        
        for &quadrant in quadrants {
            match quadrant {
                Quadrant::Leading => {
                    leading.push(1.0);
                    weakening.push(0.0);
                    lagging.push(0.0);
                    improving.push(0.0);
                }
                Quadrant::Weakening => {
                    leading.push(0.0);
                    weakening.push(1.0);
                    lagging.push(0.0);
                    improving.push(0.0);
                }
                Quadrant::Lagging => {
                    leading.push(0.0);
                    weakening.push(0.0);
                    lagging.push(1.0);
                    improving.push(0.0);
                }
                Quadrant::Improving => {
                    leading.push(0.0);
                    weakening.push(0.0);
                    lagging.push(0.0);
                    improving.push(1.0);
                }
            }
        }
        
        vec![leading, weakening, lagging, improving]
    }
    
    /// Calculate strength features (distance from center)
    fn calculate_strength_features(&self, rrg_data: &RRGData) -> Vec<f64> {
        rrg_data.normalized_rs_ratio.iter()
            .zip(rrg_data.normalized_rs_momentum.iter())
            .map(|(&ratio, &momentum)| {
                ((ratio - 100.0).powi(2) + (momentum - 100.0).powi(2)).sqrt()
            })
            .collect()
    }
    
    /// Calculate velocity features (rate of change in position)
    fn calculate_velocity_features(&self, rrg_data: &RRGData) -> MLResult<Vec<Vec<f64>>> {
        let mut features = Vec::new();
        
        // RS-Ratio velocity
        features.push(self.technical_indicators.rate_of_change(&rrg_data.normalized_rs_ratio, 1)?);
        features.push(self.technical_indicators.rate_of_change(&rrg_data.normalized_rs_ratio, 5)?);
        
        // RS-Momentum velocity
        features.push(self.technical_indicators.rate_of_change(&rrg_data.normalized_rs_momentum, 1)?);
        features.push(self.technical_indicators.rate_of_change(&rrg_data.normalized_rs_momentum, 5)?);
        
        Ok(features)
    }
    
    /// Combine all feature vectors
    fn combine_features(&self, base: &[Vec<f64>], technical: &[Vec<f64>], volume: &[Vec<f64>]) -> MLResult<Vec<Vec<f64>>> {
        if base.is_empty() {
            return Err(MLError::feature_error("Base features cannot be empty"));
        }
        
        let length = base[0].len();
        
        // Verify all feature vectors have the same length
        for feature_set in [base, technical, volume].iter() {
            for feature in feature_set.iter() {
                if feature.len() != length {
                    return Err(MLError::feature_error("Feature length mismatch"));
                }
            }
        }
        
        let mut combined = base.to_vec();
        combined.extend_from_slice(technical);
        combined.extend_from_slice(volume);
        
        Ok(combined)
    }
    
    /// Normalize features based on configuration
    pub fn normalize_features(&self, features: &[Vec<f64>]) -> MLResult<Vec<Vec<f64>>> {
        match self.config.normalization_method {
            NormalizationMethod::ZScore => self.z_score_normalize(features),
            NormalizationMethod::MinMax => self.min_max_normalize(features),
            NormalizationMethod::Robust => self.robust_normalize(features),
        }
    }
    
    /// Z-score normalization
    fn z_score_normalize(&self, features: &[Vec<f64>]) -> MLResult<Vec<Vec<f64>>> {
        let mut normalized = Vec::new();
        
        for feature in features {
            let mean = feature.iter().sum::<f64>() / feature.len() as f64;
            let variance = feature.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / feature.len() as f64;
            let std_dev = variance.sqrt();
            
            if std_dev == 0.0 {
                // If no variation, keep original values
                normalized.push(feature.clone());
            } else {
                let norm_feature = feature.iter().map(|x| (x - mean) / std_dev).collect();
                normalized.push(norm_feature);
            }
        }
        
        Ok(normalized)
    }
    
    /// Min-max normalization
    fn min_max_normalize(&self, features: &[Vec<f64>]) -> MLResult<Vec<Vec<f64>>> {
        let mut normalized = Vec::new();
        
        for feature in features {
            let min_val = feature.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max_val = feature.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            
            if min_val == max_val {
                // If no variation, keep original values
                normalized.push(feature.clone());
            } else {
                let norm_feature = feature.iter().map(|x| (x - min_val) / (max_val - min_val)).collect();
                normalized.push(norm_feature);
            }
        }
        
        Ok(normalized)
    }
    
    /// Robust normalization (using median and IQR)
    fn robust_normalize(&self, features: &[Vec<f64>]) -> MLResult<Vec<Vec<f64>>> {
        let mut normalized = Vec::new();
        
        for feature in features {
            let mut sorted_feature = feature.clone();
            sorted_feature.sort_by(|a, b| a.partial_cmp(b).unwrap());
            
            let len = sorted_feature.len();
            let q1_idx = len / 4;
            let q3_idx = 3 * len / 4;
            let median_idx = len / 2;
            
            let median = sorted_feature[median_idx];
            let q1 = sorted_feature[q1_idx];
            let q3 = sorted_feature[q3_idx];
            let iqr = q3 - q1;
            
            if iqr == 0.0 {
                // If no variation, keep original values
                normalized.push(feature.clone());
            } else {
                let norm_feature = feature.iter().map(|x| (x - median) / iqr).collect();
                normalized.push(norm_feature);
            }
        }
        
        Ok(normalized)
    }
    
    /// Create sequences and targets for training
    fn create_sequences_and_targets(&self, features: &[Vec<f64>], rrg_data: &RRGData) -> MLResult<(Vec<Vec<f64>>, Vec<Vec<f64>>, Vec<SequenceMetadata>)> {
        if features.is_empty() {
            return Err(MLError::feature_error("No features provided"));
        }
        
        let data_length = features[0].len();
        let num_features = features.len();
        
        let mut sequences = Vec::new();
        let mut targets = Vec::new();
        let mut metadata = Vec::new();
        
        // Create sliding window sequences
        for i in 0..=(data_length - self.config.sequence_length - self.config.prediction_horizon) {
            // Create input sequence
            let mut sequence = Vec::new();
            for t in i..(i + self.config.sequence_length) {
                for feature in features {
                    sequence.push(feature[t]);
                }
            }
            
            // Create target sequence (future RS-Ratio and RS-Momentum)
            let mut target = Vec::new();
            for t in (i + self.config.sequence_length)..(i + self.config.sequence_length + self.config.prediction_horizon) {
                target.push(rrg_data.normalized_rs_ratio[t]);
                target.push(rrg_data.normalized_rs_momentum[t]);
            }
            
            sequences.push(sequence);
            targets.push(target);
            
            // Create metadata
            let start_timestamp = rrg_data.timestamps[i];
            let end_timestamp = rrg_data.timestamps[i + self.config.sequence_length - 1];
            let target_timestamp = rrg_data.timestamps[i + self.config.sequence_length + self.config.prediction_horizon - 1];
            
            metadata.push(SequenceMetadata {
                symbol: rrg_data.symbol.clone(),
                sector: rrg_data.sector.clone(),
                sequence_start: start_timestamp,
                sequence_end: end_timestamp,
                target_timestamp,
                sequence_length: self.config.sequence_length,
                num_features,
            });
        }
        
        info!("Created {} sequences with {} features each", sequences.len(), num_features);
        
        Ok((sequences, targets, metadata))
    }
    
    /// Convert predictions back to RRG points
    pub fn predictions_to_rrg_points(&self, predictions: &[Vec<f64>], symbol: String) -> MLResult<Vec<RRGPoint>> {
        let mut points = Vec::new();
        
        for (i, prediction) in predictions.iter().enumerate() {
            if prediction.len() < 2 {
                return Err(MLError::prediction("Prediction must contain at least RS-Ratio and RS-Momentum"));
            }
            
            let rs_ratio = prediction[0];
            let rs_momentum = prediction[1];
            
            // Create timestamp (this would be based on the prediction horizon)
            let timestamp = Utc::now() + chrono::Duration::days(i as i64 + 1);
            
            let point = RRGPoint::new(symbol.clone(), timestamp, rs_ratio, rs_momentum);
            points.push(point);
        }
        
        Ok(points)
    }
    
    /// Get feature names for interpretability
    pub fn get_feature_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        
        // Base RRG features
        names.extend(vec![
            "rs_ratio".to_string(),
            "normalized_rs_ratio".to_string(),
            "rs_momentum".to_string(),
            "normalized_rs_momentum".to_string(),
            "quadrant_leading".to_string(),
            "quadrant_weakening".to_string(),
            "quadrant_lagging".to_string(),
            "quadrant_improving".to_string(),
            "strength".to_string(),
            "rs_ratio_velocity_1d".to_string(),
            "rs_ratio_velocity_5d".to_string(),
            "rs_momentum_velocity_1d".to_string(),
            "rs_momentum_velocity_5d".to_string(),
        ]);
        
        // Technical indicator features
        if self.config.include_technical_indicators {
            names.extend(vec![
                "sma_10".to_string(),
                "sma_20".to_string(),
                "ema_12".to_string(),
                "ema_26".to_string(),
                "macd".to_string(),
                "macd_signal".to_string(),
                "macd_histogram".to_string(),
                "rsi".to_string(),
                "bb_upper".to_string(),
                "bb_middle".to_string(),
                "bb_lower".to_string(),
                "stoch_k".to_string(),
                "stoch_d".to_string(),
            ]);
        }
        
        // Volume features
        if self.config.include_volume_features {
            names.extend(vec![
                "volume_sma_10".to_string(),
                "volume_sma_20".to_string(),
                "volume_roc".to_string(),
                "obv".to_string(),
                "vpt".to_string(),
            ]);
        }
        
        // External features (VIX, interest rates)
        // Property 8: External Feature Integration
        if self.config.include_external_features && self.has_external_features() {
            names.extend(vec![
                "vix_normalized".to_string(),
                "interest_rate_normalized".to_string(),
            ]);
        }
        
        names
    }
}

/// Complete feature set for training
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureSet {
    pub sequences: Vec<Vec<f64>>,
    pub targets: Vec<Vec<f64>>,
    pub metadata: Vec<SequenceMetadata>,
    pub feature_names: Vec<String>,
    pub config: FeatureConfig,
}

/// Metadata for each sequence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequenceMetadata {
    pub symbol: String,
    pub sector: String,
    pub sequence_start: DateTime<Utc>,
    pub sequence_end: DateTime<Utc>,
    pub target_timestamp: DateTime<Utc>,
    pub sequence_length: usize,
    pub num_features: usize,
}

/// Technical indicators calculator
#[derive(Debug)]
pub struct TechnicalIndicators;

impl TechnicalIndicators {
    pub fn new() -> Self {
        Self
    }
    
    /// Simple Moving Average
    pub fn sma(&self, data: &[f64], period: usize) -> MLResult<Vec<f64>> {
        if period == 0 {
            return Err(MLError::feature_error("Period must be greater than 0 for SMA calculation"));
        }
        if data.len() < period {
            return Err(MLError::feature_error("Insufficient data for SMA calculation"));
        }
        
        let mut sma = vec![0.0; period - 1]; // Pad with zeros for initial values
        
        for i in (period - 1)..data.len() {
            // Use saturating_sub to avoid overflow
            let start = i.saturating_sub(period - 1);
            let sum: f64 = data[start..=i].iter().sum();
            sma.push(sum / period as f64);
        }
        
        Ok(sma)
    }
    
    /// Exponential Moving Average
    pub fn ema(&self, data: &[f64], period: usize) -> MLResult<Vec<f64>> {
        if data.is_empty() {
            return Err(MLError::feature_error("Empty data for EMA calculation"));
        }
        
        let alpha = 2.0 / (period as f64 + 1.0);
        let mut ema = Vec::with_capacity(data.len());
        ema.push(data[0]); // First value is the same as input
        
        for &price in &data[1..] {
            let prev_ema = ema.last().unwrap();
            ema.push(alpha * price + (1.0 - alpha) * prev_ema);
        }
        
        Ok(ema)
    }
    
    /// MACD (Moving Average Convergence Divergence)
    pub fn macd(&self, data: &[f64], fast: usize, slow: usize, signal: usize) -> MLResult<MACDResult> {
        let ema_fast = self.ema(data, fast)?;
        let ema_slow = self.ema(data, slow)?;
        
        let macd: Vec<f64> = ema_fast.iter().zip(ema_slow.iter())
            .map(|(fast, slow)| fast - slow)
            .collect();
        
        let signal_line = self.ema(&macd, signal)?;
        let histogram: Vec<f64> = macd.iter().zip(signal_line.iter())
            .map(|(macd, signal)| macd - signal)
            .collect();
        
        Ok(MACDResult {
            macd,
            signal: signal_line,
            histogram,
        })
    }
    
    /// RSI (Relative Strength Index)
    pub fn rsi(&self, data: &[f64], period: usize) -> MLResult<Vec<f64>> {
        if period == 0 {
            return Err(MLError::feature_error("Period must be greater than 0 for RSI calculation"));
        }
        if data.len() < period + 1 {
            return Err(MLError::feature_error("Insufficient data for RSI calculation"));
        }
        
        let mut gains = Vec::new();
        let mut losses = Vec::new();
        
        // Calculate price changes
        for i in 1..data.len() {
            let change = data[i] - data[i - 1];
            gains.push(if change > 0.0 { change } else { 0.0 });
            losses.push(if change < 0.0 { -change } else { 0.0 });
        }
        
        let avg_gain = self.sma(&gains, period)?;
        let avg_loss = self.sma(&losses, period)?;
        
        let mut rsi = vec![50.0]; // Start with neutral RSI
        
        for i in 0..avg_gain.len() {
            if avg_loss[i] == 0.0 {
                rsi.push(100.0);
            } else {
                let rs = avg_gain[i] / avg_loss[i];
                rsi.push(100.0 - (100.0 / (1.0 + rs)));
            }
        }
        
        Ok(rsi)
    }
    
    /// Bollinger Bands
    pub fn bollinger_bands(&self, data: &[f64], period: usize, std_dev: f64) -> MLResult<BollingerBandsResult> {
        if period == 0 {
            return Err(MLError::feature_error("Period must be greater than 0 for Bollinger Bands calculation"));
        }
        let sma = self.sma(data, period)?;
        let mut upper = Vec::new();
        let mut lower = Vec::new();
        
        for i in (period - 1)..data.len() {
            let start = i.saturating_sub(period - 1);
            let window = &data[start..=i];
            let mean = window.iter().sum::<f64>() / period as f64;
            let variance = window.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / period as f64;
            let std = variance.sqrt();
            
            upper.push(sma[i] + std_dev * std);
            lower.push(sma[i] - std_dev * std);
        }
        
        // Pad with initial values
        let mut padded_upper = vec![data[0]; period - 1];
        let mut padded_lower = vec![data[0]; period - 1];
        padded_upper.extend(upper);
        padded_lower.extend(lower);
        
        Ok(BollingerBandsResult {
            upper: padded_upper,
            middle: sma,
            lower: padded_lower,
        })
    }
    
    /// Stochastic Oscillator
    pub fn stochastic(&self, highs: &[f64], lows: &[f64], closes: &[f64], k_period: usize, d_period: usize) -> MLResult<StochasticResult> {
        if k_period == 0 || d_period == 0 {
            return Err(MLError::feature_error("Periods must be greater than 0 for Stochastic calculation"));
        }
        if highs.len() != lows.len() || lows.len() != closes.len() {
            return Err(MLError::feature_error("High, low, and close arrays must have same length"));
        }
        
        if highs.len() < k_period {
            return Err(MLError::feature_error("Insufficient data for Stochastic calculation"));
        }
        
        let mut k_values = vec![50.0; k_period - 1]; // Pad with neutral values
        
        for i in (k_period - 1)..highs.len() {
            let start = i.saturating_sub(k_period - 1);
            let window_high = highs[start..=i].iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            let window_low = lows[start..=i].iter().fold(f64::INFINITY, |a, &b| a.min(b));
            
            if window_high == window_low {
                k_values.push(50.0);
            } else {
                let k = 100.0 * (closes[i] - window_low) / (window_high - window_low);
                k_values.push(k);
            }
        }
        
        let d_values = self.sma(&k_values, d_period)?;
        
        Ok(StochasticResult {
            k: k_values,
            d: d_values,
        })
    }
    
    /// Rate of Change
    pub fn rate_of_change(&self, data: &[f64], period: usize) -> MLResult<Vec<f64>> {
        if data.len() < period + 1 {
            return Err(MLError::feature_error("Insufficient data for ROC calculation"));
        }
        
        let mut roc = vec![0.0; period]; // Pad with zeros
        
        for i in period..data.len() {
            if data[i - period] == 0.0 {
                roc.push(0.0);
            } else {
                roc.push((data[i] - data[i - period]) / data[i - period] * 100.0);
            }
        }
        
        Ok(roc)
    }
    
    /// On-Balance Volume
    pub fn obv(&self, prices: &[f64], volumes: &[f64]) -> MLResult<Vec<f64>> {
        if prices.len() != volumes.len() {
            return Err(MLError::feature_error("Prices and volumes must have same length"));
        }
        
        if prices.is_empty() {
            return Ok(Vec::new());
        }
        
        let mut obv = vec![volumes[0]];
        
        for i in 1..prices.len() {
            let prev_obv = obv[i - 1];
            if prices[i] > prices[i - 1] {
                obv.push(prev_obv + volumes[i]);
            } else if prices[i] < prices[i - 1] {
                obv.push(prev_obv - volumes[i]);
            } else {
                obv.push(prev_obv);
            }
        }
        
        Ok(obv)
    }
    
    /// Volume-Price Trend
    pub fn vpt(&self, prices: &[f64], volumes: &[f64]) -> MLResult<Vec<f64>> {
        if prices.len() != volumes.len() {
            return Err(MLError::feature_error("Prices and volumes must have same length"));
        }
        
        if prices.is_empty() {
            return Ok(Vec::new());
        }
        
        let mut vpt = vec![0.0];
        
        for i in 1..prices.len() {
            let price_change_pct = if prices[i - 1] == 0.0 {
                0.0
            } else {
                (prices[i] - prices[i - 1]) / prices[i - 1]
            };
            
            vpt.push(vpt[i - 1] + volumes[i] * price_change_pct);
        }
        
        Ok(vpt)
    }
}

impl Default for TechnicalIndicators {
    fn default() -> Self {
        Self::new()
    }
}

/// MACD calculation result
#[derive(Debug, Clone)]
pub struct MACDResult {
    pub macd: Vec<f64>,
    pub signal: Vec<f64>,
    pub histogram: Vec<f64>,
}

/// Bollinger Bands calculation result
#[derive(Debug, Clone)]
pub struct BollingerBandsResult {
    pub upper: Vec<f64>,
    pub middle: Vec<f64>,
    pub lower: Vec<f64>,
}

/// Stochastic oscillator calculation result
#[derive(Debug, Clone)]
pub struct StochasticResult {
    pub k: Vec<f64>,
    pub d: Vec<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    
    #[test]
    fn test_feature_config_default() {
        let config = FeatureConfig::default();
        assert_eq!(config.sequence_length, 20);
        assert_eq!(config.prediction_horizon, 5);
        assert!(config.include_technical_indicators);
    }
    
    #[test]
    fn test_technical_indicators_sma() {
        let indicators = TechnicalIndicators::new();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let sma = indicators.sma(&data, 3).unwrap();
        
        assert_eq!(sma.len(), 5);
        assert_eq!(sma[2], 2.0); // (1+2+3)/3
        assert_eq!(sma[3], 3.0); // (2+3+4)/3
        assert_eq!(sma[4], 4.0); // (3+4+5)/3
    }
    
    #[test]
    fn test_quadrant_encoding() {
        let config = FeatureConfig::default();
        let engineer = FeatureEngineer::new(config);
        
        let quadrants = vec![Quadrant::Leading, Quadrant::Weakening, Quadrant::Lagging, Quadrant::Improving];
        let encoded = engineer.encode_quadrants(&quadrants);
        
        assert_eq!(encoded.len(), 4); // 4 one-hot vectors
        assert_eq!(encoded[0], vec![1.0, 0.0, 0.0, 0.0]); // Leading
        assert_eq!(encoded[1], vec![0.0, 1.0, 0.0, 0.0]); // Weakening
        assert_eq!(encoded[2], vec![0.0, 0.0, 1.0, 0.0]); // Lagging
        assert_eq!(encoded[3], vec![0.0, 0.0, 0.0, 1.0]); // Improving
    }
}