//! Sovereign Architect - OLO Core Library
//!
//! Omni-Lagrangian Oracle: Physics-based financial fragility detection framework.
//! Deploy nation-state-scale risk analysis infrastructure as a solo operator.

pub mod core;
pub mod simulation;
pub mod proofs;
pub mod network;

// Re-export key types
pub use core::lagrangian::{BankState, LagrangianConfig, compute_fragility};
pub use core::entropy::{Position, EntropyConfig, calculate_entropy, concentration_risk};
pub use simulation::monte_carlo::{MonteCarloConfig, SimulationResult, run_simulation};
pub use proofs::prover::{FragilityProver, FragilityCircuit};
pub use network::ingestion::{IngestionEngine, NetworkConfig, DataPacket};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_pipeline() {
        // Create bank state
        let state = BankState {
            assets: 1000.0,
            liabilities: 900.0,
            equity: 100.0,
            leverage: 9.0,
        };

        // Compute fragility
        let lag_config = LagrangianConfig::default();
        let fragility = compute_fragility(&state, &lag_config);
        
        assert!(fragility > 0.0);
        assert!(fragility.is_finite());
    }

    #[test]
    fn test_entropy_calculation() {
        let positions = vec![
            Position { asset: \"BTC\".to_string(), weight: 0.5 },
            Position { asset: \"ETH\".to_string(), weight: 0.3 },
            Position { asset: \"SOL\".to_string(), weight: 0.2 },
        ];

        let config = EntropyConfig::default();
        let entropy = calculate_entropy(&positions, &config);
        
        assert!(entropy > 0.0);
        assert!(entropy < 2.0); // Max entropy for 3 assets is log2(3) ≈ 1.585
    }
}
