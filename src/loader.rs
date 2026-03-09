use std::fs;
use std::path::Path;

use crate::errors::AppError;
use crate::models::{RiskEntity, TransactionEdge};

/// Loads transaction edges from a JSON file so the analysis engine can build a wallet interaction graph.
pub fn load_transaction_edges(path: impl AsRef<Path>) -> Result<Vec<TransactionEdge>, AppError> {
    let content = fs::read_to_string(path)?;
    let edges = serde_json::from_str(&content)?;
    Ok(edges)
}

/// Loads risk entities from a JSON file so built in risk entities and analyst
/// entities watchlists share the same internal representation.
pub fn load_risk_entities(path: impl AsRef<Path>) -> Result<Vec<RiskEntity>, AppError> {
    let content = fs::read_to_string(path)?;
    let entities = serde_json::from_str(&content)?;
    Ok(entities)
}
