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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Finding, RiskCategory, RiskLevel};

    fn sample_findings() -> Vec<Finding> {
        vec![
            Finding {
                address: "0xone".to_string(),
                hop_distance: 1,
                category: RiskCategory::Custom,
                risk_level: RiskLevel::Medium,
                description: "Analyst watchlist entry".to_string(),
                path: vec!["0xtarget".to_string(), "0xone".to_string()],
            },
            Finding {
                address: "0xtwo".to_string(),
                hop_distance: 1,
                category: RiskCategory::Sanctioned,
                risk_level: RiskLevel::High,
                description: "Known sanctioned wallet".to_string(),
                path: vec!["0xtarget".to_string(), "0xtwo".to_string()],
            },
        ]
    }

    #[test]
    fn builds_summary_with_correct_count_and_highest_risk() {
        let summary = build_summary(&sample_findings());

        assert_eq!(summary.risky_wallets_found, 2);
        assert_eq!(summary.highest_risk_level, Some(RiskLevel::High));
    }

    #[test]
    fn builds_summary_with_no_highest_risk_for_empty_findings() {
        let summary = build_summary(&[]);

        assert_eq!(summary.risky_wallets_found, 0);
        assert_eq!(summary.highest_risk_level, None);
    }
}
