//! src/auth/token.rs — JWT creation and validation with HS256.
//!

use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};

use serde::{Deserialize, Serialize};

use crate::enums::Role;

/// JWT claims stored inside the token.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub role: Role,
    pub exp: usize,
    pub iat: usize,
}

/// Create a signed JWT embedding the user's id, email, and role.
pub fn encode_token(
    user_id: &str,
    email: &str,
    role: &Role,
    secret: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    let expiry_hours: i64 = std::env::var("JWT_EXPIRY_HOURS")
        .unwrap_or_else(|_| "24".to_string())
        .parse()
        .unwrap_or(24);

    let now = Utc::now();
    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        role: role.clone(),
        iat: now.timestamp() as usize,
        exp: (now + chrono::Duration::hours(expiry_hours)).timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

/// Decode and validate a JWT. Returns the claims if the token is valid.
pub fn decode_token(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SECRET: &str = "test-secret-for-unit-tests-only";

    #[test]
    fn when_valid_token_should_decode() {
        let token = encode_token("user-1", "test@example.com", &Role::Admin, TEST_SECRET).unwrap();
        let claims = decode_token(&token, TEST_SECRET).unwrap();
        assert_eq!(claims.sub, "user-1");
        assert_eq!(claims.role, Role::Admin);
    }

    #[test]
    fn when_wrong_secret_should_fail() {
        let token = encode_token("user-1", "test@example.com", &Role::Admin, TEST_SECRET).unwrap();
        assert!(decode_token(&token, "wrong-secret").is_err());
    }
}

// End of File
