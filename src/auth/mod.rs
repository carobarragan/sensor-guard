//! src/auth/mod.rs — Authentication utilities: password hashing and JWT tokens.
//!

pub mod password;
pub mod token;

pub use token::{decode_token, encode_token};

// End of File
