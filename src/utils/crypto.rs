use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::{SaltString, rand_core::OsRng}};

use crate::errors::AppError;

// --- CPU-intensive, better run in a blocking thread ---
pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to hash password: {}", e)))
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Invalid password hash: {}", e)))?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password_produces_argon2_hash(){
        let hash = hash_password("secret123").unwrap();

        assert!(hash.starts_with("$argon2"), "Expected argon2 hash, get: {}", hash)
    }

    #[test]
    fn test_hash_password_is_not_plaintext(){
        let password = "MyPassword123";
        let hash = hash_password(password).unwrap();
        assert_ne!(hash, password)
    }

    #[test]
    fn test_same_password_produces_different_hashes(){
        let hash1 = hash_password("password").unwrap();
        let hash2 = hash_password("password").unwrap();

        assert_ne!(hash1, hash2, "Two hashes of same password should differ duo to random salt")
    }

    #[test]
    fn test_verify_correct_password(){
        let password = "correct_password";
        let hash = hash_password(password).unwrap();
        let result = verify_password(password, &hash).unwrap();

        assert!(result)
    }

    #[test]
    fn test_verify_wrong_password(){
        let hash = hash_password("correct_password").unwrap();
        let result = verify_password("wrong_password", &hash).unwrap();

        assert!(!result)
    }

    #[test]
    fn test_verify_empty_password_against_hash(){
        let hash = hash_password("notempty").unwrap();
        let result = verify_password("", &hash).unwrap();

        assert!(!result)
    }

    #[test]
    fn test_verify_invalid_hash_returns_error(){
        let result = verify_password("password", "not_a_valid_hash");
        
        assert!(result.is_err())
    }
    
}