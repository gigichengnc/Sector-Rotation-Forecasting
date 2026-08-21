use crate::{MLError, MLResult, FeatureConfig, LinearPredictor, LinearModelConfig, ModelMetrics};
use crate::feature_engineering::FeatureSet;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for walk-forward validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalkForwardConfig {
    /// Number of periods in training window
    pub training_window: usize,
    /// Number of periods in validation window
    pub validation_window: usize,
    /// Step size for rolling the window forward
    pub step_size: usize,
    /// Minimum number of folds required
    pub min_folds: usize,
    /// Gap between training and validation to prevent lookahead
    pub gap_periods: usize,
}

impl Default for WalkForwardConfig {
    fn default() -> Self {
        Self {
            training_window: 252, // ~1 year of trading days
            validation_window: 63, // ~3 months
            step_size: 21, // ~1 month
            min_folds: 3,
            gap_periods: 1, // 1 period gap to prevent lookahead
        }
    }
}

/// Results from a single walk-forward fold
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoldResult {
    pub fold_index: usize,
    pub train_start: usize,
    pub train_end: usize,
    pub val_start: usize,
    pub val_end: usize,
    pub train_metrics: ValidationMetrics,
    pub val_metrics: ValidationMetrics,
    pub model_metrics: ModelMetrics,
}

/// Validation metrics for model evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationMetrics {
    pub mae: f64,           // Mean Absolute Error
    pub rmse: f64,          // Root Mean Square Error
    pub mape: f64,          // Mean Absolute Percentage Error
    pub directional_accuracy: f64, // Percentage of correct direction predictions
    pub r_squared: f64,     // Coefficient of determination
    pub sample_count: usize,
}

impl Default for ValidationMetrics {
    fn default() -> Self {
        Self {
            mae: 0.0,
            rmse: 0.0,
            mape: 0.0,
            directional_accuracy: 0.0,
            r_squared: 0.0,
            sample_count: 0,
        }
    }
}

impl ValidationMetrics {
    /// Calculate metrics from predictions and actuals
    pub fn calculate(predictions: &[f64], actuals: &[f64]) -> MLResult<Self> {
        if predictions.len() != actuals.len() {
            return Err(MLError::prediction("Predictions and actuals length mismatch"));
        }
        
        if predictions.is_empty() {
            return Ok(Self::default());
        }
        
        let n = predictions.len() as f64;
        
        // MAE
        let mae = predictions.iter()
            .zip(actuals.iter())
            .map(|(p, a)| (p - a).abs())
            .sum::<f64>() / n;
        
        // RMSE
        let mse = predictions.iter()
            .zip(actuals.iter())
            .map(|(p, a)| (p - a).powi(2))
            .sum::<f64>() / n;
        let rmse = mse.sqrt();
        
        // MAPE (avoid division by zero)
        let mape = predictions.iter()
            .zip(actuals.iter())
            .filter(|(_, a)| a.abs() > 1e-10)
            .map(|(p, a)| ((p - a) / a).abs())
            .sum::<f64>() / n * 100.0;
        
        // Directional accuracy
        let mut correct_direction = 0;
        for i in 1..predictions.len() {
            let pred_direction = predictions[i] > predictions[i-1];
            let actual_direction = actuals[i] > actuals[i-1];
            if pred_direction == actual_direction {
                correct_direction += 1;
            }
        }
        let directional_accuracy = if predictions.len() > 1 {
            correct_direction as f64 / (predictions.len() - 1) as f64 * 100.0
        } else {
            0.0
        };
        
        // R-squared
        let actual_mean = actuals.iter().sum::<f64>() / n;
        let ss_tot = actuals.iter().map(|a| (a - actual_mean).powi(2)).sum::<f64>();
        let ss_res = predictions.iter()
            .zip(actuals.iter())
            .map(|(p, a)| (a - p).powi(2))
            .sum::<f64>();
        let r_squared = if ss_tot > 1e-10 { 1.0 - ss_res / ss_tot } else { 0.0 };
        
        Ok(Self {
            mae,
            rmse,
            mape,
            directional_accuracy,
            r_squared,
            sample_count: predictions.len(),
        })
    }
}

/// Training pipeline with walk-forward validation
#[derive(Debug)]
pub struct TrainingPipeline {
    pub config: WalkForwardConfig,
    pub feature_config: FeatureConfig,
    pub model_config: LinearModelConfig,
    pub fold_results: Vec<FoldResult>,
    pub best_model_fold: Option<usize>,
}

impl TrainingPipeline {
    pub fn new(config: WalkForwardConfig, feature_config: FeatureConfig, model_config: LinearModelConfig) -> Self {
        Self {
            config,
            feature_config,
            model_config,
            fold_results: Vec::new(),
            best_model_fold: None,
        }
    }
    
    /// Generate walk-forward fold indices
    pub fn generate_folds(&self, data_length: usize) -> MLResult<Vec<(usize, usize, usize, usize)>> {
        let min_required = self.config.training_window + self.config.gap_periods + self.config.validation_window;
        
        if data_length < min_required {
            return Err(MLError::prediction(format!(
                "Insufficient data: need {} points, have {}", min_required, data_length
            )));
        }
        
        let mut folds = Vec::new();
        let mut train_start = 0;
        
        while train_start + min_required <= data_length {
            let train_end = train_start + self.config.training_window;
            let val_start = train_end + self.config.gap_periods;
            let val_end = (val_start + self.config.validation_window).min(data_length);
            
            if val_end > val_start {
                folds.push((train_start, train_end, val_start, val_end));
            }
            
            train_start += self.config.step_size;
        }
        
        if folds.len() < self.config.min_folds {
            return Err(MLError::prediction(format!(
                "Insufficient folds: need {}, generated {}", self.config.min_folds, folds.len()
            )));
        }
        
        Ok(folds)
    }
    
    /// Verify no data leakage between train and validation sets
    pub fn verify_no_leakage(&self, folds: &[(usize, usize, usize, usize)]) -> MLResult<bool> {
        for (i, (train_start, train_end, val_start, val_end)) in folds.iter().enumerate() {
            // Training must end before validation starts (with gap)
            if *train_end + self.config.gap_periods > *val_start {
                return Err(MLError::prediction(format!(
                    "Data leakage detected in fold {}: train_end={}, val_start={}", 
                    i, train_end, val_start
                )));
            }
            
            // Validation must not overlap with training
            if *val_start < *train_end {
                return Err(MLError::prediction(format!(
                    "Overlap detected in fold {}: train_end={}, val_start={}", 
                    i, train_end, val_start
                )));
            }
        }
        
        Ok(true)
    }
    
    /// Run walk-forward validation
    pub async fn run_walk_forward_validation(
        &mut self,
        features: &[Vec<f64>],
        targets: &[f64],
    ) -> MLResult<WalkForwardResults> {
        let data_length = features.len();
        let folds = self.generate_folds(data_length)?;
        self.verify_no_leakage(&folds)?;
        
        self.fold_results.clear();
        let mut all_val_predictions = Vec::new();
        let mut all_val_actuals = Vec::new();
        
        for (fold_idx, (_train_start, train_end, val_start, _val_end)) in folds.iter().enumerate() {
            // Split data
            let train_features: Vec<Vec<f64>> = features[..*train_end].to_vec();
            let train_targets: Vec<f64> = targets[..*train_end].to_vec();
            let val_features: Vec<Vec<f64>> = features[*val_start..].to_vec();
            let val_targets: Vec<f64> = targets[*val_start..].to_vec();
            
            // Create feature set for training - convert targets to Vec<Vec<f64>>
            let targets_2d: Vec<Vec<f64>> = train_targets.iter().map(|&t| vec![t]).collect();
            let feature_set = FeatureSet {
                sequences: train_features.clone(),
                targets: targets_2d,
                metadata: Vec::new(),
                feature_names: Vec::new(),
                config: self.feature_config.clone(),
            };
            
            // Train model on this fold
            let mut predictor = LinearPredictor::new(self.model_config.clone())?;
            let model_metrics = predictor.train(&feature_set).await?;
            
            // Generate predictions
            let train_predictions = self.generate_predictions(&predictor, &train_features).await?;
            let val_predictions = self.generate_predictions(&predictor, &val_features).await?;
            
            // Calculate metrics
            let train_metrics = ValidationMetrics::calculate(&train_predictions, &train_targets)?;
            let val_metrics = ValidationMetrics::calculate(&val_predictions, &val_targets)?;
            
            // Store fold results
            let fold_result = FoldResult {
                fold_index: fold_idx,
                train_start: 0,
                train_end: *train_end,
                val_start: *val_start,
                val_end: features.len(),
                train_metrics,
                val_metrics: val_metrics.clone(),
                model_metrics,
            };
            
            self.fold_results.push(fold_result);
            
            // Collect validation predictions for overall metrics
            all_val_predictions.extend(val_predictions);
            all_val_actuals.extend(val_targets);
        }
        
        // Calculate overall metrics
        let overall_metrics = ValidationMetrics::calculate(&all_val_predictions, &all_val_actuals)?;
        
        // Find best fold
        self.best_model_fold = self.fold_results.iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                a.val_metrics.rmse.partial_cmp(&b.val_metrics.rmse).unwrap()
            })
            .map(|(idx, _)| idx);
        
        Ok(WalkForwardResults {
            fold_results: self.fold_results.clone(),
            overall_metrics,
            best_fold_index: self.best_model_fold,
            num_folds: folds.len(),
            total_train_samples: folds.iter().map(|(s, e, _, _)| e - s).sum(),
            total_val_samples: folds.iter().map(|(_, _, s, e)| e - s).sum(),
        })
    }
    
    async fn generate_predictions(&self, predictor: &LinearPredictor, features: &[Vec<f64>]) -> MLResult<Vec<f64>> {
        let predictions = predictor.predict(features, 1).await?;
        // Flatten the predictions - take first value from each prediction
        Ok(predictions.into_iter().filter_map(|p| p.first().copied()).collect())
    }
    
    /// Get metrics by market regime
    pub fn get_metrics_by_regime(&self) -> HashMap<String, ValidationMetrics> {
        // Placeholder - would need regime labels in actual implementation
        HashMap::new()
    }
}

/// Results from walk-forward validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalkForwardResults {
    pub fold_results: Vec<FoldResult>,
    pub overall_metrics: ValidationMetrics,
    pub best_fold_index: Option<usize>,
    pub num_folds: usize,
    pub total_train_samples: usize,
    pub total_val_samples: usize,
}

impl WalkForwardResults {
    /// Get average validation metrics across all folds
    pub fn get_average_val_metrics(&self) -> ValidationMetrics {
        if self.fold_results.is_empty() {
            return ValidationMetrics::default();
        }
        
        let n = self.fold_results.len() as f64;
        
        ValidationMetrics {
            mae: self.fold_results.iter().map(|f| f.val_metrics.mae).sum::<f64>() / n,
            rmse: self.fold_results.iter().map(|f| f.val_metrics.rmse).sum::<f64>() / n,
            mape: self.fold_results.iter().map(|f| f.val_metrics.mape).sum::<f64>() / n,
            directional_accuracy: self.fold_results.iter().map(|f| f.val_metrics.directional_accuracy).sum::<f64>() / n,
            r_squared: self.fold_results.iter().map(|f| f.val_metrics.r_squared).sum::<f64>() / n,
            sample_count: self.fold_results.iter().map(|f| f.val_metrics.sample_count).sum(),
        }
    }
    
    /// Check if model is overfitting (train much better than val)
    pub fn is_overfitting(&self, threshold: f64) -> bool {
        for fold in &self.fold_results {
            let train_rmse = fold.train_metrics.rmse;
            let val_rmse = fold.val_metrics.rmse;
            
            if train_rmse > 0.0 && (val_rmse - train_rmse) / train_rmse > threshold {
                return true;
            }
        }
        false
    }
}

/// Validation engine for comprehensive model evaluation
#[derive(Debug)]
pub struct ValidationEngine {
    pub metrics_history: Vec<ValidationMetrics>,
    pub regime_metrics: HashMap<String, Vec<ValidationMetrics>>,
}

impl ValidationEngine {
    pub fn new() -> Self {
        Self {
            metrics_history: Vec::new(),
            regime_metrics: HashMap::new(),
        }
    }
    
    /// Record validation metrics
    pub fn record_metrics(&mut self, metrics: ValidationMetrics, regime: Option<&str>) {
        self.metrics_history.push(metrics.clone());
        
        if let Some(r) = regime {
            self.regime_metrics.entry(r.to_string())
                .or_default()
                .push(metrics);
        }
    }
    
    /// Get metrics for a specific regime
    pub fn get_regime_metrics(&self, regime: &str) -> Option<&Vec<ValidationMetrics>> {
        self.regime_metrics.get(regime)
    }
    
    /// Calculate average metrics across all history
    pub fn get_average_metrics(&self) -> ValidationMetrics {
        if self.metrics_history.is_empty() {
            return ValidationMetrics::default();
        }
        
        let n = self.metrics_history.len() as f64;
        
        ValidationMetrics {
            mae: self.metrics_history.iter().map(|m| m.mae).sum::<f64>() / n,
            rmse: self.metrics_history.iter().map(|m| m.rmse).sum::<f64>() / n,
            mape: self.metrics_history.iter().map(|m| m.mape).sum::<f64>() / n,
            directional_accuracy: self.metrics_history.iter().map(|m| m.directional_accuracy).sum::<f64>() / n,
            r_squared: self.metrics_history.iter().map(|m| m.r_squared).sum::<f64>() / n,
            sample_count: self.metrics_history.iter().map(|m| m.sample_count).sum(),
        }
    }
    
    /// Check if performance is degrading
    pub fn is_performance_degrading(&self, window: usize, threshold: f64) -> bool {
        if self.metrics_history.len() < window * 2 {
            return false;
        }
        
        let recent_start = self.metrics_history.len() - window;
        let older_start = recent_start - window;
        
        let recent_rmse: f64 = self.metrics_history[recent_start..].iter()
            .map(|m| m.rmse)
            .sum::<f64>() / window as f64;
        
        let older_rmse: f64 = self.metrics_history[older_start..recent_start].iter()
            .map(|m| m.rmse)
            .sum::<f64>() / window as f64;
        
        if older_rmse > 0.0 {
            (recent_rmse - older_rmse) / older_rmse > threshold
        } else {
            false
        }
    }
}

impl Default for ValidationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validation_metrics_calculation() {
        let predictions = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let actuals = vec![1.1, 2.1, 2.9, 4.2, 4.8];
        
        let metrics = ValidationMetrics::calculate(&predictions, &actuals).unwrap();
        
        assert!(metrics.mae > 0.0);
        assert!(metrics.rmse > 0.0);
        assert!(metrics.sample_count == 5);
    }
    
    #[test]
    fn test_walk_forward_fold_generation() {
        let config = WalkForwardConfig {
            training_window: 100,
            validation_window: 20,
            step_size: 20,
            min_folds: 2,
            gap_periods: 1,
        };
        
        let pipeline = TrainingPipeline::new(
            config,
            FeatureConfig::default(),
            LinearModelConfig::default(),
        );
        
        let folds = pipeline.generate_folds(200).unwrap();
        assert!(folds.len() >= 2);
        
        // Verify no overlap
        for (train_start, train_end, val_start, val_end) in &folds {
            assert!(train_end < val_start);
            assert!(val_start < val_end);
        }
    }
}
