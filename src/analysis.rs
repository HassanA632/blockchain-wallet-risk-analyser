use std::collections::HashMap;

use crate::models::{
    ConnectionSummary, DiscoveredWallet, Finding, RelationshipStep, RiskCategory, RiskEntity,
    RiskLevel,
};

/// Matches discovered wallets against a risk index so traversal results can be
/// converted into analyst-facing findings with deterministic risk metadata.
pub fn build_findings(
    discovered_wallets: &[DiscoveredWallet],
    risk_index: &HashMap<String, RiskEntity>,
) -> Vec<Finding> {
    let mut findings = Vec::new();

    for discovered_wallet in discovered_wallets {
        if let Some(risk_entity) = find_risk_entity(&discovered_wallet.address, risk_index) {
            findings.push(Finding {
                address: discovered_wallet.address.clone(),
                hop_distance: discovered_wallet.hop_distance,
                category: risk_entity.category.clone(),
                source: risk_entity.source.clone(),
                risk_level: determine_risk_level(
                    discovered_wallet.hop_distance,
                    &risk_entity.category,
                ),
                description: risk_entity.description.clone(),
                path: discovered_wallet.path.clone(),
                relationship_path: discovered_wallet.relationship_path.clone(),
                connection_summary: build_connection_summary(discovered_wallet),
            });
        }
    }

    findings
}

/// Looks up the final risk entity for a discovered wallet address from the
/// deduplicated risk index used during analysis.
fn find_risk_entity<'a>(
    address: &str,
    risk_index: &'a HashMap<String, RiskEntity>,
) -> Option<&'a RiskEntity> {
    risk_index.get(address)
}

/// Assigns a risk level using hop distance and risk category so findings stay
/// explainable and follow a consistent severity model.
fn determine_risk_level(hop_distance: u8, category: &RiskCategory) -> RiskLevel {
    match hop_distance {
        1 => match category {
            RiskCategory::Sanctioned => RiskLevel::High,
            RiskCategory::Mixer | RiskCategory::Suspect | RiskCategory::Other => RiskLevel::Medium,
        },
        2 => RiskLevel::Low,
        _ => RiskLevel::Low,
    }
}

/// Builds a connection summary from the final relationship step so each
/// finding exposes the direct interaction evidence for the risky wallet.
fn build_connection_summary(discovered_wallet: &DiscoveredWallet) -> ConnectionSummary {
    let final_step = discovered_wallet
        .relationship_path
        .last()
        .expect("discovered wallets should always include at least one relationship step");

    ConnectionSummary {
        counterparty_wallet: final_step.from_wallet.clone(),
        sent_transaction_count: final_step.received_transaction_count,
        received_transaction_count: final_step.sent_transaction_count,
        sent_totals_by_asset: final_step.received_totals_by_asset.clone(),
        received_totals_by_asset: final_step.sent_totals_by_asset.clone(),
        assets_seen: final_step.assets_seen.clone(),
        latest_timestamp: final_step.latest_timestamp.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, HashMap};

    use crate::models::{
        DiscoveredWallet, RelationshipStep, RiskCategory, RiskEntity, RiskLevel, RiskSource,
    };

    fn sample_risk_index() -> HashMap<String, RiskEntity> {
        HashMap::from([
            (
                "0x4444444444444444444444444444444444444444".to_string(),
                RiskEntity {
                    address: "0x4444444444444444444444444444444444444444".to_string(),
                    category: RiskCategory::Sanctioned,
                    source: RiskSource::BuiltIn,
                    description: "Known sanctioned wallet".to_string(),
                },
            ),
            (
                "0x5555555555555555555555555555555555555555".to_string(),
                RiskEntity {
                    address: "0x5555555555555555555555555555555555555555".to_string(),
                    category: RiskCategory::Mixer,
                    source: RiskSource::BuiltIn,
                    description: "Known mixer wallet".to_string(),
                },
            ),
            (
                "0x3333333333333333333333333333333333333333".to_string(),
                RiskEntity {
                    address: "0x3333333333333333333333333333333333333333".to_string(),
                    category: RiskCategory::Other,
                    source: RiskSource::Custom,
                    description: "Analyst watchlist entry".to_string(),
                },
            ),
        ])
    }

    #[test]
    fn builds_findings_for_matching_wallets_only() {
        let discovered_wallets = vec![
            DiscoveredWallet {
                address: "0x4444444444444444444444444444444444444444".to_string(),
                hop_distance: 2,
                path: vec![
                    "0x1111111111111111111111111111111111111111".to_string(),
                    "0x2222222222222222222222222222222222222222".to_string(),
                    "0x4444444444444444444444444444444444444444".to_string(),
                ],
                relationship_path: vec![
                    RelationshipStep {
                        from_wallet: "0x1111111111111111111111111111111111111111".to_string(),
                        to_wallet: "0x2222222222222222222222222222222222222222".to_string(),
                        transaction_count: 1,
                        assets_seen: vec!["ETH".to_string()],
                        latest_timestamp: "2026-03-11T10:00:00Z".to_string(),
                        sent_transaction_count: 1,
                        received_transaction_count: 0,
                        sent_totals_by_asset: BTreeMap::from([("ETH".to_string(), 1.25)]),
                        received_totals_by_asset: BTreeMap::new(),
                    },
                    RelationshipStep {
                        from_wallet: "0x2222222222222222222222222222222222222222".to_string(),
                        to_wallet: "0x4444444444444444444444444444444444444444".to_string(),
                        transaction_count: 1,
                        assets_seen: vec!["ETH".to_string()],
                        latest_timestamp: "2026-03-11T10:10:00Z".to_string(),
                        sent_transaction_count: 1,
                        received_transaction_count: 0,
                        sent_totals_by_asset: BTreeMap::from([("ETH".to_string(), 0.75)]),
                        received_totals_by_asset: BTreeMap::new(),
                    },
                ],
            },
            DiscoveredWallet {
                address: "0x6666666666666666666666666666666666666666".to_string(),
                hop_distance: 1,
                path: vec![
                    "0x1111111111111111111111111111111111111111".to_string(),
                    "0x6666666666666666666666666666666666666666".to_string(),
                ],
                relationship_path: vec![RelationshipStep {
                    from_wallet: "0x1111111111111111111111111111111111111111".to_string(),
                    to_wallet: "0x6666666666666666666666666666666666666666".to_string(),
                    transaction_count: 1,
                    assets_seen: vec!["ETH".to_string()],
                    latest_timestamp: "2026-03-11T11:00:00Z".to_string(),
                    sent_transaction_count: 1,
                    received_transaction_count: 0,
                    sent_totals_by_asset: BTreeMap::from([("ETH".to_string(), 0.25)]),
                    received_totals_by_asset: BTreeMap::new(),
                }],
            },
        ];

        let findings = build_findings(&discovered_wallets, &sample_risk_index());

        assert_eq!(findings.len(), 1);
        assert_eq!(
            findings[0].address,
            "0x4444444444444444444444444444444444444444"
        );
        assert_eq!(findings[0].source, RiskSource::BuiltIn);
        assert_eq!(findings[0].relationship_path.len(), 2);
        assert_eq!(
            findings[0].connection_summary.counterparty_wallet,
            "0x2222222222222222222222222222222222222222"
        );
        assert_eq!(findings[0].connection_summary.sent_transaction_count, 0);
        assert_eq!(findings[0].connection_summary.received_transaction_count, 1);
        assert_eq!(
            findings[0]
                .connection_summary
                .received_totals_by_asset
                .get("ETH"),
            Some(&0.75)
        );
    }

    #[test]
    fn assigns_high_risk_to_direct_sanctioned_wallets() {
        let discovered_wallets = vec![DiscoveredWallet {
            address: "0x4444444444444444444444444444444444444444".to_string(),
            hop_distance: 1,
            path: vec![
                "0x1111111111111111111111111111111111111111".to_string(),
                "0x4444444444444444444444444444444444444444".to_string(),
            ],
            relationship_path: vec![RelationshipStep {
                from_wallet: "0x1111111111111111111111111111111111111111".to_string(),
                to_wallet: "0x4444444444444444444444444444444444444444".to_string(),
                transaction_count: 1,
                assets_seen: vec!["ETH".to_string()],
                latest_timestamp: "2026-03-11T12:00:00Z".to_string(),
                sent_transaction_count: 1,
                received_transaction_count: 0,
                sent_totals_by_asset: BTreeMap::from([("ETH".to_string(), 2.0)]),
                received_totals_by_asset: BTreeMap::new(),
            }],
        }];

        let findings = build_findings(&discovered_wallets, &sample_risk_index());

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].risk_level, RiskLevel::High);
        assert_eq!(findings[0].source, RiskSource::BuiltIn);
        assert_eq!(
            findings[0].connection_summary.counterparty_wallet,
            "0x1111111111111111111111111111111111111111"
        );
        assert_eq!(findings[0].connection_summary.sent_transaction_count, 0);
        assert_eq!(findings[0].connection_summary.received_transaction_count, 1);
    }

    #[test]
    fn assigns_medium_risk_to_direct_other_wallets() {
        let discovered_wallets = vec![DiscoveredWallet {
            address: "0x3333333333333333333333333333333333333333".to_string(),
            hop_distance: 1,
            path: vec![
                "0x1111111111111111111111111111111111111111".to_string(),
                "0x3333333333333333333333333333333333333333".to_string(),
            ],
            relationship_path: vec![RelationshipStep {
                from_wallet: "0x1111111111111111111111111111111111111111".to_string(),
                to_wallet: "0x3333333333333333333333333333333333333333".to_string(),
                transaction_count: 1,
                assets_seen: vec!["USDC".to_string()],
                latest_timestamp: "2026-03-11T10:05:00Z".to_string(),
                sent_transaction_count: 1,
                received_transaction_count: 0,
                sent_totals_by_asset: BTreeMap::from([("USDC".to_string(), 500.0)]),
                received_totals_by_asset: BTreeMap::new(),
            }],
        }];

        let findings = build_findings(&discovered_wallets, &sample_risk_index());

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].risk_level, RiskLevel::Medium);
        assert_eq!(findings[0].source, RiskSource::Custom);
        assert_eq!(findings[0].category, RiskCategory::Other);
        assert_eq!(
            findings[0]
                .connection_summary
                .received_totals_by_asset
                .get("USDC"),
            Some(&500.0)
        );
    }

    #[test]
    fn assigns_low_risk_to_two_hop_matches() {
        let discovered_wallets = vec![DiscoveredWallet {
            address: "0x5555555555555555555555555555555555555555".to_string(),
            hop_distance: 2,
            path: vec![
                "0x1111111111111111111111111111111111111111".to_string(),
                "0x3333333333333333333333333333333333333333".to_string(),
                "0x5555555555555555555555555555555555555555".to_string(),
            ],
            relationship_path: vec![
                RelationshipStep {
                    from_wallet: "0x1111111111111111111111111111111111111111".to_string(),
                    to_wallet: "0x3333333333333333333333333333333333333333".to_string(),
                    transaction_count: 1,
                    assets_seen: vec!["USDC".to_string()],
                    latest_timestamp: "2026-03-11T10:05:00Z".to_string(),
                    sent_transaction_count: 1,
                    received_transaction_count: 0,
                    sent_totals_by_asset: BTreeMap::from([("USDC".to_string(), 500.0)]),
                    received_totals_by_asset: BTreeMap::new(),
                },
                RelationshipStep {
                    from_wallet: "0x3333333333333333333333333333333333333333".to_string(),
                    to_wallet: "0x5555555555555555555555555555555555555555".to_string(),
                    transaction_count: 1,
                    assets_seen: vec!["DAI".to_string()],
                    latest_timestamp: "2026-03-11T10:15:00Z".to_string(),
                    sent_transaction_count: 1,
                    received_transaction_count: 0,
                    sent_totals_by_asset: BTreeMap::from([("DAI".to_string(), 1200.0)]),
                    received_totals_by_asset: BTreeMap::new(),
                },
            ],
        }];

        let findings = build_findings(&discovered_wallets, &sample_risk_index());

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].risk_level, RiskLevel::Low);
        assert_eq!(findings[0].source, RiskSource::BuiltIn);
        assert_eq!(
            findings[0].connection_summary.counterparty_wallet,
            "0x3333333333333333333333333333333333333333"
        );
        assert_eq!(findings[0].connection_summary.sent_transaction_count, 0);
        assert_eq!(findings[0].connection_summary.received_transaction_count, 1);
        assert_eq!(
            findings[0]
                .connection_summary
                .received_totals_by_asset
                .get("DAI"),
            Some(&1200.0)
        );
    }
}
