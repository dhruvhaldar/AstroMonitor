use crate::models::{TelemetryPacket, TelemetryPayload};
use serde::Serialize;
use std::fmt;

#[derive(Debug, Serialize, PartialEq, Clone, Copy)]
pub enum AlertLevel {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Serialize, PartialEq, Clone, Copy)]
pub enum AlertCondition {
    LowBattery { value: f64, threshold: f64 },
    HighTemperature { value: f64, threshold: f64 },
    LowStarConfidence { value: f64, threshold: f64 },
}

impl fmt::Display for AlertCondition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AlertCondition::LowBattery { value, threshold } => write!(
                f,
                "Low Battery: {:.2}% (Threshold: {:.2}%)",
                value, threshold
            ),
            AlertCondition::HighTemperature { value, threshold } => write!(
                f,
                "High Temperature: {:.2}C (Threshold: {:.2}C)",
                value, threshold
            ),
            AlertCondition::LowStarConfidence { value, threshold } => write!(
                f,
                "Low Star Confidence: {:.2} (Threshold: {:.2})",
                value, threshold
            ),
        }
    }
}

#[derive(Debug, Serialize, PartialEq, Clone, Copy)]
pub struct MonitorEvent {
    pub level: AlertLevel,
    pub condition: AlertCondition,
    pub timestamp: u64,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct Alert {
    pub level: AlertLevel,
    pub message: String,
    pub timestamp: u64,
}

impl From<MonitorEvent> for Alert {
    fn from(event: MonitorEvent) -> Self {
        Self {
            level: event.level,
            message: event.condition.to_string(),
            timestamp: event.timestamp,
        }
    }
}

pub struct Monitor {
    // Thresholds
    pub min_battery_level: f64,
    pub max_temp_celsius: f64,
    pub min_star_confidence: f64,
}

impl Default for Monitor {
    fn default() -> Self {
        Self {
            min_battery_level: 20.0,
            max_temp_celsius: 80.0,
            min_star_confidence: 0.8,
        }
    }
}

impl Monitor {
    pub fn new(min_battery_level: f64, max_temp_celsius: f64, min_star_confidence: f64) -> Self {
        Self {
            min_battery_level,
            max_temp_celsius,
            min_star_confidence,
        }
    }

    pub fn check(&self, packet: &TelemetryPacket<'_>) -> Option<MonitorEvent> {
        match &packet.payload {
            TelemetryPayload::Power(data) => {
                if data.battery_level < self.min_battery_level {
                    return Some(MonitorEvent {
                        level: AlertLevel::Critical,
                        condition: AlertCondition::LowBattery {
                            value: data.battery_level,
                            threshold: self.min_battery_level,
                        },
                        timestamp: packet.timestamp,
                    });
                }
            }
            TelemetryPayload::Thermal(data) => {
                if data.temp_celsius > self.max_temp_celsius {
                    return Some(MonitorEvent {
                        level: AlertLevel::Warning,
                        condition: AlertCondition::HighTemperature {
                            value: data.temp_celsius,
                            threshold: self.max_temp_celsius,
                        },
                        timestamp: packet.timestamp,
                    });
                }
            }
            TelemetryPayload::StarTracker(data) => {
                if data.confidence < self.min_star_confidence {
                    return Some(MonitorEvent {
                        level: AlertLevel::Info,
                        condition: AlertCondition::LowStarConfidence {
                            value: data.confidence,
                            threshold: self.min_star_confidence,
                        },
                        timestamp: packet.timestamp,
                    });
                }
            }
            _ => {}
        }
        None
    }

    pub fn analyze(&self, packet: &TelemetryPacket<'_>) -> Option<Alert> {
        self.check(packet).map(Alert::from)
    }
}
