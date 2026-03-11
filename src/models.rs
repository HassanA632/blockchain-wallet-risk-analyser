use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
pub enum Chain {
    Ethereum,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskCategory {
    Sanctioned,
    Mixer,
    Suspect,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskSource {
    BuiltIn,
    Custom,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    pub address: String,
    pub hop_distance: u8,
    pub category: RiskCategory,
    pub source: RiskSource,
    pub risk_level: RiskLevel,
    pub description: String,
    pub path: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionEdge {
    pub from_address: String,
    pub to_address: String,
    pub tx_hash: String,
    pub asset: String,
    pub amount: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RiskEntity {
    pub address: String,
    pub category: RiskCategory,
    pub source: RiskSource,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveredWallet {
    pub address: String,
    pub hop_distance: u8,
    pub path: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WalletRelationship {
    pub wallet_a: String,
    pub wallet_b: String,
    pub transaction_count: usize,
    pub assets_seen: Vec<String>,
    pub totals_by_asset: BTreeMap<String, f64>,
    pub latest_timestamp: String,
}
