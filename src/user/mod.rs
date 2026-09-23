//! src/user/mod.rs — User entity: registration, login, and account management.
//!

mod user_controller;
pub mod user_repository;
pub mod user_request;
pub mod user_response;
pub mod user_service;
pub mod user_struct;

pub use user_controller::{login_user, register_user};
pub use user_repository::UserRepository;
pub use user_service::UserService;

use std::sync::Arc;

pub type DynUserRepo = Arc<dyn UserRepository>;
pub type DynUserService = Arc<dyn UserService>;

// End of File
