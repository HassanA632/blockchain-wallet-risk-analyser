use std::fs;
use std::path::Path;

use crate::errors::AppError;
use crate::models::{RiskEntity, TransactionEdge};
use crate::validation::normalize_ethereum_address;

/// Loads transaction edges from a JSON file so the analysis engine can build a wallet interaction graph.
pub fn load_transaction_edges(path: impl AsRef<Path>) -> Result<Vec<TransactionEdge>, AppError> {
    let content = fs::read_to_string(path)?;
    let mut edges: Vec<TransactionEdge> = serde_json::from_str(&content)?;

    for edge in &mut edges {
        edge.from_address = normalize_ethereum_address(&edge.from_address);
        edge.to_address = normalize_ethereum_address(&edge.to_address);
    }

    Ok(edges)
}

/// Loads risk entities from a JSON file so built in risk entities and analyst
/// entities watchlists share the same internal representation.
pub fn load_risk_entities(path: impl AsRef<Path>) -> Result<Vec<RiskEntity>, AppError> {
    let content = fs::read_to_string(path)?;
    let mut entities: Vec<RiskEntity> = serde_json::from_str(&content)?;

    for entity in &mut entities {
        entity.address = normalize_ethereum_address(&entity.address);
    }

    Ok(entities)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_file_path(file_name: &str) -> PathBuf {
        let unique_suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();

        std::env::temp_dir().join(format!("{unique_suffix}_{file_name}"))
    }

    #[test]
    fn normalizes_transaction_edge_addresses_when_loading() {
        let file_path = temp_file_path("transaction_edges.json");

        let json = r#"
        [
            {
                "from_address": "0xAbCdEf1234567890aBCdef1234567890abCDef12",
                "to_address": "0x1234567890ABCDef1234567890abCDef12345678"
            }
        ]
        "#;

        fs::write(&file_path, json).expect("test graph json should be written");

        let edges = load_transaction_edges(&file_path).expect("graph should load successfully");

        assert_eq!(edges.len(), 1);
        assert_eq!(
            edges[0].from_address,
            "0xabcdef1234567890abcdef1234567890abcdef12"
        );
        assert_eq!(
            edges[0].to_address,
            "0x1234567890abcdef1234567890abcdef12345678"
        );

        fs::remove_file(&file_path).expect("test graph json should be removed");
    }

    #[test]
    fn normalizes_risk_entity_addresses_when_loading() {
        let file_path = temp_file_path("risk_entities.json");

        let json = r#"
        [
            {
                "address": "0xAbCdEf1234567890aBCdef1234567890abCDef12",
                "category": "Sanctioned",
                "description": "Known sanctioned wallet"
            }
        ]
        "#;

        fs::write(&file_path, json).expect("test risk json should be written");

        let entities =
            load_risk_entities(&file_path).expect("risk entities should load successfully");

        assert_eq!(entities.len(), 1);
        assert_eq!(
            entities[0].address,
            "0xabcdef1234567890abcdef1234567890abcdef12"
        );

        fs::remove_file(&file_path).expect("test risk json should be removed");
    }
}
