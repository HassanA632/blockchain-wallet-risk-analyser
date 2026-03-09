mod errors;
mod loader;
mod models;
mod report;
mod risk;
mod traversal;

use loader::{load_risk_entities, load_transaction_edges};
use risk::combine_risk_entities;
use traversal::discover_wallets;

fn main() -> Result<(), errors::AppError> {
    let target_wallet = "0xtarget";
    let hop_depth = 2;

    let edges = load_transaction_edges("data/sample_graph.json")?;
    let built_in_risk_entities = load_risk_entities("data/risk_entities.json")?;
    let custom_risk_entities = load_risk_entities("data/custom_risk_entities.json")?;
    let combined_risk_entities =
        combine_risk_entities(built_in_risk_entities, custom_risk_entities);

    let discovered_wallets = discover_wallets(target_wallet, hop_depth, &edges);

    println!("Loaded {} transaction edges", edges.len());
    println!(
        "Loaded {} combined risk entities",
        combined_risk_entities.len()
    );
    println!("Discovered {} wallets", discovered_wallets.len());

    Ok(())
}
