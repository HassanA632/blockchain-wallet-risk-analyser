use std::path::PathBuf;

use crate::errors::AppError;
use crate::ethereum::load_transaction_edges_from_ethereum;
use crate::loader::load_transaction_edges;
use crate::models::TransactionEdge;

/// Represents the source used to load transaction edge data so the analysis
/// pipeline can stay independent from where interaction data comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionEdgeSource {
    LocalFile { path: PathBuf },
    Ethereum { wallet: String },
}

/// Loads transaction data from the chosen source.
pub fn load_edges_from_source(
    source: &TransactionEdgeSource,
) -> Result<Vec<TransactionEdge>, AppError> {
    match source {
        TransactionEdgeSource::LocalFile { path } => load_transaction_edges(path),
        TransactionEdgeSource::Ethereum { wallet } => load_transaction_edges_from_ethereum(wallet),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_file_path(file_name: &str) -> PathBuf {
        let unique_suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();

        std::env::temp_dir().join(format!("{unique_suffix}_{file_name}"))
    }

    #[test]
    fn loads_transaction_edges_from_local_file_source() {
        let file_path = temp_file_path("source_transaction_edges.json");

        let json = r#"
        [
            {
                "from_address": "0xAbCdEf1234567890aBCdef1234567890abCDef12",
                "to_address": "0x1234567890ABCDef1234567890abCDef12345678",
                "tx_hash": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "asset": "ETH",
                "amount": "1.50",
                "timestamp": "2026-03-11T12:00:00Z"
            }
        ]
        "#;

        fs::write(&file_path, json).expect("test source json should be written");

        let source = TransactionEdgeSource::LocalFile {
            path: file_path.clone(),
        };

        let edges = load_edges_from_source(&source).expect("local source should load edges");

        assert_eq!(edges.len(), 1);
        assert_eq!(
            edges[0].from_address,
            "0xabcdef1234567890abcdef1234567890abcdef12"
        );
        assert_eq!(
            edges[0].to_address,
            "0x1234567890abcdef1234567890abcdef12345678"
        );

        fs::remove_file(&file_path).expect("test source json should be removed");
    }

    #[test]
    fn returns_stub_error_for_unimplemented_ethereum_source() {
        let source = TransactionEdgeSource::Ethereum {
            wallet: "0x1111111111111111111111111111111111111111".to_string(),
        };

        let result = load_edges_from_source(&source);

        match result {
            Err(AppError::Source(message)) => {
                assert_eq!(
                    message,
                    "Ethereum source is not implemented yet for wallet 0x1111111111111111111111111111111111111111"
                );
            }
            _ => panic!("expected source error for unimplemented ethereum source"),
        }
    }
}
