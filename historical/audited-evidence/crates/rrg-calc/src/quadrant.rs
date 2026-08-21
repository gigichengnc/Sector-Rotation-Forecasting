use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::fmt;

/// RRG Quadrants based on RS-Ratio and RS-Momentum values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Quadrant {
    /// Leading: High RS-Ratio (>100), High RS-Momentum (>100)
    /// Sectors in this quadrant are outperforming and accelerating
    Leading,
    
    /// Weakening: High RS-Ratio (>100), Low RS-Momentum (<100)  
    /// Sectors in this quadrant are outperforming but decelerating
    Weakening,
    
    /// Lagging: Low RS-Ratio (<100), Low RS-Momentum (<100)
    /// Sectors in this quadrant are underperforming and decelerating
    Lagging,
    
    /// Improving: Low RS-Ratio (<100), High RS-Momentum (>100)
    /// Sectors in this quadrant are underperforming but accelerating
    Improving,
}

impl Quadrant {
    /// Get the quadrant based on RS-Ratio and RS-Momentum values
    pub fn from_values(rs_ratio: f64, rs_momentum: f64) -> Self {
        match (rs_ratio >= 100.0, rs_momentum >= 100.0) {
            (true, true) => Quadrant::Leading,
            (true, false) => Quadrant::Weakening,
            (false, false) => Quadrant::Lagging,
            (false, true) => Quadrant::Improving,
        }
    }
    
    /// Get the color associated with this quadrant for visualization
    pub fn color(&self) -> &'static str {
        match self {
            Quadrant::Leading => "#00FF00",    // Green
            Quadrant::Weakening => "#FFFF00", // Yellow
            Quadrant::Lagging => "#FF0000",   // Red
            Quadrant::Improving => "#0000FF", // Blue
        }
    }
    
    /// Get a description of what this quadrant represents
    pub fn description(&self) -> &'static str {
        match self {
            Quadrant::Leading => "Outperforming and accelerating - Strong momentum upward",
            Quadrant::Weakening => "Outperforming but decelerating - Losing momentum",
            Quadrant::Lagging => "Underperforming and decelerating - Weak performance",
            Quadrant::Improving => "Underperforming but accelerating - Building momentum",
        }
    }
    
    /// Get investment implication for this quadrant
    pub fn investment_implication(&self) -> &'static str {
        match self {
            Quadrant::Leading => "Consider overweight position - Strong performance likely to continue",
            Quadrant::Weakening => "Consider reducing position - Performance may deteriorate",
            Quadrant::Lagging => "Consider underweight position - Weak performance may persist",
            Quadrant::Improving => "Consider building position - Performance may improve",
        }
    }
    
    /// Get the typical rotation direction from this quadrant
    pub fn typical_next_quadrant(&self) -> Quadrant {
        match self {
            Quadrant::Leading => Quadrant::Weakening,
            Quadrant::Weakening => Quadrant::Lagging,
            Quadrant::Lagging => Quadrant::Improving,
            Quadrant::Improving => Quadrant::Leading,
        }
    }
    
    /// Check if this quadrant is considered "strong" (Leading or Improving)
    pub fn is_strong(&self) -> bool {
        matches!(self, Quadrant::Leading | Quadrant::Improving)
    }
    
    /// Check if this quadrant is considered "weak" (Weakening or Lagging)
    pub fn is_weak(&self) -> bool {
        matches!(self, Quadrant::Weakening | Quadrant::Lagging)
    }
}

impl fmt::Display for Quadrant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Quadrant::Leading => write!(f, "Leading"),
            Quadrant::Weakening => write!(f, "Weakening"),
            Quadrant::Lagging => write!(f, "Lagging"),
            Quadrant::Improving => write!(f, "Improving"),
        }
    }
}

/// A point on the RRG chart with timestamp and quadrant information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RRGPoint {
    pub symbol: String,
    pub timestamp: DateTime<Utc>,
    pub rs_ratio: f64,
    pub rs_momentum: f64,
    pub quadrant: Quadrant,
    pub strength: f64, // Distance from center (0-1 scale)
}

impl RRGPoint {
    pub fn new(symbol: String, timestamp: DateTime<Utc>, rs_ratio: f64, rs_momentum: f64) -> Self {
        let quadrant = Quadrant::from_values(rs_ratio, rs_momentum);
        let strength = Self::calculate_strength(rs_ratio, rs_momentum);
        
        Self {
            symbol,
            timestamp,
            rs_ratio,
            rs_momentum,
            quadrant,
            strength,
        }
    }
    
    fn calculate_strength(rs_ratio: f64, rs_momentum: f64) -> f64 {
        // Calculate distance from center (100, 100)
        let distance = ((rs_ratio - 100.0).powi(2) + (rs_momentum - 100.0).powi(2)).sqrt();
        
        // Normalize to 0-1 scale (assuming max reasonable distance is ~100)
        (distance / 100.0).min(1.0)
    }
}

/// Quadrant classifier for analyzing RRG movements and transitions
#[derive(Debug, Clone)]
pub struct QuadrantClassifier {
    pub transition_threshold: f64,
    pub strength_threshold: f64,
}

impl QuadrantClassifier {
    pub fn new() -> Self {
        Self {
            transition_threshold: 5.0, // Minimum distance to consider a significant move
            strength_threshold: 0.3,   // Minimum strength to consider a strong position
        }
    }
    
    pub fn with_thresholds(transition_threshold: f64, strength_threshold: f64) -> Self {
        Self {
            transition_threshold,
            strength_threshold,
        }
    }
    
    /// Classify the current position strength
    pub fn classify_strength(&self, point: &RRGPoint) -> PositionStrength {
        if point.strength >= self.strength_threshold {
            PositionStrength::Strong
        } else {
            PositionStrength::Weak
        }
    }
    
    /// Detect transitions between quadrants
    pub fn detect_transitions(&self, points: &[RRGPoint]) -> Vec<QuadrantTransition> {
        if points.len() < 2 {
            return Vec::new();
        }
        
        let mut transitions = Vec::new();
        
        for window in points.windows(2) {
            let prev = &window[0];
            let curr = &window[1];
            
            if prev.quadrant != curr.quadrant {
                // Calculate transition strength based on distance moved
                let distance = ((curr.rs_ratio - prev.rs_ratio).powi(2) + 
                               (curr.rs_momentum - prev.rs_momentum).powi(2)).sqrt();
                
                if distance >= self.transition_threshold {
                    transitions.push(QuadrantTransition {
                        from: prev.quadrant,
                        to: curr.quadrant,
                        timestamp: curr.timestamp,
                        strength: distance,
                        is_clockwise: self.is_clockwise_transition(prev.quadrant, curr.quadrant),
                    });
                }
            }
        }
        
        transitions
    }
    
    /// Calculate probability of being in each quadrant based on position
    pub fn calculate_quadrant_probabilities(&self, rs_ratio: f64, rs_momentum: f64) -> QuadrantProbabilities {
        // Use sigmoid-like function to calculate probabilities based on distance from quadrant boundaries
        let ratio_factor = 1.0 / (1.0 + (-0.1 * (rs_ratio - 100.0)).exp());
        let momentum_factor = 1.0 / (1.0 + (-0.1 * (rs_momentum - 100.0)).exp());
        
        QuadrantProbabilities {
            leading: ratio_factor * momentum_factor,
            weakening: ratio_factor * (1.0 - momentum_factor),
            lagging: (1.0 - ratio_factor) * (1.0 - momentum_factor),
            improving: (1.0 - ratio_factor) * momentum_factor,
        }
    }
    
    /// Analyze the overall trend of a series of points
    pub fn analyze_trend(&self, points: &[RRGPoint]) -> TrendAnalysis {
        if points.len() < 3 {
            return TrendAnalysis::Insufficient;
        }
        
        let recent_points = if points.len() > 10 {
            &points[points.len() - 10..]
        } else {
            points
        };
        
        // Calculate average movement in RS-Ratio and RS-Momentum
        let mut ratio_trend = 0.0;
        let mut momentum_trend = 0.0;
        
        for window in recent_points.windows(2) {
            ratio_trend += window[1].rs_ratio - window[0].rs_ratio;
            momentum_trend += window[1].rs_momentum - window[0].rs_momentum;
        }
        
        ratio_trend /= (recent_points.len() - 1) as f64;
        momentum_trend /= (recent_points.len() - 1) as f64;
        
        // Classify trend based on direction and magnitude
        let trend_magnitude = (ratio_trend.powi(2) + momentum_trend.powi(2)).sqrt();
        
        if trend_magnitude < 1.0 {
            TrendAnalysis::Sideways
        } else if ratio_trend > 0.0 && momentum_trend > 0.0 {
            TrendAnalysis::StrongUpward
        } else if ratio_trend < 0.0 && momentum_trend < 0.0 {
            TrendAnalysis::StrongDownward
        } else if ratio_trend > 0.0 {
            TrendAnalysis::RatioImproving
        } else if momentum_trend > 0.0 {
            TrendAnalysis::MomentumImproving
        } else {
            TrendAnalysis::Mixed
        }
    }
    
    fn is_clockwise_transition(&self, from: Quadrant, to: Quadrant) -> bool {
        matches!(
            (from, to),
            (Quadrant::Leading, Quadrant::Weakening) |
            (Quadrant::Weakening, Quadrant::Lagging) |
            (Quadrant::Lagging, Quadrant::Improving) |
            (Quadrant::Improving, Quadrant::Leading)
        )
    }
}

impl Default for QuadrantClassifier {
    fn default() -> Self {
        Self::new()
    }
}

/// Position strength classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PositionStrength {
    Strong,
    Weak,
}

/// Quadrant transition information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuadrantTransition {
    pub from: Quadrant,
    pub to: Quadrant,
    pub timestamp: DateTime<Utc>,
    pub strength: f64,
    pub is_clockwise: bool,
}

/// Probabilities for each quadrant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuadrantProbabilities {
    pub leading: f64,
    pub weakening: f64,
    pub lagging: f64,
    pub improving: f64,
}

/// Trend analysis results
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrendAnalysis {
    StrongUpward,      // Both RS-Ratio and RS-Momentum increasing
    StrongDownward,    // Both RS-Ratio and RS-Momentum decreasing
    RatioImproving,    // RS-Ratio increasing, RS-Momentum mixed
    MomentumImproving, // RS-Momentum increasing, RS-Ratio mixed
    Sideways,          // Little movement in either direction
    Mixed,             // Conflicting signals
    Insufficient,      // Not enough data
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    
    #[test]
    fn test_quadrant_from_values() {
        assert_eq!(Quadrant::from_values(110.0, 110.0), Quadrant::Leading);
        assert_eq!(Quadrant::from_values(110.0, 90.0), Quadrant::Weakening);
        assert_eq!(Quadrant::from_values(90.0, 90.0), Quadrant::Lagging);
        assert_eq!(Quadrant::from_values(90.0, 110.0), Quadrant::Improving);
    }
    
    #[test]
    fn test_quadrant_properties() {
        assert!(Quadrant::Leading.is_strong());
        assert!(Quadrant::Improving.is_strong());
        assert!(Quadrant::Weakening.is_weak());
        assert!(Quadrant::Lagging.is_weak());
    }
    
    #[test]
    fn test_rrg_point_creation() {
        let point = RRGPoint::new("TEST".to_string(), Utc::now(), 110.0, 90.0);
        assert_eq!(point.quadrant, Quadrant::Weakening);
        assert!(point.strength > 0.0);
    }
    
    #[test]
    fn test_quadrant_classifier() {
        let classifier = QuadrantClassifier::new();
        
        let points = vec![
            RRGPoint::new("TEST".to_string(), Utc::now(), 110.0, 110.0), // Leading
            RRGPoint::new("TEST".to_string(), Utc::now(), 110.0, 90.0),  // Weakening
        ];
        
        let transitions = classifier.detect_transitions(&points);
        assert_eq!(transitions.len(), 1);
        assert_eq!(transitions[0].from, Quadrant::Leading);
        assert_eq!(transitions[0].to, Quadrant::Weakening);
        assert!(transitions[0].is_clockwise);
    }
    
    #[test]
    fn test_quadrant_probabilities() {
        let classifier = QuadrantClassifier::new();
        let probs = classifier.calculate_quadrant_probabilities(110.0, 110.0);
        
        // Should have highest probability for Leading quadrant
        assert!(probs.leading > probs.weakening);
        assert!(probs.leading > probs.lagging);
        assert!(probs.leading > probs.improving);
    }
}