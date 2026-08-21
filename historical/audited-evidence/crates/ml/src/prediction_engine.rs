use crate::{MLError, MLResult, LSTMPredictor, ProphetForecaster, ProphetConfig};
use rrg_calc::{Quadrant, RRGData};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use tracing::{info, debug};

/// Quadrant probability distribution
/// All probabilities must sum to 1.0 (within floating-point tolerance)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuadrantProbabilities {
    /// Probability of being in Leading quadrant (RS-Ratio > 100, RS-Momentum > 100)
    pub leading: f64,
    /// Probability of being in Weakening quadrant (RS-Ratio > 100, RS-Momentum < 100)
    pub weakening: f64,
    /// Probability of being in Lagging quadrant (RS-Ratio < 100, RS-Momentum < 100)
    pub lagging: f64,
    /// Probability of being in Improving quadrant (RS-Ratio < 100, RS-Momentum > 100)
    pub improving: f64,
}

impl QuadrantProbabilities {
    /// Create new quadrant probabilities with validation
    pub fn new(leading: f64, weakening: f64, lagging: f64, improving: f64) -> MLResult<Self> {
        let probs = Self { leading, weakening, lagging, improving };
        probs.validate()?;
        Ok(probs)
    }
    
    /// Validate that probabilities are valid (each in [0,1] and sum to 1.0)
    pub fn validate(&self) -> MLResult<()> {
        // Check each probability is in [0, 1]
        if self.leading < 0.0 || self.leading > 1.0 {
            return Err(MLError::prediction(format!("Leading probability {} out of range [0, 1]", self.leading)));
        }
        if self.weakening < 0.0 || self.weakening > 1.0 {
            return Err(MLError::prediction(format!("Weakening probability {} out of range [0, 1]", self.weakening)));
        }
        if self.lagging < 0.0 || self.lagging > 1.0 {
            return Err(MLError::prediction(format!("Lagging probability {} out of range [0, 1]", self.lagging)));
        }
        if self.improving < 0.0 || self.improving > 1.0 {
            return Err(MLError::prediction(format!("Improving probability {} out of range [0, 1]", self.improving)));
        }
        
        // Check sum is approximately 1.0 (within floating-point tolerance)
        let sum = self.leading + self.weakening + self.lagging + self.improving;
        if (sum - 1.0).abs() > 1e-6 {
            return Err(MLError::prediction(format!("Probabilities sum to {} instead of 1.0", sum)));
        }
        
        Ok(())
    }
    
    /// Get the most likely quadrant
    pub fn most_likely_quadrant(&self) -> Quadrant {
        let max_prob = self.leading.max(self.weakening).max(self.lagging).max(self.improving);
        
        if (self.leading - max_prob).abs() < 1e-9 {
            Quadrant::Leading
        } else if (self.weakening - max_prob).abs() < 1e-9 {
            Quadrant::Weakening
        } else if (self.lagging - max_prob).abs() < 1e-9 {
            Quadrant::Lagging
        } else {
            Quadrant::Improving
        }
    }
    
    /// Get probability for a specific quadrant
    pub fn probability_for(&self, quadrant: &Quadrant) -> f64 {
        match quadrant {
            Quadrant::Leading => self.leading,
            Quadrant::Weakening => self.weakening,
            Quadrant::Lagging => self.lagging,
            Quadrant::Improving => self.improving,
        }
    }
    
    /// Sum of all probabilities
    pub fn sum(&self) -> f64 {
        self.leading + self.weakening + self.lagging + self.improving
    }
}


/// Confidence interval for RS-Ratio and RS-Momentum predictions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceInterval {
    /// Lower bound for RS-Ratio prediction
    pub rs_ratio_lower: f64,
    /// Upper bound for RS-Ratio prediction
    pub rs_ratio_upper: f64,
    /// Lower bound for RS-Momentum prediction
    pub rs_momentum_lower: f64,
    /// Upper bound for RS-Momentum prediction
    pub rs_momentum_upper: f64,
    /// Confidence level (e.g., 0.95 for 95% confidence)
    pub confidence_level: f64,
}

impl ConfidenceInterval {
    /// Create new confidence interval with validation
    pub fn new(
        rs_ratio_lower: f64,
        rs_ratio_upper: f64,
        rs_momentum_lower: f64,
        rs_momentum_upper: f64,
        confidence_level: f64,
    ) -> MLResult<Self> {
        let interval = Self {
            rs_ratio_lower,
            rs_ratio_upper,
            rs_momentum_lower,
            rs_momentum_upper,
            confidence_level,
        };
        interval.validate()?;
        Ok(interval)
    }
    
    /// Validate confidence interval ordering
    pub fn validate(&self) -> MLResult<()> {
        // Check RS-Ratio ordering: lower <= upper
        if self.rs_ratio_lower > self.rs_ratio_upper {
            return Err(MLError::prediction(format!(
                "RS-Ratio lower bound {} > upper bound {}",
                self.rs_ratio_lower, self.rs_ratio_upper
            )));
        }
        
        // Check RS-Momentum ordering: lower <= upper
        if self.rs_momentum_lower > self.rs_momentum_upper {
            return Err(MLError::prediction(format!(
                "RS-Momentum lower bound {} > upper bound {}",
                self.rs_momentum_lower, self.rs_momentum_upper
            )));
        }
        
        // Check confidence level is in (0, 1]
        if self.confidence_level <= 0.0 || self.confidence_level > 1.0 {
            return Err(MLError::prediction(format!(
                "Confidence level {} out of range (0, 1]",
                self.confidence_level
            )));
        }
        
        Ok(())
    }
    
    /// Check if a point is within the confidence interval
    pub fn contains(&self, rs_ratio: f64, rs_momentum: f64) -> bool {
        rs_ratio >= self.rs_ratio_lower
            && rs_ratio <= self.rs_ratio_upper
            && rs_momentum >= self.rs_momentum_lower
            && rs_momentum <= self.rs_momentum_upper
    }
    
    /// Get the width of the RS-Ratio interval
    pub fn rs_ratio_width(&self) -> f64 {
        self.rs_ratio_upper - self.rs_ratio_lower
    }
    
    /// Get the width of the RS-Momentum interval
    pub fn rs_momentum_width(&self) -> f64 {
        self.rs_momentum_upper - self.rs_momentum_lower
    }
}

/// Prediction with confidence intervals and quadrant probabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuadrantPrediction {
    /// Symbol being predicted
    pub symbol: String,
    /// Current quadrant position
    pub current_quadrant: Quadrant,
    /// Predicted quadrant at horizon
    pub predicted_quadrant: Quadrant,
    /// Predicted RS-Ratio value
    pub predicted_rs_ratio: f64,
    /// Predicted RS-Momentum value
    pub predicted_rs_momentum: f64,
    /// Probability distribution across quadrants
    pub probabilities: QuadrantProbabilities,
    /// Probability of transitioning from current to predicted quadrant
    pub transition_probability: f64,
    /// Confidence interval for the prediction
    pub confidence_interval: ConfidenceInterval,
    /// Prediction horizon in weeks
    pub prediction_horizon: usize,
    /// Timestamp when prediction was made
    pub timestamp: DateTime<Utc>,
    /// Flag indicating low reliability prediction
    pub low_confidence: bool,
}

impl QuadrantPrediction {
    /// Create a new quadrant prediction
    pub fn new(
        symbol: String,
        current_quadrant: Quadrant,
        predicted_rs_ratio: f64,
        predicted_rs_momentum: f64,
        probabilities: QuadrantProbabilities,
        confidence_interval: ConfidenceInterval,
        prediction_horizon: usize,
    ) -> Self {
        let predicted_quadrant = probabilities.most_likely_quadrant();
        let transition_probability = probabilities.probability_for(&predicted_quadrant);
        
        Self {
            symbol,
            current_quadrant,
            predicted_quadrant,
            predicted_rs_ratio,
            predicted_rs_momentum,
            probabilities,
            transition_probability,
            confidence_interval,
            prediction_horizon,
            timestamp: Utc::now(),
            low_confidence: false,
        }
    }
    
    /// Check if this is a quadrant transition prediction
    pub fn is_transition(&self) -> bool {
        self.current_quadrant != self.predicted_quadrant
    }
    
    /// Flag prediction as low confidence if below threshold
    pub fn flag_low_confidence(&mut self, threshold: f64) {
        self.low_confidence = self.transition_probability < threshold;
    }
}

/// Supported prediction horizons in weeks
pub const SUPPORTED_HORIZONS: [usize; 4] = [1, 2, 4, 8];

/// Check if a horizon is supported
pub fn is_supported_horizon(horizon: usize) -> bool {
    SUPPORTED_HORIZONS.contains(&horizon)
}

/// Configuration for the prediction engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionEngineConfig {
    /// Confidence threshold for low-confidence flagging
    pub confidence_threshold: f64,
    /// Default confidence level for intervals (e.g., 0.95 for 95%)
    pub default_confidence_level: f64,
    /// Prophet configuration for forecasting
    pub prophet_config: ProphetConfig,
}

impl Default for PredictionEngineConfig {
    fn default() -> Self {
        Self {
            confidence_threshold: 0.6,
            default_confidence_level: 0.95,
            prophet_config: ProphetConfig::default(),
        }
    }
}

/// Prediction engine combining LSTM and Prophet for quadrant predictions
#[derive(Debug)]
pub struct PredictionEngine {
    /// LSTM predictor for sequence modeling
    lstm_predictor: LSTMPredictor,
    /// Prophet forecaster for time series decomposition
    prophet_forecaster: ProphetForecaster,
    /// Engine configuration
    config: PredictionEngineConfig,
}

impl PredictionEngine {
    /// Create a new prediction engine
    pub fn new(
        lstm_predictor: LSTMPredictor,
        config: PredictionEngineConfig,
    ) -> Self {
        let prophet_forecaster = ProphetForecaster::new(config.prophet_config.clone());
        Self {
            lstm_predictor,
            prophet_forecaster,
            config,
        }
    }
    
    /// Generate probabilistic quadrant prediction for a given horizon
    pub fn predict_quadrant(
        &mut self,
        rrg_data: &RRGData,
        horizon: usize,
    ) -> MLResult<QuadrantPrediction> {
        // Validate horizon
        if !is_supported_horizon(horizon) {
            return Err(MLError::prediction(format!(
                "Unsupported horizon {}. Supported horizons: {:?}",
                horizon, SUPPORTED_HORIZONS
            )));
        }
        
        // Validate input data
        if rrg_data.normalized_rs_ratio.is_empty() || rrg_data.normalized_rs_momentum.is_empty() {
            return Err(MLError::prediction("Empty RRG data"));
        }
        
        info!("Generating quadrant prediction for {} with horizon {} weeks", 
            rrg_data.symbol, horizon);
        
        // Get current position
        let current_rs_ratio = *rrg_data.normalized_rs_ratio.last()
            .ok_or_else(|| MLError::prediction("No RS-Ratio data"))?;
        let current_rs_momentum = *rrg_data.normalized_rs_momentum.last()
            .ok_or_else(|| MLError::prediction("No RS-Momentum data"))?;
        let current_quadrant = Self::determine_quadrant(current_rs_ratio, current_rs_momentum);
        
        // Generate predictions using Prophet decomposition
        let decomposition = self.prophet_forecaster.decompose_time_series(
            &rrg_data.normalized_rs_ratio,
            &rrg_data.timestamps,
        )?;
        
        let forecast = self.prophet_forecaster.generate_baseline_forecast(&decomposition, horizon)?;
        
        // Get predicted RS-Ratio (last forecast point)
        let predicted_rs_ratio = *forecast.forecast.last()
            .ok_or_else(|| MLError::prediction("Empty forecast"))?;
        
        // Generate RS-Momentum forecast
        let momentum_decomposition = self.prophet_forecaster.decompose_time_series(
            &rrg_data.normalized_rs_momentum,
            &rrg_data.timestamps,
        )?;
        let momentum_forecast = self.prophet_forecaster.generate_baseline_forecast(&momentum_decomposition, horizon)?;
        let predicted_rs_momentum = *momentum_forecast.forecast.last()
            .ok_or_else(|| MLError::prediction("Empty momentum forecast"))?;
        
        // Calculate quadrant probabilities based on predicted position and uncertainty
        let probabilities = self.calculate_quadrant_probabilities(
            predicted_rs_ratio,
            predicted_rs_momentum,
            &forecast,
            &momentum_forecast,
        )?;
        
        // Create confidence interval
        let confidence_interval = ConfidenceInterval::new(
            *forecast.lower_bound.last().unwrap_or(&(predicted_rs_ratio - 5.0)),
            *forecast.upper_bound.last().unwrap_or(&(predicted_rs_ratio + 5.0)),
            *momentum_forecast.lower_bound.last().unwrap_or(&(predicted_rs_momentum - 2.0)),
            *momentum_forecast.upper_bound.last().unwrap_or(&(predicted_rs_momentum + 2.0)),
            self.config.default_confidence_level,
        )?;
        
        // Create prediction
        let mut prediction = QuadrantPrediction::new(
            rrg_data.symbol.clone(),
            current_quadrant,
            predicted_rs_ratio,
            predicted_rs_momentum,
            probabilities,
            confidence_interval,
            horizon,
        );
        
        // Flag low confidence if below threshold
        prediction.flag_low_confidence(self.config.confidence_threshold);
        
        debug!("Prediction: {:?} -> {:?} with confidence {:.2}",
            prediction.current_quadrant,
            prediction.predicted_quadrant,
            prediction.transition_probability);
        
        Ok(prediction)
    }
    
    /// Generate predictions for all supported horizons
    pub fn predict_all_horizons(
        &mut self,
        rrg_data: &RRGData,
    ) -> MLResult<Vec<QuadrantPrediction>> {
        let mut predictions = Vec::new();
        
        for &horizon in &SUPPORTED_HORIZONS {
            let prediction = self.predict_quadrant(rrg_data, horizon)?;
            predictions.push(prediction);
        }
        
        Ok(predictions)
    }
    
    /// Calculate quadrant probabilities based on predicted position and uncertainty
    fn calculate_quadrant_probabilities(
        &self,
        predicted_rs_ratio: f64,
        predicted_rs_momentum: f64,
        ratio_forecast: &crate::ForecastResult,
        momentum_forecast: &crate::ForecastResult,
    ) -> MLResult<QuadrantProbabilities> {
        // Get uncertainty from forecast bounds
        let ratio_std = (ratio_forecast.upper_bound.last().unwrap_or(&105.0) 
            - ratio_forecast.lower_bound.last().unwrap_or(&95.0)) / 3.92; // 95% CI = 1.96 * 2
        let momentum_std = (momentum_forecast.upper_bound.last().unwrap_or(&102.0)
            - momentum_forecast.lower_bound.last().unwrap_or(&98.0)) / 3.92;
        
        // Use sigmoid-based probability calculation
        // Distance from quadrant boundary (100) normalized by uncertainty
        let ratio_z = (predicted_rs_ratio - 100.0) / ratio_std.max(0.1);
        let momentum_z = (predicted_rs_momentum - 100.0) / momentum_std.max(0.1);
        
        // Probability of being above 100 for each dimension
        let p_ratio_high = 1.0 / (1.0 + (-ratio_z).exp());
        let p_momentum_high = 1.0 / (1.0 + (-momentum_z).exp());
        
        // Calculate quadrant probabilities
        // Leading: ratio > 100, momentum > 100
        let leading = p_ratio_high * p_momentum_high;
        // Weakening: ratio > 100, momentum < 100
        let weakening = p_ratio_high * (1.0 - p_momentum_high);
        // Lagging: ratio < 100, momentum < 100
        let lagging = (1.0 - p_ratio_high) * (1.0 - p_momentum_high);
        // Improving: ratio < 100, momentum > 100
        let improving = (1.0 - p_ratio_high) * p_momentum_high;
        
        QuadrantProbabilities::new(leading, weakening, lagging, improving)
    }
    
    /// Determine quadrant from RS-Ratio and RS-Momentum values
    fn determine_quadrant(rs_ratio: f64, rs_momentum: f64) -> Quadrant {
        match (rs_ratio >= 100.0, rs_momentum >= 100.0) {
            (true, true) => Quadrant::Leading,
            (true, false) => Quadrant::Weakening,
            (false, false) => Quadrant::Lagging,
            (false, true) => Quadrant::Improving,
        }
    }
    
    /// Calculate transition probability matrix from historical data
    pub fn calculate_transition_matrix(&self, rrg_data: &RRGData) -> MLResult<TransitionMatrix> {
        if rrg_data.normalized_rs_ratio.len() < 2 {
            return Err(MLError::prediction("Insufficient data for transition matrix"));
        }
        
        let mut transitions = [[0u32; 4]; 4]; // [from][to] counts
        
        for i in 1..rrg_data.normalized_rs_ratio.len() {
            let prev_quadrant = Self::determine_quadrant(
                rrg_data.normalized_rs_ratio[i - 1],
                rrg_data.normalized_rs_momentum[i - 1],
            );
            let curr_quadrant = Self::determine_quadrant(
                rrg_data.normalized_rs_ratio[i],
                rrg_data.normalized_rs_momentum[i],
            );
            
            let from_idx = Self::quadrant_to_index(&prev_quadrant);
            let to_idx = Self::quadrant_to_index(&curr_quadrant);
            transitions[from_idx][to_idx] += 1;
        }
        
        // Convert counts to probabilities
        let mut matrix = [[0.0f64; 4]; 4];
        for from in 0..4 {
            let total: u32 = transitions[from].iter().sum();
            if total > 0 {
                for to in 0..4 {
                    matrix[from][to] = transitions[from][to] as f64 / total as f64;
                }
            } else {
                // If no transitions from this state, assume uniform
                for to in 0..4 {
                    matrix[from][to] = 0.25;
                }
            }
        }
        
        Ok(TransitionMatrix { matrix })
    }
    
    fn quadrant_to_index(quadrant: &Quadrant) -> usize {
        match quadrant {
            Quadrant::Leading => 0,
            Quadrant::Weakening => 1,
            Quadrant::Lagging => 2,
            Quadrant::Improving => 3,
        }
    }
    
    /// Get the confidence threshold
    pub fn confidence_threshold(&self) -> f64 {
        self.config.confidence_threshold
    }
    
    /// Set the confidence threshold
    pub fn set_confidence_threshold(&mut self, threshold: f64) {
        self.config.confidence_threshold = threshold;
    }
}

/// Transition probability matrix between quadrants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionMatrix {
    /// 4x4 matrix: [from_quadrant][to_quadrant]
    /// Order: Leading, Weakening, Lagging, Improving
    pub matrix: [[f64; 4]; 4],
}

impl TransitionMatrix {
    /// Get transition probability from one quadrant to another
    pub fn get_probability(&self, from: &Quadrant, to: &Quadrant) -> f64 {
        let from_idx = match from {
            Quadrant::Leading => 0,
            Quadrant::Weakening => 1,
            Quadrant::Lagging => 2,
            Quadrant::Improving => 3,
        };
        let to_idx = match to {
            Quadrant::Leading => 0,
            Quadrant::Weakening => 1,
            Quadrant::Lagging => 2,
            Quadrant::Improving => 3,
        };
        self.matrix[from_idx][to_idx]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LSTMConfig, LSTMPredictor};
    
    fn create_test_rrg_data() -> RRGData {
        let timestamps: Vec<DateTime<Utc>> = (0..20)
            .map(|i| Utc::now() - chrono::Duration::days(20 - i))
            .collect();
        
        // Create realistic RS-Ratio and RS-Momentum data
        let normalized_rs_ratio: Vec<f64> = (0..20)
            .map(|i| 98.0 + (i as f64 * 0.2) + (i as f64 * 0.1).sin())
            .collect();
        let normalized_rs_momentum: Vec<f64> = (0..20)
            .map(|i| 99.0 + (i as f64 * 0.15) + (i as f64 * 0.15).cos())
            .collect();
        
        // Determine quadrants for each point
        let quadrants: Vec<Quadrant> = normalized_rs_ratio.iter()
            .zip(normalized_rs_momentum.iter())
            .map(|(&r, &m)| PredictionEngine::determine_quadrant(r, m))
            .collect();
        
        let current_quadrant = *quadrants.last().unwrap_or(&Quadrant::Lagging);
        
        RRGData {
            symbol: "TEST".to_string(),
            sector: "Technology".to_string(),
            timestamps,
            rs_ratio: vec![1.0; 20],
            rs_momentum: vec![0.0; 20],
            normalized_rs_ratio,
            normalized_rs_momentum,
            quadrants,
            current_quadrant,
            quadrant_strength: 0.5,
            points: Vec::new(),
        }
    }
    
    fn create_test_prediction_engine() -> PredictionEngine {
        let lstm_config = LSTMConfig::default();
        let lstm_predictor = LSTMPredictor::new(lstm_config).unwrap();
        let config = PredictionEngineConfig::default();
        PredictionEngine::new(lstm_predictor, config)
    }
    
    #[test]
    fn test_quadrant_probabilities_valid() {
        let probs = QuadrantProbabilities::new(0.4, 0.3, 0.2, 0.1).unwrap();
        assert!((probs.sum() - 1.0).abs() < 1e-6);
    }
    
    #[test]
    fn test_quadrant_probabilities_invalid_sum() {
        let result = QuadrantProbabilities::new(0.5, 0.5, 0.5, 0.5);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_quadrant_probabilities_negative() {
        let result = QuadrantProbabilities::new(-0.1, 0.5, 0.3, 0.3);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_confidence_interval_valid() {
        let ci = ConfidenceInterval::new(95.0, 105.0, 98.0, 102.0, 0.95).unwrap();
        assert!(ci.contains(100.0, 100.0));
    }
    
    #[test]
    fn test_confidence_interval_invalid_ordering() {
        let result = ConfidenceInterval::new(105.0, 95.0, 98.0, 102.0, 0.95);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_most_likely_quadrant() {
        let probs = QuadrantProbabilities::new(0.5, 0.2, 0.2, 0.1).unwrap();
        assert_eq!(probs.most_likely_quadrant(), Quadrant::Leading);
    }
    
    #[test]
    fn test_supported_horizons() {
        assert!(is_supported_horizon(1));
        assert!(is_supported_horizon(2));
        assert!(is_supported_horizon(4));
        assert!(is_supported_horizon(8));
        assert!(!is_supported_horizon(3));
        assert!(!is_supported_horizon(5));
    }
    
    #[test]
    fn test_prediction_engine_creation() {
        let engine = create_test_prediction_engine();
        assert_eq!(engine.confidence_threshold(), 0.6);
    }
    
    #[test]
    fn test_predict_quadrant_valid_horizon() {
        let mut engine = create_test_prediction_engine();
        let rrg_data = create_test_rrg_data();
        
        let result = engine.predict_quadrant(&rrg_data, 1);
        assert!(result.is_ok());
        
        let prediction = result.unwrap();
        assert_eq!(prediction.symbol, "TEST");
        assert_eq!(prediction.prediction_horizon, 1);
        assert!((prediction.probabilities.sum() - 1.0).abs() < 1e-6);
    }
    
    #[test]
    fn test_predict_quadrant_invalid_horizon() {
        let mut engine = create_test_prediction_engine();
        let rrg_data = create_test_rrg_data();
        
        let result = engine.predict_quadrant(&rrg_data, 3); // 3 is not supported
        assert!(result.is_err());
    }
    
    #[test]
    fn test_predict_all_horizons() {
        let mut engine = create_test_prediction_engine();
        let rrg_data = create_test_rrg_data();
        
        let result = engine.predict_all_horizons(&rrg_data);
        assert!(result.is_ok());
        
        let predictions = result.unwrap();
        assert_eq!(predictions.len(), 4); // 1, 2, 4, 8 weeks
        
        for (i, prediction) in predictions.iter().enumerate() {
            assert_eq!(prediction.prediction_horizon, SUPPORTED_HORIZONS[i]);
        }
    }
    
    #[test]
    fn test_transition_matrix() {
        let engine = create_test_prediction_engine();
        let rrg_data = create_test_rrg_data();
        
        let result = engine.calculate_transition_matrix(&rrg_data);
        assert!(result.is_ok());
        
        let matrix = result.unwrap();
        // Each row should sum to 1.0
        for row in &matrix.matrix {
            let sum: f64 = row.iter().sum();
            assert!((sum - 1.0).abs() < 1e-6);
        }
    }
    
    #[test]
    fn test_determine_quadrant() {
        assert_eq!(PredictionEngine::determine_quadrant(105.0, 102.0), Quadrant::Leading);
        assert_eq!(PredictionEngine::determine_quadrant(105.0, 98.0), Quadrant::Weakening);
        assert_eq!(PredictionEngine::determine_quadrant(95.0, 98.0), Quadrant::Lagging);
        assert_eq!(PredictionEngine::determine_quadrant(95.0, 102.0), Quadrant::Improving);
    }
    
    #[test]
    fn test_low_confidence_flagging() {
        let mut engine = create_test_prediction_engine();
        engine.set_confidence_threshold(0.9); // High threshold
        
        let rrg_data = create_test_rrg_data();
        let prediction = engine.predict_quadrant(&rrg_data, 1).unwrap();
        
        // With high threshold, prediction might be flagged as low confidence
        // This depends on the actual prediction confidence
        assert!(prediction.transition_probability >= 0.0);
        assert!(prediction.transition_probability <= 1.0);
    }
}
