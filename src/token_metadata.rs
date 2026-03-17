use std::str::FromStr;

use alloy::{primitives::Address, providers::Provider, sol};

use crate::errors::AppError;
use crate::models::TokenMetadata;

sol! {
    #[allow(missing_docs)]
    #[sol(rpc)]
    contract IERC20Metadata {
        function symbol() external view returns (string);
        function decimals() external view returns (uint8);
    }
}

/// Resolves ERC-20 metadata for a token contract so output can display
/// readable symbols and token amounts.
pub async fn resolve_token_metadata(
    provider: &impl Provider,
    token_address: &str,
) -> Result<TokenMetadata, AppError> {
    let token_address = Address::from_str(token_address).map_err(|error| {
        AppError::Source(format!(
            "invalid token contract address {token_address}: {error}"
        ))
    })?;

    let contract = IERC20Metadata::new(token_address, provider);

    let symbol = contract.symbol().call().await.map_err(|error| {
        AppError::Source(format!(
            "failed to resolve ERC-20 symbol for {}: {error}",
            token_address
        ))
    })?;

    let decimals = contract.decimals().call().await.map_err(|error| {
        AppError::Source(format!(
            "failed to resolve ERC-20 decimals for {}: {error}",
            token_address
        ))
    })?;

    Ok(TokenMetadata {
        address: format!("{token_address:#x}"),
        symbol,
        decimals,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stores_token_metadata_fields() {
        let metadata = TokenMetadata {
            address: "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_string(),
            symbol: "USDC".to_string(),
            decimals: 6,
        };

        assert_eq!(
            metadata.address,
            "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
        );
        assert_eq!(metadata.symbol, "USDC");
        assert_eq!(metadata.decimals, 6);
    }
}
