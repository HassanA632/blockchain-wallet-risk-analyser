use clap::Parser;
use std::path::PathBuf;

use blockchain_wallet_risk_analyser::analysis::build_findings;
use blockchain_wallet_risk_analyser::cli::CliArgs;
use blockchain_wallet_risk_analyser::errors::AppError;
use blockchain_wallet_risk_analyser::filter::filter_edges_by_date_range;
use blockchain_wallet_risk_analyser::loader::load_service_wallets;
use blockchain_wallet_risk_analyser::loader::{
    load_built_in_risk_entities, load_custom_risk_entities,
};
use blockchain_wallet_risk_analyser::models::DataSource;
use blockchain_wallet_risk_analyser::output::write_output;
use blockchain_wallet_risk_analyser::relationships::build_wallet_relationships;
use blockchain_wallet_risk_analyser::report::build_risk_report;
use blockchain_wallet_risk_analyser::risk::build_risk_index;
use blockchain_wallet_risk_analyser::service_wallets::build_service_wallet_index;
use blockchain_wallet_risk_analyser::source::{TransactionEdgeSource, load_edges_from_source};
use blockchain_wallet_risk_analyser::traversal::discover_wallets;

const DEFAULT_GRAPH_PATH: &str = "data/sample_graph.json";
const DEFAULT_RISK_LIST_PATH: &str = "data/risk_entities.json";
const DEFAULT_SERVICE_WALLET_LIST_PATH: &str = "data/service_wallets.json";

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let args = CliArgs::parse();
    args.validate().map_err(AppError::Cli)?;

    let edge_source = match args.source {
        DataSource::Local => {
            let graph_path = args.graph.as_deref().unwrap_or(DEFAULT_GRAPH_PATH);

            TransactionEdgeSource::LocalFile {
                path: PathBuf::from(graph_path),
            }
        }
        DataSource::Ethereum => TransactionEdgeSource::Ethereum {
            wallet: args.wallet.clone(),
        },
    };

    let edges = load_edges_from_source(&edge_source).await?;
    let filtered_edges =
        filter_edges_by_date_range(&edges, args.from_date.as_deref(), args.to_date.as_deref());

    let built_in_risk_entities = load_built_in_risk_entities(DEFAULT_RISK_LIST_PATH)?;
    let service_wallets = load_service_wallets(DEFAULT_SERVICE_WALLET_LIST_PATH)?;
    let service_wallet_index = build_service_wallet_index(service_wallets);

    let custom_risk_entities = match args.custom_risk_list.as_deref() {
        Some(path) => load_custom_risk_entities(path)?,
        None => Vec::new(),
    };

    let risk_index = build_risk_index(built_in_risk_entities, custom_risk_entities);
    let relationships = build_wallet_relationships(&filtered_edges);
    let discovered_wallets = discover_wallets(
        &args.wallet,
        args.hops,
        &relationships,
        &service_wallet_index,
    );

    let findings = build_findings(&discovered_wallets, &risk_index);

    let report = build_risk_report(args.wallet, args.chain, args.hops, findings);

    let json =
        serde_json::to_string_pretty(&report).expect("risk report serialization should succeed");

    match args.output.as_deref() {
        Some(path) => write_output(path, &json)?,
        None => println!("{json}"),
    }

    Ok(())
}
