//! src/reading/reading_struct.rs — SensorReading database model.
//!

use serde::Serialize;

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct SensorReading {
    pub id: String,
    pub sensor_name: String,
    pub value: f64,
    pub unit: String,
    pub recorded_at: String,
    pub created_by: String,
}

// End of File
