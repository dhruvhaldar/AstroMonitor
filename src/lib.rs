pub mod models;

pub use models::{
    CelestialCoordinates, PowerData, StarTrackerReading, Subsystem, TelemetryPacket,
    TelemetryPayload, ThermalData,
};

pub mod parser;
pub use parser::{Parser, ParserError};

pub mod monitor;
pub use monitor::{Alert, AlertLevel, Monitor};

pub mod simulation;

pub mod gui;

#[cfg(test)]
mod lib_tests;
