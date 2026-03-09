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
