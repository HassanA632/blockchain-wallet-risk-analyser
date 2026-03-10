use clap::Parser;

use blockchain_wallet_risk_analyser::analysis::build_findings;
use blockchain_wallet_risk_analyser::cli::CliArgs;
use blockchain_wallet_risk_analyser::errors::AppError;
use blockchain_wallet_risk_analyser::loader::{load_risk_entities, load_transaction_edges};
use blockchain_wallet_risk_analyser::report::build_risk_report;
use blockchain_wallet_risk_analyser::risk::combine_risk_entities;
use blockchain_wallet_risk_analyser::traversal::discover_wallets;

fn main() -> Result<(), AppError> {
    let args = CliArgs::parse();

    let edges = load_transaction_edges("data/sample_graph.json")?;
    let built_in_risk_entities = load_risk_entities("data/risk_entities.json")?;

    let custom_risk_entities = match args.custom_risk_list.as_deref() {
        Some(path) => load_risk_entities(path)?,
        None => Vec::new(),
    };

    let combined_risk_entities =
        combine_risk_entities(built_in_risk_entities, custom_risk_entities);

    let discovered_wallets = discover_wallets(&args.wallet, args.hops, &edges);
    let findings = build_findings(&discovered_wallets, &combined_risk_entities);

    let report = build_risk_report(args.wallet, args.chain, args.hops, findings);

    let json =
        serde_json::to_string_pretty(&report).expect("risk report serialization should succeed");

    println!("{json}");

    Ok(())
}
