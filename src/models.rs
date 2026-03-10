use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Chain {
    Ethereum,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskCategory {
    Sanctioned,
    Mixer,
    Suspect,
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
    pub risk_level: RiskLevel,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionEdge {
    pub from_address: String,
    pub to_address: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RiskEntity {
    pub address: String,
    pub category: RiskCategory,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveredWallet {
    pub address: String,
    pub hop_distance: u8,
    pub path: Vec<String>,
}
