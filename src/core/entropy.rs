//! Shannon Entropy Calculator for Portfolio Concentration Risk
//!
//! Measures information diversity in portfolio allocations using Shannon entropy.
//! Higher entropy = more diversified portfolio = lower concentration risk.

use std::collections::HashMap;

/// Portfolio position with weight
#[derive(Debug, Clone)]
pub struct Position {
    pub asset: String,
    pub weight: f64,
}

/// Entropy configuration
#[derive(Debug, Clone)]
pub struct EntropyConfig {
    /// Minimum weight threshold (ignore positions below this)
    pub min_weight: f64,
    /// Normalize weights to sum to 1.0
    pub normalize: bool,
}

impl Default for EntropyConfig {
    fn default() -> Self {
        Self {
            min_weight: 1e-6,
            normalize: true,
        }
    }
}

/// Calculate Shannon entropy of portfolio weights
///
/// H = -Σ(p_i * log2(p_i))
///
/// # Arguments
/// * `positions` - Portfolio positions with weights
/// * `config` - Entropy calculation configuration
///
/// # Returns
/// Shannon entropy in bits (higher = more diversified)
pub fn calculate_entropy(positions: &[Position], config: &EntropyConfig) -> f64 {
    // Filter positions above minimum weight
    let filtered: Vec<f64> = positions
        .iter()
        .map(|p| p.weight)
        .filter(|&w| w >= config.min_weight)
        .collect();

    if filtered.is_empty() {
        return 0.0;
    }

    // Normalize weights if requested
    let weights = if config.normalize {
        let sum: f64 = filtered.iter().sum();
        if sum < 1e-10 {
            return 0.0;
        }
        filtered.iter().map(|&w| w / sum).collect()
    } else {
        filtered
    };

    // Calculate Shannon entropy
    weights
        .iter()
        .filter(|&&w| w > 0.0)
        .map(|&w| -w * w.log2())
        .sum()
}

/// Calculate normalized entropy (0-1 scale)
///
/// Divides entropy by maximum possible entropy log2(N)
pub fn normalized_entropy(positions: &[Position], config: &EntropyConfig) -> f64 {
    let entropy = calculate_entropy(positions, config);
    let n = positions.len() as f64;
    
    if n <= 1.0 {
        return 0.0;
    }
    
    entropy / n.log2()
}

/// Calculate concentration risk metric (inverse of normalized entropy)
///
/// Returns value in [0, 1] where 1 = maximum concentration
pub fn concentration_risk(positions: &[Position], config: &EntropyConfig) -> f64 {
    1.0 - normalized_entropy(positions, config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uniform_distribution() {
        let positions = vec![
            Position { asset: "A".to_string(), weight: 0.25 },
            Position { asset: "B".to_string(), weight: 0.25 },
            Position { asset: "C".to_string(), weight: 0.25 },
            Position { asset: "D".to_string(), weight: 0.25 },
        ];
        
        let config = EntropyConfig::default();
        let entropy = calculate_entropy(&positions, &config);
        
        // Uniform distribution should have maximum entropy = log2(4) = 2.0
        assert!((entropy - 2.0).abs() < 1e-10);
        assert!((normalized_entropy(&positions, &config) - 1.0).abs() < 1e-10);
        assert!(concentration_risk(&positions, &config) < 1e-10);
    }

    #[test]
    fn test_concentrated_portfolio() {
        let positions = vec![
            Position { asset: "A".to_string(), weight: 0.97 },
            Position { asset: "B".to_string(), weight: 0.01 },
            Position { asset: "C".to_string(), weight: 0.01 },
            Position { asset: "D".to_string(), weight: 0.01 },
        ];
        
        let config = EntropyConfig::default();
        let norm_entropy = normalized_entropy(&positions, &config);
        let conc_risk = concentration_risk(&positions, &config);
        
        // Highly concentrated portfolio should have low normalized entropy
        assert!(norm_entropy < 0.5);
        // And high concentration risk
        assert!(conc_risk > 0.5);
    }

    #[test]
    fn test_single_asset() {
        let positions = vec![
            Position { asset: "A".to_string(), weight: 1.0 },
        ];
        
        let config = EntropyConfig::default();
        let entropy = calculate_entropy(&positions, &config);
        
        // Single asset has zero entropy
        assert!(entropy.abs() < 1e-10);
        assert!(concentration_risk(&positions, &config) > 0.99);
    }
}
