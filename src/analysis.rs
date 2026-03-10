use crate::models::{DiscoveredWallet, Finding, RiskCategory, RiskEntity, RiskLevel};

/// Matches discovered wallets against known risk entities so graph traversal
/// results can be converted into analyst facing risk findings.
pub fn build_findings(
    discovered_wallets: &[DiscoveredWallet],
    risk_entities: &[RiskEntity],
) -> Vec<Finding> {
    let mut findings = Vec::new();

    for discovered_wallet in discovered_wallets {
        if let Some(risk_entity) = find_risk_entity(&discovered_wallet.address, risk_entities) {
            findings.push(Finding {
                address: discovered_wallet.address.clone(),
                hop_distance: discovered_wallet.hop_distance,
                category: risk_entity.category.clone(),
                risk_level: determine_risk_level(
                    discovered_wallet.hop_distance,
                    &risk_entity.category,
                ),
                description: risk_entity.description.clone(),
            });
        }
    }

    findings
}

/// Finds the risk entity record for a discovered wallet address so matching can
/// enrich traversal results with risk metadata.
fn find_risk_entity<'a>(address: &str, risk_entities: &'a [RiskEntity]) -> Option<&'a RiskEntity> {
    risk_entities
        .iter()
        .find(|entity| entity.address == address)
}

/// Assigns a risk level using hop distance and risk category so findings stay
/// explainable and follow a consistent severity model.
fn determine_risk_level(hop_distance: u8, category: &RiskCategory) -> RiskLevel {
    match hop_distance {
        1 => match category {
            RiskCategory::Sanctioned => RiskLevel::High,
            RiskCategory::Mixer | RiskCategory::Suspect | RiskCategory::Custom => RiskLevel::Medium,
        },
        2 => RiskLevel::Low,
        _ => RiskLevel::Low,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{DiscoveredWallet, RiskCategory, RiskEntity};

    fn sample_risk_entities() -> Vec<RiskEntity> {
        vec![
            RiskEntity {
                address: "0xrisky1".to_string(),
                category: RiskCategory::Sanctioned,
                description: "Known sanctioned wallet".to_string(),
            },
            RiskEntity {
                address: "0xrisky2".to_string(),
                category: RiskCategory::Mixer,
                description: "Known mixer wallet".to_string(),
            },
            RiskEntity {
                address: "0xwatch1".to_string(),
                category: RiskCategory::Custom,
                description: "Analyst watchlist entry".to_string(),
            },
        ]
    }

    #[test]
    fn builds_findings_for_matching_wallets_only() {
        let discovered_wallets = vec![
            DiscoveredWallet {
                address: "0xrisky1".to_string(),
                hop_distance: 2,
                path: vec![
                    "0xtarget".to_string(),
                    "0xwallet1".to_string(),
                    "0xrisky1".to_string(),
                ],
            },
            DiscoveredWallet {
                address: "0xclean1".to_string(),
                hop_distance: 1,
                path: vec!["0xtarget".to_string(), "0xclean1".to_string()],
            },
        ];

        let findings = build_findings(&discovered_wallets, &sample_risk_entities());

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].address, "0xrisky1");
    }

    #[test]
    fn assigns_high_risk_to_direct_sanctioned_wallets() {
        let discovered_wallets = vec![DiscoveredWallet {
            address: "0xrisky1".to_string(),
            hop_distance: 1,
            path: vec!["0xtarget".to_string(), "0xrisky1".to_string()],
        }];

        let findings = build_findings(&discovered_wallets, &sample_risk_entities());

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].risk_level, RiskLevel::High);
    }

    #[test]
    fn assigns_medium_risk_to_direct_custom_wallets() {
        let discovered_wallets = vec![DiscoveredWallet {
            address: "0xwatch1".to_string(),
            hop_distance: 1,
            path: vec!["0xtarget".to_string(), "0xwatch1".to_string()],
        }];

        let findings = build_findings(&discovered_wallets, &sample_risk_entities());

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].risk_level, RiskLevel::Medium);
    }

    #[test]
    fn assigns_low_risk_to_two_hop_matches() {
        let discovered_wallets = vec![DiscoveredWallet {
            address: "0xrisky2".to_string(),
            hop_distance: 2,
            path: vec![
                "0xtarget".to_string(),
                "0xwallet2".to_string(),
                "0xrisky2".to_string(),
            ],
        }];

        let findings = build_findings(&discovered_wallets, &sample_risk_entities());

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].risk_level, RiskLevel::Low);
    }
}
