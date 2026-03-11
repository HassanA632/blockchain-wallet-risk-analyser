use std::collections::HashMap;

use crate::models::{DiscoveredWallet, Finding, RiskCategory, RiskEntity, RiskLevel};

/// Matches discovered wallets against known risk wallets so graph traversal
/// results can be converted into risk findings.
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
                risk_level: determine_risk_level(
                    discovered_wallet.hop_distance,
                    &risk_entity.category,
                ),
                description: risk_entity.description.clone(),
                path: discovered_wallet.path.clone(),
                source: risk_entity.source.clone(),
                relationship_path: discovered_wallet.relationship_path.clone(),
            });
        }
    }

    findings
}

/// Finds the risk entity record for a discovered wallet address so matching can
/// enrich traversal results with risk data.
fn find_risk_entity<'a>(
    address: &str,
    risk_index: &'a HashMap<String, RiskEntity>,
) -> Option<&'a RiskEntity> {
    risk_index.get(address)
}

/// Assigns risk level using hop distance and risk category so findings stay
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
                        totals_by_asset: BTreeMap::from([("ETH".to_string(), 1.25)]),
                        latest_timestamp: "2026-03-11T10:00:00Z".to_string(),
                    },
                    RelationshipStep {
                        from_wallet: "0x2222222222222222222222222222222222222222".to_string(),
                        to_wallet: "0x4444444444444444444444444444444444444444".to_string(),
                        transaction_count: 1,
                        assets_seen: vec!["ETH".to_string()],
                        totals_by_asset: BTreeMap::from([("ETH".to_string(), 0.75)]),
                        latest_timestamp: "2026-03-11T10:10:00Z".to_string(),
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
                    totals_by_asset: BTreeMap::from([("ETH".to_string(), 0.25)]),
                    latest_timestamp: "2026-03-11T11:00:00Z".to_string(),
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
        assert_eq!(
            findings[0].path,
            vec![
                "0x1111111111111111111111111111111111111111".to_string(),
                "0x2222222222222222222222222222222222222222".to_string(),
                "0x4444444444444444444444444444444444444444".to_string(),
            ]
        );
        assert_eq!(findings[0].relationship_path.len(), 2);
        assert_eq!(
            findings[0].relationship_path[0].from_wallet,
            "0x1111111111111111111111111111111111111111"
        );
        assert_eq!(
            findings[0].relationship_path[1].to_wallet,
            "0x4444444444444444444444444444444444444444"
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
                totals_by_asset: BTreeMap::from([("ETH".to_string(), 2.0)]),
                latest_timestamp: "2026-03-11T12:00:00Z".to_string(),
            }],
        }];

        let findings = build_findings(&discovered_wallets, &sample_risk_index());

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].risk_level, RiskLevel::High);
        assert_eq!(findings[0].source, RiskSource::BuiltIn);
        assert_eq!(findings[0].relationship_path.len(), 1);
        assert_eq!(
            findings[0].relationship_path[0].from_wallet,
            "0x1111111111111111111111111111111111111111"
        );
        assert_eq!(
            findings[0].relationship_path[0].to_wallet,
            "0x4444444444444444444444444444444444444444"
        );
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
                totals_by_asset: BTreeMap::from([("USDC".to_string(), 500.0)]),
                latest_timestamp: "2026-03-11T10:05:00Z".to_string(),
            }],
        }];

        let findings = build_findings(&discovered_wallets, &sample_risk_index());

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].risk_level, RiskLevel::Medium);
        assert_eq!(findings[0].source, RiskSource::Custom);
        assert_eq!(findings[0].category, RiskCategory::Other);
        assert_eq!(findings[0].relationship_path.len(), 1);
        assert_eq!(
            findings[0].relationship_path[0].totals_by_asset.get("USDC"),
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
                    totals_by_asset: BTreeMap::from([("USDC".to_string(), 500.0)]),
                    latest_timestamp: "2026-03-11T10:05:00Z".to_string(),
                },
                RelationshipStep {
                    from_wallet: "0x3333333333333333333333333333333333333333".to_string(),
                    to_wallet: "0x5555555555555555555555555555555555555555".to_string(),
                    transaction_count: 1,
                    assets_seen: vec!["DAI".to_string()],
                    totals_by_asset: BTreeMap::from([("DAI".to_string(), 1200.0)]),
                    latest_timestamp: "2026-03-11T10:15:00Z".to_string(),
                },
            ],
        }];

        let findings = build_findings(&discovered_wallets, &sample_risk_index());

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].risk_level, RiskLevel::Low);
        assert_eq!(findings[0].source, RiskSource::BuiltIn);
        assert_eq!(findings[0].relationship_path.len(), 2);
        assert_eq!(
            findings[0].relationship_path[0].to_wallet,
            "0x3333333333333333333333333333333333333333"
        );
        assert_eq!(
            findings[0].relationship_path[1].to_wallet,
            "0x5555555555555555555555555555555555555555"
        );
    }
}
