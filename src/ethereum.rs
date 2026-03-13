use crate::errors::AppError;
use crate::models::TransactionEdge;

/// Loads transaction data for a wallet from Ethereum.
pub fn load_transaction_edges_from_ethereum(
    wallet: &str,
) -> Result<Vec<TransactionEdge>, AppError> {
    Err(AppError::Source(format!(
        "Ethereum source is not implemented yet for wallet {wallet}"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_stub_error_for_ethereum_loader() {
        let result =
            load_transaction_edges_from_ethereum("0x1111111111111111111111111111111111111111");

        match result {
            Err(AppError::Source(message)) => {
                assert_eq!(
                    message,
                    "Ethereum source is not implemented yet for wallet 0x1111111111111111111111111111111111111111"
                );
            }
            _ => panic!("expected source error from ethereum loader scaffold"),
        }
    }
}
