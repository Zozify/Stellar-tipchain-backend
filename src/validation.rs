pub fn validate_username(username: &str) -> Result<(), String> {
    if username.len() < 3 || username.len() > 32 {
        return Err("username must be between 3 and 32 characters".to_string());
    }
    if !username
        .chars()
        .next()
        .map(|c| c.is_ascii_alphabetic())
        .unwrap_or(false)
    {
        return Err("username must start with a letter".to_string());
    }
    if !username
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err("username may only contain letters, numbers, and underscores".to_string());
    }
    Ok(())
}

pub fn validate_stellar_address(address: &str) -> Result<(), String> {
    if address.len() != 56 {
        return Err("wallet_address must be a 56-character Stellar public key".to_string());
    }
    if !address.starts_with('G') {
        return Err("wallet_address must be a valid Stellar public key starting with 'G'".to_string());
    }
    if !address
        .chars()
        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
    {
        return Err("wallet_address must be base32 encoded (A-Z, 2-7)".to_string());
    }
    Ok(())
}

pub fn validate_amount(amount: &str) -> Result<(), String> {
    match amount.trim().parse::<f64>() {
        Ok(value) if value.is_finite() && value > 0.0 => Ok(()),
        _ => Err("amount must be a positive number".to_string()),
    }
}

pub fn validate_transaction_hash(hash: &str) -> Result<(), String> {
    if hash.trim().is_empty() {
        return Err("transaction_hash must not be empty".to_string());
    }
    if !hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("transaction_hash must be a hexadecimal string".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_username() {
        assert!(validate_username("alice_99").is_ok());
    }

    #[test]
    fn rejects_short_username() {
        assert!(validate_username("ab").is_err());
    }

    #[test]
    fn rejects_username_starting_with_digit() {
        assert!(validate_username("9alice").is_err());
    }

    #[test]
    fn rejects_username_with_invalid_chars() {
        assert!(validate_username("alice!").is_err());
    }

    #[test]
    fn accepts_valid_stellar_address() {
        let address = format!("G{}", "A".repeat(55));
        assert!(validate_stellar_address(&address).is_ok());
    }

    #[test]
    fn rejects_wrong_length_address() {
        assert!(validate_stellar_address("GSHORT").is_err());
    }

    #[test]
    fn rejects_address_not_starting_with_g() {
        let address = format!("A{}", "A".repeat(55));
        assert!(validate_stellar_address(&address).is_err());
    }

    #[test]
    fn rejects_lowercase_address() {
        let address = format!("G{}", "a".repeat(55));
        assert!(validate_stellar_address(&address).is_err());
    }

    #[test]
    fn accepts_valid_amount() {
        assert!(validate_amount("10.5").is_ok());
    }

    #[test]
    fn rejects_zero_amount() {
        assert!(validate_amount("0").is_err());
    }

    #[test]
    fn rejects_negative_amount() {
        assert!(validate_amount("-5").is_err());
    }

    #[test]
    fn rejects_non_numeric_amount() {
        assert!(validate_amount("abc").is_err());
    }
}
