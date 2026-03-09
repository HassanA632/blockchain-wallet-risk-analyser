mod errors;
mod loader;
mod models;
mod report;

use loader::{load_risk_entities, load_transaction_edges};

fn main() -> Result<(), errors::AppError> {
    let edges = load_transaction_edges("data/sample_graph.json")?;
    let built_in_risk_entities = load_risk_entities("data/risk_entities.json")?;
    let custom_risk_entities = load_risk_entities("data/custom_risk_entities.json")?;

    println!("Loaded {} transaction edges", edges.len());
    println!(
        "Loaded {} built-in risk entities",
        built_in_risk_entities.len()
    );
    println!("Loaded {} custom risk entities", custom_risk_entities.len());

    Ok(())
}
