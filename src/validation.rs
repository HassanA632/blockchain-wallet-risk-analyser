/// Validates an Ethereum wallet address using basic format checks so invalid
/// input can be rejected before the pipeline runs.
pub fn validate_ethereum_address(address: &str) -> Result<String, String> {
    let normalized_address = normalize_ethereum_address(address);

    if !normalized_address.starts_with("0x") {
        return Err("Ethereum addresses must start with 0x".to_string());
    }

    if normalized_address.len() != 42 {
        return Err("Ethereum addresses must be 42 characters long".to_string());
    }

    if !normalized_address[2..]
        .chars()
        .all(|character| character.is_ascii_hexdigit())
    {
        return Err(
            "Ethereum addresses must contain only hexadecimal characters after 0x".to_string(),
        );
    }

    Ok(normalized_address)
}

/// Normalizes an Ethereum address into lowercase form used internally so
/// matching and deduplication remain consistent across data sources.
pub fn normalize_ethereum_address(address: &str) -> String {
    address.to_ascii_lowercase()
}

/// Validates a UTC timestamp in  ISO 8601 format so analysts can
/// supply date filters that compare safely as strings.
pub fn validate_utc_timestamp(timestamp: &str) -> Result<String, String> {
    if timestamp.len() != 20 {
        return Err("Timestamps must be in the format YYYY-MM-DDTHH:MM:SSZ".to_string());
    }

    let bytes = timestamp.as_bytes();

    let expected_separators = [
        (4, b'-'),
        (7, b'-'),
        (10, b'T'),
        (13, b':'),
        (16, b':'),
        (19, b'Z'),
    ];

    for (index, expected) in expected_separators {
        if bytes[index] != expected {
            return Err("Timestamps must be in the format YYYY-MM-DDTHH:MM:SSZ".to_string());
        }
    }

    let digit_positions = [0, 1, 2, 3, 5, 6, 8, 9, 11, 12, 14, 15, 17, 18];

    if !digit_positions
        .iter()
        .all(|&index| bytes[index].is_ascii_digit())
    {
        return Err("Timestamps must be in the format YYYY-MM-DDTHH:MM:SSZ".to_string());
    }

    Ok(timestamp.to_string())
}

/// Validates that an optional date range is chronologically ordered so analysts
/// do not accidentally request an inverted time window.
pub fn validate_date_range(from_date: Option<&str>, to_date: Option<&str>) -> Result<(), String> {
    if let (Some(from_date), Some(to_date)) = (from_date, to_date) {
        if from_date > to_date {
            return Err("from-date cannot be later than to-date".to_string());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_ethereum_address() {
        let address = "0x1234567890abcdef1234567890abcdef12345678";

        let result = validate_ethereum_address(address);

        assert_eq!(result, Ok(address.to_string()));
    }

    #[test]
    fn normalizes_mixed_case_ethereum_address() {
        let address = "0xAbCdEf1234567890aBCdef1234567890abCDef12";

        let result = validate_ethereum_address(address);

        assert_eq!(
            result,
            Ok("0xabcdef1234567890abcdef1234567890abcdef12".to_string())
        );
    }

    #[test]
    fn rejects_address_without_0x_prefix() {
        let result = validate_ethereum_address("1234567890abcdef1234567890abcdef12345678");

        assert_eq!(
            result,
            Err("Ethereum addresses must start with 0x".to_string())
        );
    }

    #[test]
    fn rejects_address_with_wrong_length() {
        let result = validate_ethereum_address("0x123456");

        assert_eq!(
            result,
            Err("Ethereum addresses must be 42 characters long".to_string())
        );
    }

    #[test]
    fn rejects_address_with_non_hex_characters() {
        let result = validate_ethereum_address("0x1234567890abcdef1234567890abcdef1234567z");

        assert_eq!(
            result,
            Err("Ethereum addresses must contain only hexadecimal characters after 0x".to_string())
        );
    }

    #[test]
    fn accepts_valid_utc_timestamp() {
        let result = validate_utc_timestamp("2026-03-11T10:05:00Z");

        assert_eq!(result, Ok("2026-03-11T10:05:00Z".to_string()));
    }

    #[test]
    fn rejects_badly_formatted_utc_timestamp() {
        let result = validate_utc_timestamp("2026/03/11 10:05:00");

        assert_eq!(
            result,
            Err("Timestamps must be in the format YYYY-MM-DDTHH:MM:SSZ".to_string())
        );
    }

    #[test]
    fn accepts_valid_date_range() {
        let result =
            validate_date_range(Some("2026-03-11T10:00:00Z"), Some("2026-03-11T10:15:00Z"));

        assert_eq!(result, Ok(()));
    }

    #[test]
    fn rejects_inverted_date_range() {
        let result =
            validate_date_range(Some("2026-03-11T10:15:00Z"), Some("2026-03-11T10:00:00Z"));

        assert_eq!(
            result,
            Err("from-date cannot be later than to-date".to_string())
        );
    }
}
