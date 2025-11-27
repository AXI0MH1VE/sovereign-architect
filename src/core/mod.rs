//! # Core Module
//!
//! Financial physics engine for OLO Core.
//! Contains Lagrangian constraint optimization and entropy calculations.

pub mod lagrangian;
pub mod entropy;

// Re-export key types
pub use lagrangian::{BankState, LagrangianConfig, compute_fragility};
pub use entropy::{calculate_portfolio_entropy, EntropyConfig};
