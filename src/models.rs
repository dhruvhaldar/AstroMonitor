use std::borrow::Cow;

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
pub struct StarTrackerReading<'a> {
    #[serde(borrow)]
    pub target_id: Option<Cow<'a, str>>,
    pub coordinates: CelestialCoordinates,
    pub confidence: f64, // 0.0 to 1.0
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TelemetryPayload<'a> {
    Power(PowerData),
    Thermal(ThermalData),
    #[serde(borrow)]
    StarTracker(StarTrackerReading<'a>),
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TelemetryPacket<'a> {
    pub timestamp: u64, // Unix timestamp
    pub subsystem: Subsystem,
    #[serde(borrow)]
    pub payload: TelemetryPayload<'a>,
}
