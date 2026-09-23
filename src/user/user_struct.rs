//! src/user/user_struct.rs — User database model.
//!

#[derive(Debug, sqlx::FromRow)]
#[allow(dead_code)]
pub struct User {
    pub id: String,
    pub email: String,
    pub password_hash: String,
    pub role: String,
    pub created_at: String,
}

// End of File
