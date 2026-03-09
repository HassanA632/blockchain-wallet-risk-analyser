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
