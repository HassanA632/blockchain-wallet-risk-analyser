use std::collections::HashMap;

use crate::models::RiskEntity;

/// Builds a deduplicated risk index keyed by wallet address so matching stays
/// fast and analyst supplied entries override built-in intelligence.
pub fn build_risk_index(
    built_in: Vec<RiskEntity>,
    custom: Vec<RiskEntity>,
) -> HashMap<String, RiskEntity> {
    let mut index = HashMap::new();

    for entity in built_in {
        index.insert(entity.address.clone(), entity);
    }

    for entity in custom {
        index.insert(entity.address.clone(), entity);
    }

    index
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{RiskCategory, RiskSource};

    #[test]
    fn custom_entities_override_built_in_entries_with_same_address() {
        let built_in = vec![RiskEntity {
            address: "0xabc".to_string(),
            category: RiskCategory::Mixer,
            source: RiskSource::BuiltIn,
            description: "Known mixer wallet".to_string(),
        }];

        let custom = vec![RiskEntity {
            address: "0xabc".to_string(),
            category: RiskCategory::Other,
            source: RiskSource::Custom,
            description: "Analyst-linked address".to_string(),
        }];

        let index = build_risk_index(built_in, custom);

        assert_eq!(index.len(), 1);

        let entity = index
            .get("0xabc")
            .expect("0xabc should exist in the risk index");

        assert_eq!(entity.category, RiskCategory::Other);
        assert_eq!(entity.source, RiskSource::Custom);
        assert_eq!(entity.description, "Analyst-linked address");
    }

    #[test]
    fn keeps_distinct_addresses_from_both_sources() {
        let built_in = vec![RiskEntity {
            address: "0xone".to_string(),
            category: RiskCategory::Sanctioned,
            source: RiskSource::BuiltIn,
            description: "Known sanctioned wallet".to_string(),
        }];

        let custom = vec![RiskEntity {
            address: "0xtwo".to_string(),
            category: RiskCategory::Other,
            source: RiskSource::Custom,
            description: "Analyst watchlist entry".to_string(),
        }];

        let index = build_risk_index(built_in, custom);

        assert_eq!(index.len(), 2);
        assert!(index.contains_key("0xone"));
        assert!(index.contains_key("0xtwo"));
    }
}
