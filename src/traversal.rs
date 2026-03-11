use std::collections::HashSet;

use crate::models::{DiscoveredWallet, RelationshipStep, WalletRelationship};

/// Discovers wallets connected to a target address within the requested hop
/// depth so later analysis can check them against risk intelligence.
pub fn discover_wallets(
    target_wallet: &str,
    hop_depth: u8,
    relationships: &[WalletRelationship],
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

                first_hop_wallets.push((neighbour.to_string(), first_step));
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
/// given address is part of that relationship allowing traversal to treat it
/// as a wallet-to-wallet connection.
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
    let (transaction_count, totals_by_asset) =
        if relationship.wallet_a == from_wallet && relationship.wallet_b == to_wallet {
            (
                relationship.a_to_b_transaction_count,
                relationship.a_to_b_totals_by_asset.clone(),
            )
        } else {
            (
                relationship.b_to_a_transaction_count,
                relationship.b_to_a_totals_by_asset.clone(),
            )
        };

    RelationshipStep {
        from_wallet: from_wallet.to_string(),
        to_wallet: to_wallet.to_string(),
        transaction_count,
        assets_seen: relationship.assets_seen.clone(),
        totals_by_asset,
        latest_timestamp: relationship.latest_timestamp.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TransactionEdge;
    use crate::relationships::build_wallet_relationships;

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

        let wallet1 = discovered
            .iter()
            .find(|wallet| wallet.address == "0x2222222222222222222222222222222222222222")
            .expect("0x2222222222222222222222222222222222222222 should be discovered");

        assert_eq!(
            wallet1.path,
            vec![
                "0x1111111111111111111111111111111111111111".to_string(),
                "0x2222222222222222222222222222222222222222".to_string()
            ]
        );
        assert_eq!(wallet1.relationship_path.len(), 1);

        assert_eq!(
            wallet1.relationship_path[0].from_wallet,
            "0x1111111111111111111111111111111111111111"
        );
        assert_eq!(
            wallet1.relationship_path[0].to_wallet,
            "0x2222222222222222222222222222222222222222"
        );
        assert_eq!(wallet1.relationship_path[0].transaction_count, 1);

        let wallet2 = discovered
            .iter()
            .find(|wallet| wallet.address == "0x3333333333333333333333333333333333333333")
            .expect("0x3333333333333333333333333333333333333333 should be discovered");

        assert_eq!(
            wallet2.path,
            vec![
                "0x1111111111111111111111111111111111111111".to_string(),
                "0x3333333333333333333333333333333333333333".to_string()
            ]
        );
        assert_eq!(wallet2.relationship_path.len(), 1);

        assert_eq!(
            wallet2.relationship_path[0].from_wallet,
            "0x1111111111111111111111111111111111111111"
        );
        assert_eq!(
            wallet2.relationship_path[0].to_wallet,
            "0x3333333333333333333333333333333333333333"
        );
    }

    #[test]
    fn discovers_wallets_up_to_two_hops() {
        let discovered = discover_wallets(
            "0x1111111111111111111111111111111111111111",
            2,
            &sample_relationships(),
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

        let risky1 = discovered
            .iter()
            .find(|wallet| wallet.address == "0x4444444444444444444444444444444444444444")
            .expect("0x4444444444444444444444444444444444444444 should be discovered");

        assert_eq!(risky1.hop_distance, 2);
        assert_eq!(
            risky1.path,
            vec![
                "0x1111111111111111111111111111111111111111".to_string(),
                "0x2222222222222222222222222222222222222222".to_string(),
                "0x4444444444444444444444444444444444444444".to_string()
            ]
        );
        assert_eq!(risky1.relationship_path.len(), 2);

        assert_eq!(
            risky1.relationship_path[0].from_wallet,
            "0x1111111111111111111111111111111111111111"
        );
        assert_eq!(
            risky1.relationship_path[0].to_wallet,
            "0x2222222222222222222222222222222222222222"
        );
        assert_eq!(
            risky1.relationship_path[1].from_wallet,
            "0x2222222222222222222222222222222222222222"
        );
        assert_eq!(
            risky1.relationship_path[1].to_wallet,
            "0x4444444444444444444444444444444444444444"
        );

        let risky2 = discovered
            .iter()
            .find(|wallet| wallet.address == "0x5555555555555555555555555555555555555555")
            .expect("0x5555555555555555555555555555555555555555 should be discovered");

        assert_eq!(risky2.hop_distance, 2);
        assert_eq!(
            risky2.path,
            vec![
                "0x1111111111111111111111111111111111111111".to_string(),
                "0x3333333333333333333333333333333333333333".to_string(),
                "0x5555555555555555555555555555555555555555".to_string()
            ]
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
        );

        assert!(
            discovered
                .iter()
                .all(|wallet| wallet.address != "0x1111111111111111111111111111111111111111")
        );

        let wallet1 = discovered
            .iter()
            .find(|wallet| wallet.address == "0x2222222222222222222222222222222222222222")
            .expect("0x2222222222222222222222222222222222222222 should still be discovered");

        assert_eq!(
            wallet1.path,
            vec![
                "0x1111111111111111111111111111111111111111".to_string(),
                "0x2222222222222222222222222222222222222222".to_string()
            ]
        );

        assert_eq!(wallet1.relationship_path.len(), 1);
        assert_eq!(
            wallet1.relationship_path[0].from_wallet,
            "0x1111111111111111111111111111111111111111"
        );
        assert_eq!(
            wallet1.relationship_path[0].to_wallet,
            "0x2222222222222222222222222222222222222222"
        );
        assert_eq!(wallet1.relationship_path[0].transaction_count, 1);
        assert_eq!(
            wallet1.relationship_path[0].totals_by_asset.get("ETH"),
            Some(&1.0)
        );
    }
}
