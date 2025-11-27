//! Monte Carlo Simulation Engine for Fragility Analysis
//!
//! Parallel stress testing of financial systems using Monte Carlo methods.
//! Simulates thousands of scenarios to compute Value-at-Risk and tail risk.

use rayon::prelude::*;
use rand::distributions::Distribution;
use rand::rngs::StdRng;
use rand::SeedableRng;
use rand_distr::Normal;
use std::sync::Arc;

use crate::core::lagrangian::{BankState, LagrangianConfig, compute_fragility};

/// Monte Carlo configuration
#[derive(Debug, Clone)]
pub struct MonteCarloConfig {
    /// Number of simulation paths
    pub num_simulations: usize,
    /// Random seed for reproducibility
    pub seed: u64,
    /// Shock magnitude (standard deviations)
    pub shock_size: f64,
    /// Parallel threads (0 = auto)
    pub num_threads: usize,
}

impl Default for MonteCarloConfig {
    fn default() -> Self {
        Self {
            num_simulations: 10_000,
            seed: 42,
            shock_size: 2.0,
            num_threads: 0,
        }
    }
}

/// Simulation result
#[derive(Debug, Clone)]
pub struct SimulationResult {
    /// Fragility scores for all paths
    pub fragilities: Vec<f64>,
    /// Mean fragility
    pub mean: f64,
    /// Standard deviation
    pub std_dev: f64,
    /// 95% Value-at-Risk
    pub var_95: f64,
    /// 99% Value-at-Risk
    pub var_99: f64,
    /// Maximum fragility observed
    pub max_fragility: f64,
}

/// Run Monte Carlo simulation
///
/// Applies random shocks to bank state and computes fragility distribution
pub fn run_simulation(
    base_state: &BankState,
    lag_config: &LagrangianConfig,
    mc_config: &MonteCarloConfig,
) -> SimulationResult {
    let lag_config = Arc::new(lag_config.clone());
    
    // Generate all random shocks upfront
    let mut rng = StdRng::seed_from_u64(mc_config.seed);
    let normal = Normal::new(0.0, mc_config.shock_size).unwrap();
    
    let shocks: Vec<(f64, f64, f64, f64)> = (0..mc_config.num_simulations)
        .map(|_| {
            (
                normal.sample(&mut rng),
                normal.sample(&mut rng),
                normal.sample(&mut rng),
                normal.sample(&mut rng),
            )
        })
        .collect();
    
    // Parallel simulation
    let fragilities: Vec<f64> = shocks
        .par_iter()
        .map(|(shock_assets, shock_liab, shock_equity, shock_lev)| {
            let shocked_state = BankState {
                assets: (base_state.assets * (1.0 + shock_assets * 0.01)).max(0.0),
                liabilities: (base_state.liabilities * (1.0 + shock_liab * 0.01)).max(0.0),
                equity: (base_state.equity * (1.0 + shock_equity * 0.01)).max(0.0),
                leverage: (base_state.leverage * (1.0 + shock_lev * 0.01)).max(0.0),
            };
            compute_fragility(&shocked_state, &lag_config)
        })
        .collect();
    
    // Compute statistics
    let mut sorted = fragilities.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    
    let mean = fragilities.iter().sum::<f64>() / fragilities.len() as f64;
    let variance = fragilities.iter()
        .map(|x| (x - mean).powi(2))
        .sum::<f64>() / fragilities.len() as f64;
    let std_dev = variance.sqrt();
    
    let var_95_idx = (0.95 * fragilities.len() as f64) as usize;
    let var_99_idx = (0.99 * fragilities.len() as f64) as usize;
    
    SimulationResult {
        fragilities,
        mean,
        std_dev,
        var_95: sorted[var_95_idx.min(sorted.len() - 1)],
        var_99: sorted[var_99_idx.min(sorted.len() - 1)],
        max_fragility: sorted[sorted.len() - 1],
    }
}

/// Calculate tail risk metrics
pub fn calculate_tail_risk(result: &SimulationResult, threshold: f64) -> f64 {
    let exceedances = result.fragilities.iter()
        .filter(|&&f| f > threshold)
        .count();
    exceedances as f64 / result.fragilities.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monte_carlo_basic() {
        let base_state = BankState {
            assets: 1000.0,
            liabilities: 900.0,
            equity: 100.0,
            leverage: 9.0,
        };
        
        let lag_config = LagrangianConfig::default();
        let mc_config = MonteCarloConfig {
            num_simulations: 1000,
            ..Default::default()
        };
        
        let result = run_simulation(&base_state, &lag_config, &mc_config);
        
        assert_eq!(result.fragilities.len(), 1000);
        assert!(result.mean >= 0.0);
        assert!(result.std_dev >= 0.0);
        assert!(result.var_99 >= result.var_95);
        assert!(result.max_fragility >= result.var_99);
    }
    
    #[test]
    fn test_tail_risk() {
        let base_state = BankState {
            assets: 1000.0,
            liabilities: 950.0,
            equity: 50.0,
            leverage: 19.0,
        };
        
        let lag_config = LagrangianConfig::default();
        let mc_config = MonteCarloConfig {
            num_simulations: 1000,
            shock_size: 3.0,
            ..Default::default()
        };
        
        let result = run_simulation(&base_state, &lag_config, &mc_config);
        let tail_risk = calculate_tail_risk(&result, result.mean);
        
        // Approximately 50% should exceed mean in normal distribution
        assert!(tail_risk > 0.4 && tail_risk < 0.6);
    }
}
