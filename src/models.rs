#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Chain {
    Ethereum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RiskCategory {
    Sanctioned,
    Mixer,
    Suspect,
    Custom,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub address: String,
    pub hop_distance: u8,
    pub category: RiskCategory,
    pub risk_level: RiskLevel,
    pub description: String,
}
