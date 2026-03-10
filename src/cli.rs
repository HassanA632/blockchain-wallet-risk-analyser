use clap::Parser;

use crate::models::Chain;

/// Defines the command line inputs required to run a wallet exposure analysis
/// so analysts can supply the target, scope, and optional custom risk data.
#[derive(Debug, Parser)]
#[command(name = "blockchain-wallet-risk-analyser")]
pub struct CliArgs {
    #[arg(long)]
    pub chain: Chain,

    #[arg(long)]
    pub wallet: String,

    #[arg(long, value_parser = clap::value_parser!(u8).range(1..=2))]
    pub hops: u8,

    #[arg(long = "custom-risk-list")]
    pub custom_risk_list: Option<String>,

    #[arg(long)]
    pub output: Option<String>,
}
