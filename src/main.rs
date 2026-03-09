mod models;
mod report;

use models::{Chain, Finding, RiskCategory, RiskLevel};
use report::{ReportSummary, RiskReport};

fn main() {
    let report = RiskReport {
        target_wallet: "0xTARGETWALLET1".to_string(),
        chain: Chain::Ethereum,
        hop_depth: 2,
        summary: ReportSummary {
            risky_wallets_found: 2,
            highest_risk_level: Some(RiskLevel::High),
        },
        findings: vec![
            Finding {
                address: "0XWALLET_2".to_string(),
                hop_distance: 1,
                category: RiskCategory::Sanctioned,
                risk_level: RiskLevel::High,
                description: "Directly identified as a sanctioned wallet".to_string(),
            },
            Finding {
                address: "0XWALLET_3".to_string(),
                hop_distance: 2,
                category: RiskCategory::Mixer,
                risk_level: RiskLevel::Low,
                description: "Indirect 2-hop exposure to a mixer wallet".to_string(),
            },
        ],
    };

    let json = serde_json::to_string_pretty(&report).expect("report serialization failed");
    println!("{json}");
}
