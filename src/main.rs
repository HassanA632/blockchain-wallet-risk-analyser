use blockchain_wallet_risk_analyser::analysis::build_findings;
use blockchain_wallet_risk_analyser::errors::AppError;
use blockchain_wallet_risk_analyser::loader::{load_risk_entities, load_transaction_edges};
use blockchain_wallet_risk_analyser::risk::combine_risk_entities;
use blockchain_wallet_risk_analyser::traversal::discover_wallets;

fn main() -> Result<(), AppError> {
    let target_wallet = "0xtarget";
    let hop_depth = 2;

    let edges = load_transaction_edges("data/sample_graph.json")?;
    let built_in_risk_entities = load_risk_entities("data/risk_entities.json")?;
    let custom_risk_entities = load_risk_entities("data/custom_risk_entities.json")?;
    let combined_risk_entities =
        combine_risk_entities(built_in_risk_entities, custom_risk_entities);

    let discovered_wallets = discover_wallets(target_wallet, hop_depth, &edges);
    let findings = build_findings(&discovered_wallets, &combined_risk_entities);

    println!("Loaded {} transaction edges", edges.len());
    println!(
        "Loaded {} combined risk entities",
        combined_risk_entities.len()
    );
    println!("Discovered {} wallets", discovered_wallets.len());
    println!("Built {} findings", findings.len());

    Ok(())
}
