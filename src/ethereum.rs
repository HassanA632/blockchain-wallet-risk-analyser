use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

use alloy::{
    primitives::{Address, B256, U256, b256},
    providers::{Provider, ProviderBuilder},
    rpc::types::{BlockNumberOrTag, Filter, Log},
};

use crate::errors::AppError;
use crate::models::TransactionEdge;

const LOG_QUERY_BLOCK_WINDOW: u64 = 9;
const LOG_QUERY_WINDOW_COUNT: u64 = 25;
const ERC20_TRANSFER_EVENT_SIGNATURE: B256 =
    b256!("ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EthereumSourceConfig {
    pub rpc_url: String,
}

/// Loads transaction data for a wallet from Ethereum.
///
/// This version scans several recent block windows for ERC-20
/// transfer logs where the wallet appears as sender or receiver.
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

    eprintln!(
        "[ethereum] wallet={} latest_block={} window_size={} window_count={}",
        format!("{wallet:#x}"),
        latest_block,
        LOG_QUERY_BLOCK_WINDOW,
        LOG_QUERY_WINDOW_COUNT
    );

    let block_windows =
        build_block_windows(latest_block, LOG_QUERY_BLOCK_WINDOW, LOG_QUERY_WINDOW_COUNT);

    let mut all_logs = Vec::new();

    for (index, (from_block, to_block)) in block_windows.iter().enumerate() {
        let outgoing_logs = fetch_wallet_transfer_logs(
            &provider,
            wallet,
            *from_block,
            *to_block,
            TransferDirection::Outgoing,
        )
        .await?;

        let incoming_logs = fetch_wallet_transfer_logs(
            &provider,
            wallet,
            *from_block,
            *to_block,
            TransferDirection::Incoming,
        )
        .await?;

        eprintln!(
            "[ethereum] window={} from_block={} to_block={} outgoing_logs={} incoming_logs={}",
            index + 1,
            from_block,
            to_block,
            outgoing_logs.len(),
            incoming_logs.len()
        );

        all_logs.extend(outgoing_logs);
        all_logs.extend(incoming_logs);
    }

    let timestamp_by_block = build_block_timestamp_map(&provider, &all_logs).await?;

    let mut edges = Vec::new();

    for log in all_logs {
        if let Some(edge) = map_transfer_log_to_edge(log, wallet, &timestamp_by_block) {
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

/// Builds recent inclusive block windows so log queries can stay within
/// provider limits while still looking back across a wider history.
fn build_block_windows(latest_block: u64, window_size: u64, window_count: u64) -> Vec<(u64, u64)> {
    let mut windows = Vec::new();
    let mut current_to_block = latest_block;

    for _ in 0..window_count {
        let current_from_block = current_to_block.saturating_sub(window_size);
        windows.push((current_from_block, current_to_block));

        if current_from_block == 0 {
            break;
        }

        current_to_block = current_from_block.saturating_sub(1);
    }

    windows
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

/// Builds a lookup map of block number to UTC timestamp string so
/// live Ethereum edges can carry real block timestamp.
async fn build_block_timestamp_map(
    provider: &impl Provider,
    logs: &[Log],
) -> Result<HashMap<u64, String>, AppError> {
    let mut unique_block_numbers = HashSet::new();

    for log in logs {
        if let Some(block_number) = log.block_number {
            unique_block_numbers.insert(block_number);
        }
    }

    let mut timestamp_by_block = HashMap::new();

    for block_number in unique_block_numbers {
        let block = provider
            .get_block_by_number(BlockNumberOrTag::Number(block_number))
            .await
            .map_err(|error| {
                AppError::Source(format!(
                    "failed to fetch block {block_number} for timestamp enrichment: {error}"
                ))
            })?
            .ok_or_else(|| {
                AppError::Source(format!(
                    "block {block_number} was not returned during timestamp enrichment"
                ))
            })?;

        let timestamp = format_block_timestamp(block.header.timestamp);

        timestamp_by_block.insert(block_number, timestamp);
    }

    Ok(timestamp_by_block)
}

/// Formats a Unix block timestamp into a UTC string suitable for filtering.
///
/// This keeps the timestamp stable and comparable even before a full datetime
/// library is introduced for richer formatting.
fn format_block_timestamp(timestamp_seconds: u64) -> String {
    use chrono::{TimeZone, Utc};

    Utc.timestamp_opt(timestamp_seconds as i64, 0)
        .single()
        .expect("block timestamp should convert to a valid UTC datetime")
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string()
}

/// Maps a matching ERC-20 transfer log into the internal transaction-edge shape.
fn map_transfer_log_to_edge(
    log: Log,
    wallet: Address,
    timestamp_by_block: &HashMap<u64, String>,
) -> Option<TransactionEdge> {
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
    let block_number = log.block_number?;
    let timestamp = timestamp_by_block.get(&block_number)?.clone();

    Some(TransactionEdge {
        from_address: format!("{from_address:#x}"),
        to_address: format!("{to_address:#x}"),
        tx_hash: format!("{tx_hash:#x}"),
        asset: format!("{token_address:#x}"),
        amount,
        timestamp,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransferDirection {
    Outgoing,
    Incoming,
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
    fn builds_recent_block_windows() {
        let windows = build_block_windows(100, 9, 3);

        assert_eq!(windows, vec![(91, 100), (81, 90), (71, 80)]);
    }

    #[test]
    fn stops_block_windows_at_zero() {
        let windows = build_block_windows(5, 9, 3);

        assert_eq!(windows, vec![(0, 5)]);
    }

    #[test]
    fn formats_block_timestamp_as_utc_string() {
        let formatted = format_block_timestamp(1_700_000_000);

        assert_eq!(formatted, "2023-11-14T22:13:20Z");
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
                timestamp: "2026-03-14T12:00:00Z".to_string(),
            },
            TransactionEdge {
                from_address: "0xaaa".to_string(),
                to_address: "0xbbb".to_string(),
                tx_hash: "0xtx".to_string(),
                asset: "0xtoken".to_string(),
                amount: "100".to_string(),
                timestamp: "2026-03-14T12:00:00Z".to_string(),
            },
        ];

        let deduplicated = deduplicate_edges(edges);

        assert_eq!(deduplicated.len(), 1);
    }
}
