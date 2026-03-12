use crate::models::TransactionEdge;

/// Filters transaction edges with optional timestamp range so analyst can
/// restrict exposure to a specific time window.
pub fn filter_edges_by_date_range(
    edges: &[TransactionEdge],
    from_date: Option<&str>,
    to_date: Option<&str>,
) -> Vec<TransactionEdge> {
    edges
        .iter()
        .filter(|edge| matches_date_range(&edge.timestamp, from_date, to_date))
        .cloned()
        .collect()
}

/// Checks if a timestamp falls within the range so filtering logic stays separate from the main pipeline.
fn matches_date_range(timestamp: &str, from_date: Option<&str>, to_date: Option<&str>) -> bool {
    if let Some(from_date) = from_date {
        if timestamp < from_date {
            return false;
        }
    }

    if let Some(to_date) = to_date {
        if timestamp > to_date {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TransactionEdge;

    fn sample_edges() -> Vec<TransactionEdge> {
        vec![
            TransactionEdge {
                from_address: "0x1111111111111111111111111111111111111111".to_string(),
                to_address: "0x2222222222222222222222222222222222222222".to_string(),
                tx_hash: "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                    .to_string(),
                asset: "ETH".to_string(),
                amount: "1.25".to_string(),
                timestamp: "2026-03-11T10:00:00Z".to_string(),
            },
            TransactionEdge {
                from_address: "0x1111111111111111111111111111111111111111".to_string(),
                to_address: "0x3333333333333333333333333333333333333333".to_string(),
                tx_hash: "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
                    .to_string(),
                asset: "USDC".to_string(),
                amount: "500.00".to_string(),
                timestamp: "2026-03-11T10:05:00Z".to_string(),
            },
            TransactionEdge {
                from_address: "0x2222222222222222222222222222222222222222".to_string(),
                to_address: "0x4444444444444444444444444444444444444444".to_string(),
                tx_hash: "0xcccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
                    .to_string(),
                asset: "ETH".to_string(),
                amount: "0.75".to_string(),
                timestamp: "2026-03-11T10:10:00Z".to_string(),
            },
        ]
    }

    #[test]
    fn keeps_all_edges_when_no_date_range_is_provided() {
        let filtered = filter_edges_by_date_range(&sample_edges(), None, None);

        assert_eq!(filtered.len(), 3);
    }

    #[test]
    fn filters_edges_from_a_lower_bound_inclusively() {
        let filtered =
            filter_edges_by_date_range(&sample_edges(), Some("2026-03-11T10:05:00Z"), None);

        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].timestamp, "2026-03-11T10:05:00Z");
        assert_eq!(filtered[1].timestamp, "2026-03-11T10:10:00Z");
    }

    #[test]
    fn filters_edges_to_an_upper_bound_inclusively() {
        let filtered =
            filter_edges_by_date_range(&sample_edges(), None, Some("2026-03-11T10:05:00Z"));

        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].timestamp, "2026-03-11T10:00:00Z");
        assert_eq!(filtered[1].timestamp, "2026-03-11T10:05:00Z");
    }

    #[test]
    fn filters_edges_within_a_bounded_range() {
        let filtered = filter_edges_by_date_range(
            &sample_edges(),
            Some("2026-03-11T10:01:00Z"),
            Some("2026-03-11T10:09:00Z"),
        );

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].timestamp, "2026-03-11T10:05:00Z");
    }
}
