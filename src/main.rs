mod errors;
mod loader;
mod models;
mod report;
mod risk;

use loader::{load_risk_entities, load_transaction_edges};
use risk::combine_risk_entities;

fn main() -> Result<(), errors::AppError> {
    let edges = load_transaction_edges("data/sample_graph.json")?;
    let built_in_risk_entities = load_risk_entities("data/risk_entities.json")?;
    let custom_risk_entities = load_risk_entities("data/custom_risk_entities.json")?;
    let combined_risk_entities =
        combine_risk_entities(built_in_risk_entities, custom_risk_entities);

    println!("Loaded {} transaction edges", edges.len());
    println!(
        "Loaded {} combined risk entities",
        combined_risk_entities.len()
    );

    Ok(())
}
