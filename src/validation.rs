/// Validates an Ethereum wallet address using basic format checks so invalid
/// input can be rejected before the pipeline runs.
pub fn validate_ethereum_address(address: &str) -> Result<String, String> {
    if !address.starts_with("0x") {
        return Err("Ethereum addresses must start with 0x".to_string());
    }

    if address.len() != 42 {
        return Err("Ethereum addresses must be 42 characters long".to_string());
    }

    if !address[2..]
        .chars()
        .all(|character| character.is_ascii_hexdigit())
    {
        return Err(
            "Ethereum addresses must contain only hexadecimal characters after 0x".to_string(),
        );
    }

    Ok(address.to_string())
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
        let result = validate_ethereum_address("0x1234567890abcdzf1234567890abcdef1234567z");

        assert_eq!(
            result,
            Err("Ethereum addresses must contain only hexadecimal characters after 0x".to_string())
        );
    }
}
