//! src/auth/password.rs — Password hashing and verification using Argon2id.
//!

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

/// Hash a plaintext password with Argon2id and a random salt.
pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default(); // Argon2id variant with safe defaults
    let hash = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

/// Verify a plaintext password against a stored Argon2id hash.
/// Returns Ok(()) on match, Err on mismatch or invalid hash.
pub fn verify_password(password: &str, hash: &str) -> Result<(), argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(hash)?;
    Argon2::default().verify_password(password.as_bytes(), &parsed_hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn when_correct_password_should_verify() {
        let hash = hash_password("secure_password_123").unwrap();
        assert!(verify_password("secure_password_123", &hash).is_ok());
    }

    #[test]
    fn when_wrong_password_should_fail() {
        let hash = hash_password("secure_password_123").unwrap();
        assert!(verify_password("wrong_password", &hash).is_err());
    }

    #[test]
    fn when_hashing_same_password_twice_should_produce_different_hashes() {
        let h1 = hash_password("same_password").unwrap();
        let h2 = hash_password("same_password").unwrap();
        // Different salts → different hashes (prevents rainbow table attacks)
        assert_ne!(h1, h2);
    }
}

// End of File
