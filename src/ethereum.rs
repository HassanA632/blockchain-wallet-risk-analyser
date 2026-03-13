use alloy::providers::ProviderBuilder;

use crate::errors::AppError;
use crate::models::TransactionEdge;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EthereumSourceConfig {
    pub rpc_url: String,
}

/// Loads transaction data for a wallet from Ethereum.
pub async fn load_transaction_edges_from_ethereum(
    wallet: &str,
    config: &EthereumSourceConfig,
) -> Result<Vec<TransactionEdge>, AppError> {
    let _provider = build_provider(config).await?;

    Err(AppError::Source(format!(
        "Ethereum source is not implemented yet for wallet {wallet}"
    )))
}

/// Reads Ethereum source configuration from the environment so the source can
/// connect to an RPC endpoint without hardcoding provider details.
pub fn load_ethereum_source_config() -> Result<EthereumSourceConfig, AppError> {
    let rpc_url = std::env::var("ETH_RPC_URL").ok();
    ethereum_source_config_from_rpc_url(rpc_url.as_deref())
}

/// Builds Ethereum source configuration from an optional RPC URL so config
/// validation can be tested without mutating global environment variables.
fn ethereum_source_config_from_rpc_url(
    rpc_url: Option<&str>,
) -> Result<EthereumSourceConfig, AppError> {
    let rpc_url = rpc_url.ok_or_else(|| {
        AppError::Source("ETH_RPC_URL environment variable is not set".to_string())
    })?;

    Ok(EthereumSourceConfig {
        rpc_url: rpc_url.to_string(),
    })
}

/// Builds an Alloy provider from the current Ethereum source config so
/// later ingestion code can reuse one setup path.
async fn build_provider(
    config: &EthereumSourceConfig,
) -> Result<impl alloy::providers::Provider, AppError> {
    ProviderBuilder::new()
        .connect(&config.rpc_url)
        .await
        .map_err(|error| AppError::Source(format!("failed to connect to Ethereum RPC: {error}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_ethereum_source_config_from_rpc_url() {
        let config = ethereum_source_config_from_rpc_url(Some("http://127.0.0.1:8545"))
            .expect("config should build from rpc url");

        assert_eq!(config.rpc_url, "http://127.0.0.1:8545");
    }

    #[test]
    fn returns_error_when_rpc_url_is_missing() {
        let result = ethereum_source_config_from_rpc_url(None);

        match result {
            Err(AppError::Source(message)) => {
                assert_eq!(message, "ETH_RPC_URL environment variable is not set");
            }
            _ => panic!("expected missing ETH_RPC_URL source error"),
        }
    }
}
