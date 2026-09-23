//! src/reading/reading_request.rs — Request DTOs for reading endpoints.
//!

use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateReadingRequest {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Sensor name is required (max 100 chars)"
    ))]
    pub sensor_name: String,

    pub value: f64,

    #[validate(length(min = 1, max = 20, message = "Unit is required (max 20 chars)"))]
    pub unit: String,
}

// End of File
