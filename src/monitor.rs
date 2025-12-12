use crate::models::{TelemetryPacket, TelemetryPayload};
use serde::Serialize;

#[derive(Debug, Serialize, PartialEq)]
pub enum AlertLevel {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct Alert {
    pub level: AlertLevel,
    pub message: String,
    pub timestamp: u64,
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

    pub fn analyze(&self, packet: &TelemetryPacket) -> Option<Alert> {
        match &packet.payload {
            TelemetryPayload::Power(data) => {
                if data.battery_level < self.min_battery_level {
                    return Some(Alert {
                        level: AlertLevel::Critical,
                        message: format!(
                            "Low Battery: {:.2}% (Threshold: {:.2}%)",
                            data.battery_level, self.min_battery_level
                        ),
                        timestamp: packet.timestamp,
                    });
                }
            }
            TelemetryPayload::Thermal(data) => {
                if data.temp_celsius > self.max_temp_celsius {
                    return Some(Alert {
                        level: AlertLevel::Warning,
                        message: format!(
                            "High Temperature: {:.2}C (Threshold: {:.2}C)",
                            data.temp_celsius, self.max_temp_celsius
                        ),
                        timestamp: packet.timestamp,
                    });
                }
            }
            TelemetryPayload::StarTracker(data) => {
                if data.confidence < self.min_star_confidence {
                    return Some(Alert {
                        level: AlertLevel::Info,
                        message: format!(
                            "Low Star Confidence: {:.2} (Threshold: {:.2})",
                            data.confidence, self.min_star_confidence
                        ),
                        timestamp: packet.timestamp,
                    });
                }
            }
            _ => {}
        }
        None
    }
}
