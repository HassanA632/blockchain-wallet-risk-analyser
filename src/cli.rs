use clap::Parser;

use crate::models::Chain;
use crate::validation::validate_ethereum_address;

/// Defines the command line inputs required to run a wallet exposure analysis
/// so analysts can supply the target, scope, and optional custom risk data.
#[derive(Debug, Parser)]
#[command(name = "blockchain-wallet-risk-analyser")]
pub struct CliArgs {
    #[arg(long)]
    pub chain: Chain,

    #[arg(long, value_parser = validate_ethereum_address)]
    pub wallet: String,

    #[arg(long, value_parser = clap::value_parser!(u8).range(1..=2))]
    pub hops: u8,

    #[arg(long)]
    pub graph: Option<String>,

    #[arg(long = "custom-risk-list")]
    pub custom_risk_list: Option<String>,

    #[arg(long)]
    pub output: Option<String>,

    #[arg(long = "from-date")]
    pub from_date: Option<String>,

    #[arg(long = "to-date")]
    pub to_date: Option<String>,
}
