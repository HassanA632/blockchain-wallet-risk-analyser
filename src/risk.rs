use crate::models::RiskEntity;

/// Combines built in + analyst supplied risk entities into a single list
/// that can be used by the analysis engine during wallet matching.
pub fn combine_risk_entities(
    built_in: Vec<RiskEntity>,
    custom: Vec<RiskEntity>,
) -> Vec<RiskEntity> {
    let mut combined = built_in;
    combined.extend(custom);
    combined
}
