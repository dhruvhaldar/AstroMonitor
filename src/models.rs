use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Subsystem {
    Power,
    Thermal,
    Aocs, // Attitude and Orbit Control System
    StarTracker,
    Propulsion,
    Science,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AocsMode {
    Safe,
    Pointing,
    Detumbling,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AocsData {
    pub mode: AocsMode,
    pub quaternion: [f64; 4],       // x, y, z, w
    pub angular_velocity: [f64; 3], // x, y, z
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EngineStatus {
    Off,
    On,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PropulsionData {
    pub fuel_level: f64, // Percentage
    pub pressure: f64,   // Bar
    pub engine_status: EngineStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScienceData {
    pub instrument_id: String,
    pub wavelength: f64,    // nm
    pub exposure_time: u32, // ms
    pub data_size: u64,     // bytes
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
    Aocs(AocsData),
    StarTracker(StarTrackerReading),
    Propulsion(PropulsionData),
    Science(ScienceData),
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TelemetryPacket {
    pub timestamp: u64, // Unix timestamp
    pub subsystem: Subsystem,
    pub payload: TelemetryPayload,
}
