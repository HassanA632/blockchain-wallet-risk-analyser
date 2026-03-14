use std::{collections::HashSet, str::FromStr};

use alloy::{
    primitives::{Address, B256, U256, b256},
    providers::{Provider, ProviderBuilder},
    rpc::types::{BlockNumberOrTag, Filter, Log},
};

use crate::errors::AppError;
use crate::models::TransactionEdge;

const DEFAULT_RECENT_BLOCK_WINDOW: u64 = 9;
const ERC20_TRANSFER_EVENT_SIGNATURE: B256 =
    b256!("ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EthereumSourceConfig {
    pub rpc_url: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransferDirection {
    Outgoing,
    Incoming,
}

/// Loads transaction data for a wallet from Ethereum.
///
/// This first live version fetches a bounded recent set of ERC-20 transfer logs
/// where the wallet appears as sender or receiver and then maps them into
/// transaction edges for the existing analysis pipeline.
pub async fn load_transaction_edges_from_ethereum(
    wallet: &str,
    config: &EthereumSourceConfig,
) -> Result<Vec<TransactionEdge>, AppError> {
    let provider = build_provider(config).await?;
    let wallet = parse_wallet_address(wallet)?;

    let latest_block = provider
        .get_block_number()
        .await
        .map_err(|error| AppError::Source(format!("failed to get latest block number: {error}")))?;

    let from_block = latest_block.saturating_sub(DEFAULT_RECENT_BLOCK_WINDOW);

    // Temporary debug output to trace live Ethereum ingestion during development.
    eprintln!(
        "[ethereum] wallet={} latest_block={} from_block={} window={}",
        format!("{wallet:#x}"),
        latest_block,
        from_block,
        DEFAULT_RECENT_BLOCK_WINDOW
    );

    let outgoing_logs = fetch_wallet_transfer_logs(
        &provider,
        wallet,
        from_block,
        latest_block,
        TransferDirection::Outgoing,
    )
    .await?;

    let incoming_logs = fetch_wallet_transfer_logs(
        &provider,
        wallet,
        from_block,
        latest_block,
        TransferDirection::Incoming,
    )
    .await?;

    eprintln!(
        "[ethereum] outgoing_logs={} incoming_logs={}",
        outgoing_logs.len(),
        incoming_logs.len()
    );

    let mut edges = Vec::new();

    for log in outgoing_logs.into_iter().chain(incoming_logs.into_iter()) {
        if let Some(edge) = map_transfer_log_to_edge(log, wallet) {
            edges.push(edge);
        }
    }

    let edges = deduplicate_edges(edges);

    eprintln!("[ethereum] transaction_edges={}", edges.len());

    Ok(edges)
}

/// Reads Ethereum source configuration from the environment so the source can
/// connect to an RPC endpoint without hardcoding provider details.
pub fn load_ethereum_source_config() -> Result<EthereumSourceConfig, AppError> {
    let rpc_url = std::env::var("ETH_RPC_URL").ok();
    ethereum_source_config_from_rpc_url(rpc_url.as_deref())
}

/// Builds Ethereum source configuration from an optional RPC URL so config
/// validation can be tested without mutating global environment variables.
fn ethereum_source_config_from_rpc_url(
    rpc_url: Option<&str>,
) -> Result<EthereumSourceConfig, AppError> {
    let rpc_url = rpc_url.ok_or_else(|| {
        AppError::Source("ETH_RPC_URL environment variable is not set".to_string())
    })?;

    Ok(EthereumSourceConfig {
        rpc_url: rpc_url.to_string(),
    })
}

/// Builds an Alloy provider from the current Ethereum source configuration so
/// later ingestion code can reuse one setup path.
async fn build_provider(config: &EthereumSourceConfig) -> Result<impl Provider, AppError> {
    let rpc_url = config
        .rpc_url
        .parse()
        .map_err(|error| AppError::Source(format!("invalid ETH_RPC_URL: {error}")))?;

    Ok(ProviderBuilder::new().connect_http(rpc_url))
}

/// Parses the target wallet into an Alloy address so Ethereum log matching can
/// compare against a typed wallet value.
fn parse_wallet_address(wallet: &str) -> Result<Address, AppError> {
    Address::from_str(wallet)
        .map_err(|error| AppError::Source(format!("invalid Ethereum wallet address: {error}")))
}

/// Fetches ERC-20 transfer logs for a wallet in one direction so the live query
/// only returns logs relevant to the wallet currently being analysed.
async fn fetch_wallet_transfer_logs(
    provider: &impl Provider,
    wallet: Address,
    from_block: u64,
    to_block: u64,
    direction: TransferDirection,
) -> Result<Vec<Log>, AppError> {
    let wallet_topic = address_to_topic(wallet);

    let filter = match direction {
        TransferDirection::Outgoing => Filter::new()
            .event_signature(ERC20_TRANSFER_EVENT_SIGNATURE)
            .from_block(BlockNumberOrTag::Number(from_block))
            .to_block(BlockNumberOrTag::Number(to_block))
            .topic1(wallet_topic),
        TransferDirection::Incoming => Filter::new()
            .event_signature(ERC20_TRANSFER_EVENT_SIGNATURE)
            .from_block(BlockNumberOrTag::Number(from_block))
            .to_block(BlockNumberOrTag::Number(to_block))
            .topic2(wallet_topic),
    };

    provider.get_logs(&filter).await.map_err(|error| {
        let direction_label = match direction {
            TransferDirection::Outgoing => "outgoing",
            TransferDirection::Incoming => "incoming",
        };

        AppError::Source(format!(
            "failed to fetch {direction_label} ERC-20 transfer logs: {error}"
        ))
    })
}

/// Maps a matching ERC-20 transfer log into the internal transaction-edge shape.
///
/// This first version keeps the token contract address as the asset identifier
/// and uses a placeholder timestamp until block timestamp enrichment is added.
fn map_transfer_log_to_edge(log: Log, wallet: Address) -> Option<TransactionEdge> {
    let topics = log.topics();

    if topics.len() < 3 {
        return None;
    }

    let from_address = topic_to_address(topics[1])?;
    let to_address = topic_to_address(topics[2])?;

    if from_address != wallet && to_address != wallet {
        return None;
    }

    let tx_hash = log.transaction_hash?;
    let token_address = log.address();
    let amount = decode_transfer_value(log.data().data.as_ref())?;

    Some(TransactionEdge {
        from_address: format!("{from_address:#x}"),
        to_address: format!("{to_address:#x}"),
        tx_hash: format!("{tx_hash:#x}"),
        asset: format!("{token_address:#x}"),
        amount,
        timestamp: "unknown".to_string(),
    })
}

/// Deduplicates edges after incoming and outgoing log queries are merged so a
/// self-transfer or overlapping result does not appear twice.
fn deduplicate_edges(edges: Vec<TransactionEdge>) -> Vec<TransactionEdge> {
    let mut seen = HashSet::new();

    edges
        .into_iter()
        .filter(|edge| {
            let key = format!(
                "{}|{}|{}|{}|{}",
                edge.tx_hash, edge.from_address, edge.to_address, edge.asset, edge.amount
            );

            seen.insert(key)
        })
        .collect()
}

/// Converts an address into the 32-byte topic form used for indexed log
/// filtering on Ethereum.
fn address_to_topic(address: Address) -> B256 {
    let mut bytes = [0u8; 32];
    bytes[12..].copy_from_slice(address.as_slice());
    B256::from(bytes)
}

/// Extracts an Ethereum address from an indexed log topic.
fn topic_to_address(topic: B256) -> Option<Address> {
    let bytes = topic.as_slice();
    Some(Address::from_slice(&bytes[12..]))
}

/// Decodes the ERC-20 transfer value from log data into a decimal string.
fn decode_transfer_value(data: &[u8]) -> Option<String> {
    if data.len() != 32 {
        return None;
    }

    let value = U256::from_be_slice(data);
    Some(value.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_ethereum_source_config_from_rpc_url() {
        let config = ethereum_source_config_from_rpc_url(Some("http://127.0.0.1:8545"))
            .expect("config should build from rpc url");

        assert_eq!(config.rpc_url, "http://127.0.0.1:8545");
    }

    #[test]
    fn returns_error_when_rpc_url_is_missing() {
        let result = ethereum_source_config_from_rpc_url(None);

        match result {
            Err(AppError::Source(message)) => {
                assert_eq!(message, "ETH_RPC_URL environment variable is not set");
            }
            _ => panic!("expected missing ETH_RPC_URL source error"),
        }
    }

    #[test]
    fn converts_address_to_topic() {
        let address =
            Address::from_str("0x1111111111111111111111111111111111111111").expect("valid address");

        let topic = address_to_topic(address);

        assert_eq!(
            format!("{topic:#x}"),
            "0x0000000000000000000000001111111111111111111111111111111111111111"
        );
    }

    #[test]
    fn extracts_address_from_topic() {
        let topic = b256!("0000000000000000000000001111111111111111111111111111111111111111");

        let address = topic_to_address(topic).expect("topic should decode to address");

        assert_eq!(
            format!("{address:#x}"),
            "0x1111111111111111111111111111111111111111"
        );
    }

    #[test]
    fn decodes_transfer_value_from_log_data() {
        let data = hex::decode("00000000000000000000000000000000000000000000000000000000000003e8")
            .expect("hex data should decode");

        let amount = decode_transfer_value(&data).expect("value should decode");

        assert_eq!(amount, "1000");
    }

    #[test]
    fn deduplicates_merged_transaction_edges() {
        let edges = vec![
            TransactionEdge {
                from_address: "0xaaa".to_string(),
                to_address: "0xbbb".to_string(),
                tx_hash: "0xtx".to_string(),
                asset: "0xtoken".to_string(),
                amount: "100".to_string(),
                timestamp: "unknown".to_string(),
            },
            TransactionEdge {
                from_address: "0xaaa".to_string(),
                to_address: "0xbbb".to_string(),
                tx_hash: "0xtx".to_string(),
                asset: "0xtoken".to_string(),
                amount: "100".to_string(),
                timestamp: "unknown".to_string(),
            },
        ];

        let deduplicated = deduplicate_edges(edges);

        assert_eq!(deduplicated.len(), 1);
    }
}
