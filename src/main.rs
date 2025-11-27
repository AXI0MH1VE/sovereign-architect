//! Sovereign Architect CLI
//!
//! Command-line interface for OLO Core fragility analysis.

use clap::{Parser, Subcommand};
use sovereign_architect::*;
use std::error::Error;

#[derive(Parser)]
#[command(name = "olo")]
#[command(about = "Omni-Lagrangian Oracle - Financial Fragility Detection", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compute fragility score for a bank state
    Fragility {
        #[arg(short, long)]
        assets: f64,
        #[arg(short, long)]
        liabilities: f64,
        #[arg(short, long)]
        equity: f64,
        #[arg(short = 'v', long)]
        leverage: f64,
    },
    /// Run Monte Carlo simulation
    Simulate {
        #[arg(short, long)]
        assets: f64,
        #[arg(short, long)]
        liabilities: f64,
        #[arg(short, long)]
        equity: f64,
        #[arg(short = 'v', long)]
        leverage: f64,
        #[arg(short, long, default_value_t = 10000)]
        iterations: usize,
    },
    /// Calculate portfolio entropy
    Entropy {
        #[arg(short, long)]
        weights: Vec<f64>,
    },
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Fragility {
            assets,
            liabilities,
            equity,
            leverage,
        } => {
            let state = BankState {
                assets,
                liabilities,
                equity,
                leverage,
            };

            let config = LagrangianConfig::default();
            let fragility = compute_fragility(&state, &config);

            println!(\"Bank State:\");
            println!(\"  Assets: ${:.2}\", assets);
            println!(\"  Liabilities: ${:.2}\", liabilities);
            println!(\"  Equity: ${:.2}\", equity);
            println!(\"  Leverage: {:.2}x\", leverage);
            println!(\"\");
            println!(\"Fragility Score: {:.4}\", fragility);

            if fragility > 20.0 {
                println!(\"⚠️  HIGH RISK - System approaching critical instability\");
            } else if fragility > 10.0 {
                println!(\"⚡ MEDIUM RISK - Elevated fragility detected\");
            } else {
                println!(\"✅ LOW RISK - System appears stable\");
            }
        }

        Commands::Simulate {
            assets,
            liabilities,
            equity,
            leverage,
            iterations,
        } => {
            let state = BankState {
                assets,
                liabilities,
                equity,
                leverage,
            };

            let lag_config = LagrangianConfig::default();
            let mc_config = MonteCarloConfig {
                num_simulations: iterations,
                ..Default::default()
            };

            println!(\"Running {} Monte Carlo simulations...\", iterations);
            let result = run_simulation(&state, &lag_config, &mc_config);

            println!(\"\");
            println!(\"Simulation Results:\");
            println!(\"  Mean Fragility: {:.4}\", result.mean);
            println!(\"  Std Deviation: {:.4}\", result.std_dev);
            println!(\"  95% VaR: {:.4}\", result.var_95);
            println!(\"  99% VaR: {:.4}\", result.var_99);
            println!(\"  Max Fragility: {:.4}\", result.max_fragility);
        }

        Commands::Entropy { weights } => {
            let positions: Vec<Position> = weights
                .iter()
                .enumerate()
                .map(|(i, &w)| Position {
                    asset: format!(\"Asset{}\", i + 1),
                    weight: w,
                })
                .collect();

            let config = EntropyConfig::default();
            let entropy = calculate_entropy(&positions, &config);
            let conc_risk = concentration_risk(&positions, &config);

            println!(\"Portfolio Entropy Analysis:\");
            println!(\"  Shannon Entropy: {:.4} bits\", entropy);
            println!(\"  Concentration Risk: {:.2}%\", conc_risk * 100.0);

            if conc_risk > 0.7 {
                println!(\"⚠️  HIGH CONCENTRATION - Portfolio highly concentrated\");
            } else if conc_risk > 0.4 {
                println!(\"⚡ MEDIUM CONCENTRATION - Consider diversification\");
            } else {
                println!(\"✅ WELL DIVERSIFIED - Healthy portfolio distribution\");
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_cli() {
        // CLI test would go here
        assert!(true);
    }
}
