//! src/user/user_response.rs — Response DTOs for user endpoints.
//!

use serde::Serialize;

use crate::enums::Role;

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub token_type: String,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
    pub role: Role,
}

// End of File
