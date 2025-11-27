//! # Lagrangian Physics Engine
//!
//! The heart of OLO Core - calculates financial fragility using constraint optimization.
//! This module implements the Omni-Lagrangian Fragility Score using exponential barrier functions
//! and thermodynamic entropy penalties.

use ndarray::{Array2, Array1};
use serde::{Deserialize, Serialize};

/// Bank state vector containing regulatory metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankState {
    /// Tier 1 Capital (CET1) - Core equity capital
    pub tier1_capital: f64,
    
    /// Total Risk-Weighted Assets
    pub total_assets: f64,
    
    /// Liquidity Coverage Ratio (Basel III)
    pub liquidity_coverage: f64,
    
    /// Shannon entropy index from asset diversity
    /// Calculated from portfolio concentration: H = -Σ(p_i * log(p_i))
    pub entropy_index: f64,
}

/// Configuration for Lagrangian multiplier calculation
#[derive(Debug, Clone)]
pub struct LagrangianConfig {
    /// Lambda sensitivity parameter - controls stress spike rate
    /// Higher values = faster exponential growth as constraints approach violation
    pub lambda_sensitivity: f64,
    
    /// Regulatory minimum capital ratio (Basel III: typically 0.08 = 8%)
    pub regulatory_min_capital: f64,
}

impl Default for LagrangianConfig {
    fn default() -> Self {
        LagrangianConfig {
            lambda_sensitivity: 2.0,
            regulatory_min_capital: 0.08,
        }
    }
}

/// The Omni-Lagrangian Fragility Calculator
/// 
/// Computes system fragility score using constrained optimization theory.
/// The score represents the "shadow price" of regulatory constraint violations.
/// 
/// # Mathematical Framework
/// 
/// L(x, λ) = f(x) + λ * g(x)
/// 
/// Where:
/// - f(x) = objective function (portfolio entropy + liquidity stress)
/// - g(x) = constraint function (capital adequacy)
/// - λ = Lagrangian multiplier (shadow price of stress)
/// 
/// # Returns
/// 
/// Normalized fragility score in range [0, 100]
/// - 0-30: Low fragility (well-capitalized)
/// - 30-70: Medium fragility (stressed)
/// - 70-100: High fragility (near-insolvency)
///
/// # Example
/// 
/// ```
/// use olo_core::core::lagrangian::{BankState, LagrangianConfig, compute_fragility};
/// 
/// let bank = BankState {
///     tier1_capital: 10_000.0,
///     total_assets: 100_000.0,
///     liquidity_coverage: 1.2,
///     entropy_index: 2.5,
/// };
/// 
/// let config = LagrangianConfig::default();
/// let fragility = compute_fragility(&bank, &config);
/// ```
pub fn compute_fragility(bank: &BankState, config: &LagrangianConfig) -> f64 {
    // STEP 1: Calculate Capital Constraint Distance g(x)
    // Constraint: tier1_capital >= regulatory_min * total_assets
    // If violated (distance < 0), bank is technically insolvent
    let constraint_distance = bank.tier1_capital - (bank.total_assets * config.regulatory_min_capital);
    
    // STEP 2: Compute Lagrangian Multiplier λ (Shadow Price of Stress)
    // Using exponential barrier function: λ = α * exp(-g(x))
    // As g(x) → 0, λ → ∞ (infinite stress)
    // This models the non-linear "cliff effect" in financial fragility
    let lambda = if constraint_distance <= 0.0 {
        // Cap at mathematical insolvency threshold
        1000.0
    } else {
        // Exponential barrier: stress spikes as constraint approaches
        config.lambda_sensitivity * (-1.0 * constraint_distance).exp()
    };

    // STEP 3: Thermodynamic Entropy Penalty
    // Higher entropy (portfolio disorder) = higher systemic risk
    // Entropy measures concentration risk via Shannon information theory
    // Penalty weight: 1.5x multiplier for entropy contribution
    let entropy_penalty = bank.entropy_index * 1.5;

    // STEP 4: Liquidity Stress Component
    // Inverse relationship: lower LCR = higher liquidity stress
    // LCR < 1.0 means insufficient liquid assets for 30-day stress
    let liquidity_stress = (1.0 / bank.liquidity_coverage) * 10.0;

    // STEP 5: Composite Raw Score
    // Sum all stress components
    let raw_score = lambda + entropy_penalty + liquidity_stress;
    
    // STEP 6: Sigmoid Normalization to [0, 100]
    // Maps (0, ∞) → (0, 100) using logistic function
    // This ensures interpretable scores regardless of input magnitudes
    let normalized_score = 100.0 * (raw_score / (raw_score + 50.0));
    
    // Clamp to valid range (defensive programming)
    normalized_score.max(0.0).min(100.0)
}

/// Calculate capital adequacy ratio (CAR)
/// 
/// CAR = Tier1 Capital / Risk-Weighted Assets
/// 
/// Basel III minimum: 8%
/// Well-capitalized threshold: 10%
pub fn capital_adequacy_ratio(bank: &BankState) -> f64 {
    bank.tier1_capital / bank.total_assets
}

/// Check if bank meets regulatory capital requirements
pub fn is_adequately_capitalized(bank: &BankState, min_ratio: f64) -> bool {
    capital_adequacy_ratio(bank) >= min_ratio
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_well_capitalized_bank() {
        let bank = BankState {
            tier1_capital: 15_000.0,  // 15% ratio
            total_assets: 100_000.0,
            liquidity_coverage: 1.5,
            entropy_index: 2.0,
        };
        
        let config = LagrangianConfig::default();
        let fragility = compute_fragility(&bank, &config);
        
        assert!(fragility < 50.0, "Well-capitalized bank should have low fragility");
    }

    #[test]
    fn test_undercapitalized_bank() {
        let bank = BankState {
            tier1_capital: 5_000.0,   // 5% ratio - below minimum
            total_assets: 100_000.0,
            liquidity_coverage: 0.8,   // Below 1.0 threshold
            entropy_index: 3.5,        // High concentration risk
        };
        
        let config = LagrangianConfig::default();
        let fragility = compute_fragility(&bank, &config);
        
        assert!(fragility > 70.0, "Undercapitalized bank should have high fragility");
    }

    #[test]
    fn test_capital_adequacy_ratio() {
        let bank = BankState {
            tier1_capital: 10_000.0,
            total_assets: 100_000.0,
            liquidity_coverage: 1.0,
            entropy_index: 2.0,
        };
        
        let car = capital_adequacy_ratio(&bank);
        assert_eq!(car, 0.10);
    }
}
