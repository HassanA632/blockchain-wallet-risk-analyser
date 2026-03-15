use std::collections::{HashMap, HashSet};

use crate::models::{DiscoveredWallet, RelationshipStep, ServiceWallet, WalletRelationship};

/// Discovers wallets connected to a target address within the requested hop
/// depth so later analysis can check them against risk intelligence.
pub fn discover_wallets(
    target_wallet: &str,
    hop_depth: u8,
    relationships: &[WalletRelationship],
    service_wallet_index: &HashMap<String, ServiceWallet>,
) -> Vec<DiscoveredWallet> {
    let mut discovered = Vec::new();
    let mut seen_addresses = HashSet::new();
    let mut first_hop_wallets: Vec<(String, RelationshipStep)> = Vec::new();

    for relationship in relationships {
        if let Some(neighbour) = connected_wallet(target_wallet, relationship) {
            if seen_addresses.insert(neighbour.to_string()) {
                let first_step = relationship_step(target_wallet, neighbour, relationship);

                let wallet = DiscoveredWallet {
                    address: neighbour.to_string(),
                    hop_distance: 1,
                    path: vec![target_wallet.to_string(), neighbour.to_string()],
                    relationship_path: vec![first_step.clone()],
                };

                if !is_service_wallet(neighbour, service_wallet_index) {
                    first_hop_wallets.push((neighbour.to_string(), first_step));
                }

                discovered.push(wallet);
            }
        }
    }

    if hop_depth < 2 {
        return discovered;
    }

    for (first_hop_wallet, first_step) in first_hop_wallets {
        for relationship in relationships {
            if let Some(neighbour) = connected_wallet(&first_hop_wallet, relationship) {
                if neighbour == target_wallet {
                    continue;
                }

                if seen_addresses.insert(neighbour.to_string()) {
                    discovered.push(DiscoveredWallet {
                        address: neighbour.to_string(),
                        hop_distance: 2,
                        path: vec![
                            target_wallet.to_string(),
                            first_hop_wallet.clone(),
                            neighbour.to_string(),
                        ],
                        relationship_path: vec![
                            first_step.clone(),
                            relationship_step(&first_hop_wallet, neighbour, relationship),
                        ],
                    });
                }
            }
        }
    }

    discovered
}

/// Returns the wallet on the opposite side of a wallet relationship when the
/// given address is part of that relationship.
fn connected_wallet<'a>(address: &str, relationship: &'a WalletRelationship) -> Option<&'a str> {
    if relationship.wallet_a == address {
        Some(relationship.wallet_b.as_str())
    } else if relationship.wallet_b == address {
        Some(relationship.wallet_a.as_str())
    } else {
        None
    }
}

/// Converts a wallet relationship into a path step oriented to the current
/// traversal direction so reports show the hop sequence.
fn relationship_step(
    from_wallet: &str,
    to_wallet: &str,
    relationship: &WalletRelationship,
) -> RelationshipStep {
    let (
        sent_transaction_count,
        received_transaction_count,
        sent_totals_by_asset,
        received_totals_by_asset,
    ) = if relationship.wallet_a == from_wallet && relationship.wallet_b == to_wallet {
        (
            relationship.a_to_b_transaction_count,
            relationship.b_to_a_transaction_count,
            relationship.a_to_b_totals_by_asset.clone(),
            relationship.b_to_a_totals_by_asset.clone(),
        )
    } else {
        (
            relationship.b_to_a_transaction_count,
            relationship.a_to_b_transaction_count,
            relationship.b_to_a_totals_by_asset.clone(),
            relationship.a_to_b_totals_by_asset.clone(),
        )
    };

    RelationshipStep {
        from_wallet: from_wallet.to_string(),
        to_wallet: to_wallet.to_string(),
        transaction_count: relationship.transaction_count,
        assets_seen: relationship.assets_seen.clone(),
        latest_timestamp: relationship.latest_timestamp.clone(),
        sent_transaction_count,
        received_transaction_count,
        sent_totals_by_asset,
        received_totals_by_asset,
    }
}

/// Checks whether a wallet is a known service wallet so traversal can stop
/// expansion beyond this address.
fn is_service_wallet(wallet: &str, service_wallet_index: &HashMap<String, ServiceWallet>) -> bool {
    service_wallet_index.contains_key(wallet)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ServiceType, TransactionEdge};
    use crate::relationships::build_wallet_relationships;
    use crate::service_wallets::build_service_wallet_index;

    fn empty_service_wallet_index() -> HashMap<String, ServiceWallet> {
        HashMap::new()
    }

    fn sample_relationships() -> Vec<WalletRelationship> {
        let edges = vec![
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
                from_address: "0x1111111111111111111111111111111111111111".to_string(),
                to_address: "0x3333333333333333333333333333333333333333".to_string(),
                tx_hash: "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
                    .to_string(),
                asset: "USDC".to_string(),
                amount: "500.00".to_string(),
                timestamp: "2026-03-11T10:05:00Z".to_string(),
            },
            TransactionEdge {
                from_address: "0x2222222222222222222222222222222222222222".to_string(),
                to_address: "0x4444444444444444444444444444444444444444".to_string(),
                tx_hash: "0xcccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
                    .to_string(),
                asset: "ETH".to_string(),
                amount: "0.75".to_string(),
                timestamp: "2026-03-11T10:10:00Z".to_string(),
            },
            TransactionEdge {
                from_address: "0x3333333333333333333333333333333333333333".to_string(),
                to_address: "0x5555555555555555555555555555555555555555".to_string(),
                tx_hash: "0xdddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
                    .to_string(),
                asset: "DAI".to_string(),
                amount: "1200.00".to_string(),
                timestamp: "2026-03-11T10:15:00Z".to_string(),
            },
        ];

        build_wallet_relationships(&edges)
    }

    #[test]
    fn discovers_direct_wallets_at_one_hop() {
        let discovered = discover_wallets(
            "0x1111111111111111111111111111111111111111",
            1,
            &sample_relationships(),
            &empty_service_wallet_index(),
        );

        assert_eq!(discovered.len(), 2);
        assert!(
            discovered
                .iter()
                .any(|wallet| { wallet.address == "0x2222222222222222222222222222222222222222" })
        );
        assert!(
            discovered
                .iter()
                .any(|wallet| { wallet.address == "0x3333333333333333333333333333333333333333" })
        );
        assert!(discovered.iter().all(|wallet| wallet.hop_distance == 1));
    }

    #[test]
    fn discovers_wallets_up_to_two_hops() {
        let discovered = discover_wallets(
            "0x1111111111111111111111111111111111111111",
            2,
            &sample_relationships(),
            &empty_service_wallet_index(),
        );

        assert_eq!(discovered.len(), 4);
        assert!(
            discovered
                .iter()
                .any(|wallet| { wallet.address == "0x4444444444444444444444444444444444444444" })
        );
        assert!(
            discovered
                .iter()
                .any(|wallet| { wallet.address == "0x5555555555555555555555555555555555555555" })
        );
    }

    #[test]
    fn does_not_rediscover_target_wallet() {
        let edges = vec![
            TransactionEdge {
                from_address: "0x1111111111111111111111111111111111111111".to_string(),
                to_address: "0x2222222222222222222222222222222222222222".to_string(),
                tx_hash: "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                    .to_string(),
                asset: "ETH".to_string(),
                amount: "1.00".to_string(),
                timestamp: "2026-03-11T10:00:00Z".to_string(),
            },
            TransactionEdge {
                from_address: "0x2222222222222222222222222222222222222222".to_string(),
                to_address: "0x1111111111111111111111111111111111111111".to_string(),
                tx_hash: "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
                    .to_string(),
                asset: "ETH".to_string(),
                amount: "0.50".to_string(),
                timestamp: "2026-03-11T10:05:00Z".to_string(),
            },
        ];

        let relationships = build_wallet_relationships(&edges);

        let discovered = discover_wallets(
            "0x1111111111111111111111111111111111111111",
            2,
            &relationships,
            &empty_service_wallet_index(),
        );

        assert!(
            discovered
                .iter()
                .all(|wallet| wallet.address != "0x1111111111111111111111111111111111111111")
        );
    }

    #[test]
    fn includes_service_wallet_but_does_not_expand_beyond_it() {
        let edges = vec![
            TransactionEdge {
                from_address: "0x1111111111111111111111111111111111111111".to_string(),
                to_address: "0x2222222222222222222222222222222222222222".to_string(),
                tx_hash: "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                    .to_string(),
                asset: "ETH".to_string(),
                amount: "1.00".to_string(),
                timestamp: "2026-03-11T10:00:00Z".to_string(),
            },
            TransactionEdge {
                from_address: "0x2222222222222222222222222222222222222222".to_string(),
                to_address: "0x3333333333333333333333333333333333333333".to_string(),
                tx_hash: "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
                    .to_string(),
                asset: "ETH".to_string(),
                amount: "2.00".to_string(),
                timestamp: "2026-03-11T10:05:00Z".to_string(),
            },
        ];

        let relationships = build_wallet_relationships(&edges);

        let service_wallet_index = build_service_wallet_index(vec![ServiceWallet {
            address: "0x2222222222222222222222222222222222222222".to_string(),
            label: "Sample Exchange Hot Wallet".to_string(),
            service_type: ServiceType::Exchange,
        }]);

        let discovered = discover_wallets(
            "0x1111111111111111111111111111111111111111",
            2,
            &relationships,
            &service_wallet_index,
        );

        assert!(discovered.iter().any(|wallet| {
            wallet.address == "0x2222222222222222222222222222222222222222"
                && wallet.hop_distance == 1
        }));

        assert!(
            !discovered
                .iter()
                .any(|wallet| { wallet.address == "0x3333333333333333333333333333333333333333" })
        );
    }
}
