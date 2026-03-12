use clap::Parser;

use crate::models::Chain;
use crate::validation::{validate_date_range, validate_ethereum_address, validate_utc_timestamp};

/// Defines the command-line inputs required to run a wallet exposure analysis
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

    #[arg(long = "from-date", value_parser = validate_utc_timestamp)]
    pub from_date: Option<String>,

    #[arg(long = "to-date", value_parser = validate_utc_timestamp)]
    pub to_date: Option<String>,
}

impl CliArgs {
    /// Validates CLI constraints so date filters are logically
    /// consistent before the analysis pipeline runs.
    pub fn validate(&self) -> Result<(), String> {
        validate_date_range(self.from_date.as_deref(), self.to_date.as_deref())
    }
}
