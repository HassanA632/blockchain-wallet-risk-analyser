use serde::{Deserialize, Serialize};

use crate::models::{Chain, Finding, RiskLevel};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportSummary {
    pub risky_wallets_found: usize,
    pub highest_risk_level: Option<RiskLevel>,
    pub direct_exposure_count: usize,
    pub indirect_exposure_count: usize,
    pub low_risk_count: usize,
    pub medium_risk_count: usize,
    pub high_risk_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    let highest_risk_level: Option<RiskLevel> = findings
        .iter()
        .map(|finding| finding.risk_level.clone())
        .max();

    let direct_exposure_count = findings
        .iter()
        .filter(|finding| finding.hop_distance == 1)
        .count();

    let indirect_exposure_count = findings
        .iter()
        .filter(|finding| finding.hop_distance == 2)
        .count();

    let low_risk_count = findings
        .iter()
        .filter(|finding| finding.risk_level == RiskLevel::Low)
        .count();

    let medium_risk_count = findings
        .iter()
        .filter(|finding| finding.risk_level == RiskLevel::Medium)
        .count();

    let high_risk_count = findings
        .iter()
        .filter(|finding| finding.risk_level == RiskLevel::High)
        .count();

    ReportSummary {
        risky_wallets_found: findings.len(),
        highest_risk_level,
        direct_exposure_count,
        indirect_exposure_count,
        low_risk_count,
        medium_risk_count,
        high_risk_count,
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
    use std::collections::BTreeMap;

    use super::*;
    use crate::models::{Finding, RelationshipStep, RiskCategory, RiskLevel, RiskSource};

    fn sample_findings() -> Vec<Finding> {
        vec![
            Finding {
                address: "0xone".to_string(),
                hop_distance: 1,
                category: RiskCategory::Other,
                source: RiskSource::Custom,
                risk_level: RiskLevel::Medium,
                description: "Analyst watchlist entry".to_string(),
                path: vec!["0xtarget".to_string(), "0xone".to_string()],
                relationship_path: vec![RelationshipStep {
                    from_wallet: "0xtarget".to_string(),
                    to_wallet: "0xone".to_string(),
                    transaction_count: 1,
                    assets_seen: vec!["ETH".to_string()],
                    totals_by_asset: BTreeMap::from([("ETH".to_string(), 1.0)]),
                    latest_timestamp: "2026-03-11T10:00:00Z".to_string(),
                }],
            },
            Finding {
                address: "0xtwo".to_string(),
                hop_distance: 1,
                category: RiskCategory::Sanctioned,
                source: RiskSource::BuiltIn,
                risk_level: RiskLevel::High,
                description: "Known sanctioned wallet".to_string(),
                path: vec!["0xtarget".to_string(), "0xtwo".to_string()],
                relationship_path: vec![RelationshipStep {
                    from_wallet: "0xtarget".to_string(),
                    to_wallet: "0xtwo".to_string(),
                    transaction_count: 1,
                    assets_seen: vec!["ETH".to_string()],
                    totals_by_asset: BTreeMap::from([("ETH".to_string(), 2.0)]),
                    latest_timestamp: "2026-03-11T10:10:00Z".to_string(),
                }],
            },
            Finding {
                address: "0xthree".to_string(),
                hop_distance: 2,
                category: RiskCategory::Mixer,
                source: RiskSource::BuiltIn,
                risk_level: RiskLevel::Low,
                description: "Known mixer wallet".to_string(),
                path: vec![
                    "0xtarget".to_string(),
                    "0xwallet1".to_string(),
                    "0xthree".to_string(),
                ],
                relationship_path: vec![
                    RelationshipStep {
                        from_wallet: "0xtarget".to_string(),
                        to_wallet: "0xwallet1".to_string(),
                        transaction_count: 1,
                        assets_seen: vec!["USDC".to_string()],
                        totals_by_asset: BTreeMap::from([("USDC".to_string(), 500.0)]),
                        latest_timestamp: "2026-03-11T10:05:00Z".to_string(),
                    },
                    RelationshipStep {
                        from_wallet: "0xwallet1".to_string(),
                        to_wallet: "0xthree".to_string(),
                        transaction_count: 1,
                        assets_seen: vec!["DAI".to_string()],
                        totals_by_asset: BTreeMap::from([("DAI".to_string(), 1200.0)]),
                        latest_timestamp: "2026-03-11T10:15:00Z".to_string(),
                    },
                ],
            },
        ]
    }

    #[test]
    fn builds_summary_with_correct_counts_and_highest_risk() {
        let summary = build_summary(&sample_findings());

        assert_eq!(summary.risky_wallets_found, 3);
        assert_eq!(summary.highest_risk_level, Some(RiskLevel::High));
        assert_eq!(summary.direct_exposure_count, 2);
        assert_eq!(summary.indirect_exposure_count, 1);
        assert_eq!(summary.low_risk_count, 1);
        assert_eq!(summary.medium_risk_count, 1);
        assert_eq!(summary.high_risk_count, 1);
    }

    #[test]
    fn builds_summary_with_zero_counts_for_empty_findings() {
        let summary = build_summary(&[]);

        assert_eq!(summary.risky_wallets_found, 0);
        assert_eq!(summary.highest_risk_level, None);
        assert_eq!(summary.direct_exposure_count, 0);
        assert_eq!(summary.indirect_exposure_count, 0);
        assert_eq!(summary.low_risk_count, 0);
        assert_eq!(summary.medium_risk_count, 0);
        assert_eq!(summary.high_risk_count, 0);
    }
}
