use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Subsystem {
    Power,
    Thermal,
    Aocs, // Attitude and Orbit Control System
    StarTracker,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CelestialCoordinates {
    pub right_ascension: f64, // degrees
    pub declination: f64,     // degrees
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PowerData {
    pub voltage: f64,       // Volts
    pub current: f64,       // Amperes
    pub battery_level: f64, // Percentage
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThermalData {
    pub temp_celsius: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StarTrackerReading {
    pub target_id: Option<String>,
    pub coordinates: CelestialCoordinates,
    pub confidence: f64, // 0.0 to 1.0
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TelemetryPayload {
    Power(PowerData),
    Thermal(ThermalData),
    StarTracker(StarTrackerReading),
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TelemetryPacket {
    pub timestamp: u64, // Unix timestamp
    pub subsystem: Subsystem,
    pub payload: TelemetryPayload,
}
