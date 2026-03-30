use once_cell::sync::Lazy;
use regex::Regex;

// Indonesian phone number: +62xxx or 08xxx, 10 - 15 digits total
pub static PHONE_ID_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(\+62|08)[0-9]{8,12}$").expect("valid regex")
});

// SKU format: upparcase letters, digits, hypens only, 3 – 50 chars.. E.g. WMS-PROD-001
static SKU_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[A-Z0-9][A-Z0-9\-]{1,48}[A-Z0-9]$").expect("valid regex")
});

/// Valid: `+628123456789`, `081234567890`
/// Invalid: `12345`, `+1234567890` (non-Indonesian prefix)
pub fn validate_indonesian_phone(phone: &str) -> Result<(), validator::ValidationError> {
    if PHONE_ID_REGEX.is_match(phone) {
        Ok(())
    } else {
        let mut err = validator::ValidationError::new("phone_format");

        err.message = Some("Phone must be a valid Indonesian number (+62xxx or 08xxx)".into());
        Err(err)
    }
}

/// Validates SKU format — uppercase alphanumeric with hyphens, 3 – 50 chars.
pub fn validate_sku(sku: &str) -> Result<(), validator::ValidationError> {
    if SKU_REGEX.is_match(sku) {
        Ok(())
    } else {
        let mut err = validator::ValidationError::new("sku_format");
        err.message = Some(
            "SKU must contain only uppercase letters, digits, and hyphens (e.g. WMS-PROD-001)".into(),
        );
        Err(err)
    }
}
 
/// Validates password strength — requires at least one uppercase, lowercase, and digit.
pub fn validate_password_strength(password: &str) -> Result<(), validator::ValidationError> {
    let has_min_length = password.len() >= 8;
    
    let has_upper = password.chars().any(|c| c.is_uppercase());
    let has_lower = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_digit(10));

    if has_min_length && has_upper && has_lower && has_digit {
        Ok(())
    } else {
        let mut err = validator::ValidationError::new("password_strength");
        err.message = Some(
            "Password must be at least 8 characters long and contain at least one uppercase letter, one lowercase letter, and one digit".into(),
        );
        Err(err)
    }
}

/// Validates that a quantity is positive (> 0).
pub fn validate_positive_quantity(qty: i32) -> Result<(), validator::ValidationError> {
    if qty > 0 {
        Ok(())
    } else {
        let mut err = validator::ValidationError::new("positive_quantity");
        err.message = Some("Quantity must be greater than 0".into());
        Err(err)
    }
}
 
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_indonesian_phones() {
        assert!(validate_indonesian_phone("+6281234567890").is_ok());
        assert!(validate_indonesian_phone("081234567890").is_ok());
        assert!(validate_indonesian_phone("+628999999999").is_ok());
    }

        #[test]
    fn test_invalid_indonesian_phones() {
        assert!(validate_indonesian_phone("12345").is_err());
        assert!(validate_indonesian_phone("+1234567890").is_err());
        assert!(validate_indonesian_phone("091234567890").is_err()); // wrong prefix
        assert!(validate_indonesian_phone("").is_err());
    }

    #[test]
    fn test_valid_skus() {
        assert!(validate_sku("WMS-PROD-001").is_ok());
        assert!(validate_sku("AB12").is_ok());
        assert!(validate_sku("PRODUCT-CAT-001").is_ok());
    }

    #[test]
    fn test_invalid_skus() {
        assert!(validate_sku("wms-prod-001").is_err()); // lowercase
        assert!(validate_sku("A").is_err());             // too short
        assert!(validate_sku("PROD 001").is_err());      // space not allowed
        assert!(validate_sku("-PROD-001").is_err());     // cannot start with hyphen
    }

    #[test]
    fn test_password_strength() {
        assert!(validate_password_strength("Password1").is_ok());
        assert!(validate_password_strength("MyPass123!").is_ok());
        assert!(validate_password_strength("password1").is_err()); // no uppercase
        assert!(validate_password_strength("PASSWORD1").is_err()); // no lowercase
        assert!(validate_password_strength("Password").is_err());  // no digit
    }

    #[test]
    fn test_positive_quantity() {
        assert!(validate_positive_quantity(1).is_ok());
        assert!(validate_positive_quantity(100).is_ok());
        assert!(validate_positive_quantity(0).is_err());
        assert!(validate_positive_quantity(-1).is_err());
    }
}