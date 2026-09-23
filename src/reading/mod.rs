//! src/reading/mod.rs — Sensor reading entity: ingest and query sensor data.
//!

mod reading_controller;
pub mod reading_repository;
pub mod reading_request;
pub mod reading_service;
pub mod reading_struct;

pub use reading_controller::{create_reading, get_reading, list_readings};
pub use reading_repository::ReadingRepository;
pub use reading_service::ReadingService;

use std::sync::Arc;

pub type DynReadingRepo = Arc<dyn ReadingRepository>;
pub type DynReadingService = Arc<dyn ReadingService>;

// End of File
