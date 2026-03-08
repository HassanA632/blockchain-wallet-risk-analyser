use crate::models::{Chain, Finding, RiskLevel};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportSummary {
    pub risky_wallets_found: usize,
    pub highest_risk_level: Option<RiskLevel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RiskReport {
    pub target_wallet: String,
    pub chain: Chain,
    pub hop_depth: u8,
    pub summary: ReportSummary,
    pub findings: Vec<Finding>,
}
