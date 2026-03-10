use std::collections::HashSet;

use crate::models::{DiscoveredWallet, TransactionEdge};

/// Discovers wallets connected to a target address within the requested hop
/// depth so later analysis can check them against risk intelligence.
pub fn discover_wallets(
    target_wallet: &str,
    hop_depth: u8,
    edges: &[TransactionEdge],
) -> Vec<DiscoveredWallet> {
    let mut discovered = Vec::new();
    let mut seen_addresses = HashSet::new();
    let mut first_hop_wallets = Vec::new();

    for edge in edges {
        if let Some(neighbour) = connected_wallet(target_wallet, edge) {
            if seen_addresses.insert(neighbour.to_string()) {
                let wallet = DiscoveredWallet {
                    address: neighbour.to_string(),
                    hop_distance: 1,
                    path: vec![target_wallet.to_string(), neighbour.to_string()],
                };

                first_hop_wallets.push(neighbour.to_string());
                discovered.push(wallet);
            }
        }
    }

    if hop_depth < 2 {
        return discovered;
    }

    for first_hop_wallet in first_hop_wallets {
        for edge in edges {
            if let Some(neighbour) = connected_wallet(&first_hop_wallet, edge) {
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
                    });
                }
            }
        }
    }

    discovered
}

/// Returns the wallet on the opposite side of an interaction edge when the
/// given address is part of that edge, allowing traversal to treat edges as
/// wallet-to-wallet connections.
fn connected_wallet<'a>(address: &str, edge: &'a TransactionEdge) -> Option<&'a str> {
    if edge.from_address == address {
        Some(edge.to_address.as_str())
    } else if edge.to_address == address {
        Some(edge.from_address.as_str())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TransactionEdge;

    fn sample_edges() -> Vec<TransactionEdge> {
        vec![
            TransactionEdge {
                from_address: "0xtarget".to_string(),
                to_address: "0xwallet1".to_string(),
            },
            TransactionEdge {
                from_address: "0xtarget".to_string(),
                to_address: "0xwallet2".to_string(),
            },
            TransactionEdge {
                from_address: "0xwallet1".to_string(),
                to_address: "0xrisky1".to_string(),
            },
            TransactionEdge {
                from_address: "0xwallet2".to_string(),
                to_address: "0xrisky2".to_string(),
            },
        ]
    }

    #[test]
    fn discovers_direct_wallets_at_one_hop() {
        let discovered = discover_wallets("0xtarget", 1, &sample_edges());

        assert_eq!(discovered.len(), 2);
        assert!(
            discovered
                .iter()
                .any(|wallet| wallet.address == "0xwallet1")
        );
        assert!(
            discovered
                .iter()
                .any(|wallet| wallet.address == "0xwallet2")
        );
        assert!(discovered.iter().all(|wallet| wallet.hop_distance == 1));

        let wallet1 = discovered
            .iter()
            .find(|wallet| wallet.address == "0xwallet1")
            .expect("0xwallet1 should be discovered");

        assert_eq!(
            wallet1.path,
            vec!["0xtarget".to_string(), "0xwallet1".to_string()]
        );

        let wallet2 = discovered
            .iter()
            .find(|wallet| wallet.address == "0xwallet2")
            .expect("0xwallet2 should be discovered");

        assert_eq!(
            wallet2.path,
            vec!["0xtarget".to_string(), "0xwallet2".to_string()]
        );
    }

    #[test]
    fn discovers_wallets_up_to_two_hops() {
        let discovered = discover_wallets("0xtarget", 2, &sample_edges());

        assert_eq!(discovered.len(), 4);
        assert!(discovered.iter().any(|wallet| wallet.address == "0xrisky1"));
        assert!(discovered.iter().any(|wallet| wallet.address == "0xrisky2"));

        let risky1 = discovered
            .iter()
            .find(|wallet| wallet.address == "0xrisky1")
            .expect("0xrisky1 should be discovered");

        assert_eq!(risky1.hop_distance, 2);
        assert_eq!(
            risky1.path,
            vec![
                "0xtarget".to_string(),
                "0xwallet1".to_string(),
                "0xrisky1".to_string()
            ]
        );

        let risky2 = discovered
            .iter()
            .find(|wallet| wallet.address == "0xrisky2")
            .expect("0xrisky2 should be discovered");

        assert_eq!(risky2.hop_distance, 2);
        assert_eq!(
            risky2.path,
            vec![
                "0xtarget".to_string(),
                "0xwallet2".to_string(),
                "0xrisky2".to_string()
            ]
        );
    }

    #[test]
    fn does_not_rediscover_target_wallet() {
        let edges = vec![
            TransactionEdge {
                from_address: "0xtarget".to_string(),
                to_address: "0xwallet1".to_string(),
            },
            TransactionEdge {
                from_address: "0xwallet1".to_string(),
                to_address: "0xtarget".to_string(),
            },
        ];

        let discovered = discover_wallets("0xtarget", 2, &edges);

        assert!(discovered.iter().all(|wallet| wallet.address != "0xtarget"));

        let wallet1 = discovered
            .iter()
            .find(|wallet| wallet.address == "0xwallet1")
            .expect("0xwallet1 should still be discovered");

        assert_eq!(
            wallet1.path,
            vec!["0xtarget".to_string(), "0xwallet1".to_string()]
        );
    }
}
