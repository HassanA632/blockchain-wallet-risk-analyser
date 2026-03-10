use serde::{Deserialize, Serialize};

use crate::models::{Chain, Finding, RiskLevel};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportSummary {
    pub risky_wallets_found: usize,
    pub highest_risk_level: Option<RiskLevel>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RiskReport {
    pub target_wallet: String,
    pub chain: Chain,
    pub hop_depth: u8,
    pub summary: ReportSummary,
    pub findings: Vec<Finding>,
}

/// Builds a report summary from the generated findings so the final output
/// includes a quick analyst-facing overview of exposure severity.
pub fn build_summary(findings: &[Finding]) -> ReportSummary {
    let highest_risk_level = findings
        .iter()
        .map(|finding| finding.risk_level.clone())
        .max();

    ReportSummary {
        risky_wallets_found: findings.len(),
        highest_risk_level,
    }
}

/// Builds the final risk report from the analysis results so the CLI can
/// return one structured JSON document for the target wallet.
pub fn build_risk_report(
    target_wallet: String,
    chain: Chain,
    hop_depth: u8,
    findings: Vec<Finding>,
) -> RiskReport {
    let summary = build_summary(&findings);

    RiskReport {
        target_wallet,
        chain,
        hop_depth,
        summary,
        findings,
    }
}
