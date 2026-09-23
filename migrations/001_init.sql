-- Users table
-- SECURITY: passwords are stored as Argon2id hashes, never plaintext.
CREATE TABLE IF NOT EXISTS users (
    id          TEXT PRIMARY KEY NOT NULL,
    email       TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    role        TEXT NOT NULL DEFAULT 'operator' CHECK (role IN ('admin', 'operator')),
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Sensor readings table
CREATE TABLE IF NOT EXISTS sensor_readings (
    id          TEXT PRIMARY KEY NOT NULL,
    sensor_name TEXT NOT NULL,
    value       REAL NOT NULL,
    unit        TEXT NOT NULL,
    recorded_at TEXT NOT NULL DEFAULT (datetime('now')),
    created_by  TEXT NOT NULL,
    FOREIGN KEY (created_by) REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_sensor_readings_sensor_name ON sensor_readings(sensor_name);
CREATE INDEX IF NOT EXISTS idx_sensor_readings_created_by ON sensor_readings(created_by);
