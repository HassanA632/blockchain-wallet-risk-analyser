use clap::Parser;

use crate::models::{Chain, DataSource};
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

    #[arg(long, default_value = "local")]
    pub source: DataSource,

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
    /// Validates CLI constraints so date filters are logically consistent and
    /// app fails on obvious invalid CLI combinations before the analysis pipeline runs.
    pub fn validate(&self) -> Result<(), String> {
        validate_date_range(self.from_date.as_deref(), self.to_date.as_deref())?;

        if self.source == DataSource::Ethereum && self.graph.is_some() {
            return Err("--graph can only be used with --source local".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_local_args() -> CliArgs {
        CliArgs {
            chain: Chain::Ethereum,
            wallet: "0x1111111111111111111111111111111111111111".to_string(),
            hops: 2,
            source: DataSource::Local,
            graph: None,
            custom_risk_list: None,
            output: None,
            from_date: None,
            to_date: None,
        }
    }

    #[test]
    fn accepts_graph_with_local_source() {
        let mut args = valid_local_args();
        args.graph = Some("data/sample_graph.json".to_string());

        let result = args.validate();

        assert_eq!(result, Ok(()));
    }

    #[test]
    fn rejects_graph_with_ethereum_source() {
        let mut args = valid_local_args();
        args.source = DataSource::Ethereum;
        args.graph = Some("data/sample_graph.json".to_string());

        let result = args.validate();

        assert_eq!(
            result,
            Err("--graph can only be used with --source local".to_string())
        );
    }
}
