use std::collections::HashMap;

use crate::models::ServiceWallet;

/// Builds service wallet index keyed by address so traversal and
/// reporting can look up known infrastructure wallets quickly.
pub fn build_service_wallet_index(
    service_wallets: Vec<ServiceWallet>,
) -> HashMap<String, ServiceWallet> {
    service_wallets
        .into_iter()
        .map(|service_wallet| (service_wallet.address.clone(), service_wallet))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ServiceType, ServiceWallet};

    #[test]
    fn builds_service_wallet_index_by_address() {
        let service_wallets = vec![ServiceWallet {
            address: "0xabc".to_string(),
            label: "Kraken Hot Wallet".to_string(),
            service_type: ServiceType::Exchange,
        }];

        let index = build_service_wallet_index(service_wallets);

        assert_eq!(index.len(), 1);
        assert_eq!(
            index
                .get("0xabc")
                .expect("service wallet should exist")
                .label,
            "Kraken Hot Wallet"
        );
    }
}
