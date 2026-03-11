use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::models::{TransactionEdge, WalletRelationship};

/// Aggregates raw transfer edges into wallet-level relationships so repeated
/// interactions between the same wallet pair are merged for exposure analysis.
pub fn build_wallet_relationships(edges: &[TransactionEdge]) -> Vec<WalletRelationship> {
    let mut grouped: HashMap<(String, String), RelationshipAccumulator> = HashMap::new();

    for edge in edges {
        let (wallet_a, wallet_b) = canonical_wallet_pair(&edge.from_address, &edge.to_address);

        let relationship = grouped
            .entry((wallet_a, wallet_b))
            .or_insert_with(RelationshipAccumulator::default);

        relationship.transaction_count += 1;
        relationship.assets_seen.insert(edge.asset.clone());

        let amount = edge
            .amount
            .parse::<f64>()
            .expect("transaction amounts should parse as f64 during relationship aggregation");

        *relationship
            .totals_by_asset
            .entry(edge.asset.clone())
            .or_insert(0.0) += amount;

        if edge.timestamp > relationship.latest_timestamp {
            relationship.latest_timestamp = edge.timestamp.clone();
        }
    }

    let mut relationships: Vec<WalletRelationship> = grouped
        .into_iter()
        .map(|((wallet_a, wallet_b), accumulator)| WalletRelationship {
            wallet_a,
            wallet_b,
            transaction_count: accumulator.transaction_count,
            assets_seen: accumulator.assets_seen.into_iter().collect(),
            totals_by_asset: accumulator.totals_by_asset,
            latest_timestamp: accumulator.latest_timestamp,
        })
        .collect();

    relationships.sort_by(|left, right| {
        left.wallet_a
            .cmp(&right.wallet_a)
            .then(left.wallet_b.cmp(&right.wallet_b))
    });

    relationships
}

/// Normalizes a wallet pair into a deterministic order so both directions of
/// the same relationship are grouped into one traversal-friendly connection.
fn canonical_wallet_pair(left: &str, right: &str) -> (String, String) {
    if left <= right {
        (left.to_string(), right.to_string())
    } else {
        (right.to_string(), left.to_string())
    }
}

#[derive(Debug, Default)]
struct RelationshipAccumulator {
    transaction_count: usize,
    assets_seen: BTreeSet<String>,
    totals_by_asset: BTreeMap<String, f64>,
    latest_timestamp: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TransactionEdge;

    fn sample_edges() -> Vec<TransactionEdge> {
        vec![
            TransactionEdge {
                from_address: "0x1111111111111111111111111111111111111111".to_string(),
                to_address: "0x2222222222222222222222222222222222222222".to_string(),
                tx_hash: "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                    .to_string(),
                asset: "ETH".to_string(),
                amount: "1.25".to_string(),
                timestamp: "2026-03-11T10:00:00Z".to_string(),
            },
            TransactionEdge {
                from_address: "0x2222222222222222222222222222222222222222".to_string(),
                to_address: "0x1111111111111111111111111111111111111111".to_string(),
                tx_hash: "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
                    .to_string(),
                asset: "ETH".to_string(),
                amount: "0.75".to_string(),
                timestamp: "2026-03-11T10:05:00Z".to_string(),
            },
            TransactionEdge {
                from_address: "0x1111111111111111111111111111111111111111".to_string(),
                to_address: "0x2222222222222222222222222222222222222222".to_string(),
                tx_hash: "0xcccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
                    .to_string(),
                asset: "USDC".to_string(),
                amount: "500.00".to_string(),
                timestamp: "2026-03-11T10:10:00Z".to_string(),
            },
            TransactionEdge {
                from_address: "0x3333333333333333333333333333333333333333".to_string(),
                to_address: "0x4444444444444444444444444444444444444444".to_string(),
                tx_hash: "0xdddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
                    .to_string(),
                asset: "DAI".to_string(),
                amount: "1200.00".to_string(),
                timestamp: "2026-03-11T10:15:00Z".to_string(),
            },
        ]
    }

    #[test]
    fn groups_multiple_transfers_into_one_wallet_relationship() {
        let relationships = build_wallet_relationships(&sample_edges());

        assert_eq!(relationships.len(), 2);

        let relationship = relationships
            .iter()
            .find(|relationship| {
                relationship.wallet_a == "0x1111111111111111111111111111111111111111"
                    && relationship.wallet_b == "0x2222222222222222222222222222222222222222"
            })
            .expect("relationship between wallets 0x111... and 0x222... should exist");

        assert_eq!(relationship.transaction_count, 3);
        assert_eq!(
            relationship.assets_seen,
            vec!["ETH".to_string(), "USDC".to_string()]
        );
        assert_eq!(relationship.totals_by_asset.get("ETH"), Some(&2.0));
        assert_eq!(relationship.totals_by_asset.get("USDC"), Some(&500.0));
        assert_eq!(relationship.latest_timestamp, "2026-03-11T10:10:00Z");
    }

    #[test]
    fn stores_wallet_pairs_in_deterministic_order() {
        let relationships = build_wallet_relationships(&sample_edges());

        let relationship = relationships
            .iter()
            .find(|relationship| {
                relationship.wallet_a == "0x1111111111111111111111111111111111111111"
                    && relationship.wallet_b == "0x2222222222222222222222222222222222222222"
            })
            .expect("relationship between wallets 0x111... and 0x222... should exist");

        assert_eq!(
            relationship.wallet_a,
            "0x1111111111111111111111111111111111111111"
        );
        assert_eq!(
            relationship.wallet_b,
            "0x2222222222222222222222222222222222222222"
        );
    }
}
