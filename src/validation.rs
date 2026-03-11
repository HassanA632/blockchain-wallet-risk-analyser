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
}
