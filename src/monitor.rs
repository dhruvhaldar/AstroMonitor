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
    SensorFailure { subsystem: &'static str },
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
            AlertCondition::SensorFailure { subsystem } => {
                write!(f, "Sensor Failure: {} reports invalid data", subsystem)
            }
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
                if !data.voltage.is_finite()
                    || !data.current.is_finite()
                    || !data.battery_level.is_finite()
                {
                    return Some(MonitorEvent {
                        level: AlertLevel::Critical,
                        condition: AlertCondition::SensorFailure { subsystem: "Power" },
                        timestamp: packet.timestamp,
                    });
                }
                // Security Check: Validate Battery Level Range (0.0 - 100.0)
                if !(0.0..=100.0).contains(&data.battery_level) {
                    return Some(MonitorEvent {
                        level: AlertLevel::Critical,
                        condition: AlertCondition::SensorFailure { subsystem: "Power" },
                        timestamp: packet.timestamp,
                    });
                }
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
                if !data.temp_celsius.is_finite() {
                    return Some(MonitorEvent {
                        level: AlertLevel::Critical,
                        condition: AlertCondition::SensorFailure {
                            subsystem: "Thermal",
                        },
                        timestamp: packet.timestamp,
                    });
                }
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
                if !data.confidence.is_finite()
                    || !data.coordinates.right_ascension.is_finite()
                    || !data.coordinates.declination.is_finite()
                {
                    return Some(MonitorEvent {
                        level: AlertLevel::Critical,
                        condition: AlertCondition::SensorFailure {
                            subsystem: "StarTracker",
                        },
                        timestamp: packet.timestamp,
                    });
                }
                // Security Check: Validate Confidence Range (0.0 - 1.0)
                // Security Check: Validate Celestial Coordinates
                if !(0.0..=360.0).contains(&data.coordinates.right_ascension)
                    || !(-90.0..=90.0).contains(&data.coordinates.declination)
                {
                    return Some(MonitorEvent {
                        level: AlertLevel::Critical,
                        condition: AlertCondition::SensorFailure {
                            subsystem: "StarTracker",
                        },
                        timestamp: packet.timestamp,
                    });
                }

                if !(0.0..=1.0).contains(&data.confidence) {
                    return Some(MonitorEvent {
                        level: AlertLevel::Critical,
                        condition: AlertCondition::SensorFailure {
                            subsystem: "StarTracker",
                        },
                        timestamp: packet.timestamp,
                    });
                }
                // Security Check: Ensure Target ID is a valid printable string (no control characters)
                if let Some(id) = &data.target_id {
                    let mut has_control = false;
                    for (i, &b) in id.as_bytes().iter().enumerate() {
                        if b < 32 || b == 127 {
                            has_control = true;
                            break;
                        }
                        if b >= 128 {
                            // Bolt Optimization: Fall back to full UTF-8 char check for non-ASCII
                            // to ensure correct handling of Unicode control characters.
                            // Start scan from current byte index to avoid rescanning ASCII prefix.
                            // SAFETY: i is a valid byte index from the iterator, and since we iterate
                            // bytes, slicing at i is safe if we are at an ASCII byte boundary,
                            // but wait, if b >= 128, b is the *start* of a multibyte sequence.
                            // So i is a valid char boundary.
                            if id[i..].chars().any(|c| c.is_control()) {
                                has_control = true;
                            }
                            break;
                        }
                    }

                    if has_control {
                         return Some(MonitorEvent {
                            level: AlertLevel::Critical,
                            condition: AlertCondition::SensorFailure {
                                subsystem: "StarTracker",
                            },
                            timestamp: packet.timestamp,
                        });
                    }
                }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{PowerData, Subsystem, TelemetryPayload, StarTrackerReading, CelestialCoordinates};
    use std::borrow::Cow;

    #[test]
    fn test_monitor_alerts_on_nan() {
        let monitor = Monitor::default();
        let packet = TelemetryPacket {
            timestamp: 1234567890,
            subsystem: Subsystem::Power,
            payload: TelemetryPayload::Power(PowerData {
                voltage: 28.0,
                current: 2.5,
                battery_level: f64::NAN, // Invalid battery
            }),
        };

        let event = monitor.check(&packet);
        assert!(event.is_some(), "Monitor should alert on NaN battery level");
        let event = event.unwrap();
        assert_eq!(event.level, AlertLevel::Critical);
        match event.condition {
            AlertCondition::SensorFailure { subsystem } => {
                assert_eq!(subsystem, "Power");
            }
            _ => panic!("Expected SensorFailure alert"),
        }
    }

    #[test]
    fn test_monitor_detects_invalid_string_id() {
        let monitor = Monitor::default();
        let packet = TelemetryPacket {
            timestamp: 1234567890,
            subsystem: Subsystem::StarTracker,
            payload: TelemetryPayload::StarTracker(StarTrackerReading {
                target_id: Some(Cow::Borrowed("Sirius\nB")), // Invalid newline
                coordinates: CelestialCoordinates {
                    right_ascension: 10.0,
                    declination: 20.0,
                },
                confidence: 0.9,
            }),
        };

        let event = monitor.check(&packet);
        assert!(event.is_some(), "Monitor should alert on control characters in target_id");
        let event = event.unwrap();
        assert_eq!(event.level, AlertLevel::Critical);
        match event.condition {
            AlertCondition::SensorFailure { subsystem } => {
                assert_eq!(subsystem, "StarTracker");
            }
            _ => panic!("Expected SensorFailure alert, got {:?}", event.condition),
        }
    }

    #[test]
    fn test_monitor_negative_confidence_security_gap() {
        let monitor = Monitor::default();
        let packet = TelemetryPacket {
            timestamp: 1234567890,
            subsystem: Subsystem::StarTracker,
            payload: TelemetryPayload::StarTracker(StarTrackerReading {
                target_id: Some(Cow::Borrowed("Sirius")),
                coordinates: CelestialCoordinates {
                    right_ascension: 10.0,
                    declination: 20.0,
                },
                confidence: -1.0, // Invalid negative confidence
            }),
        };

        let event = monitor.check(&packet);
        assert!(event.is_some());
        let event = event.unwrap();

        // FIX VERIFIED:
        // Negative confidence should now trigger "Sensor Failure" (Critical).
        assert_eq!(event.level, AlertLevel::Critical);
        match event.condition {
            AlertCondition::SensorFailure { subsystem } => {
                assert_eq!(subsystem, "StarTracker");
            }
            _ => panic!("Expected SensorFailure alert"),
        }
    }

    #[test]
    fn test_monitor_invalid_battery_security_gap() {
        let monitor = Monitor::default();
        let packet = TelemetryPacket {
            timestamp: 1234567890,
            subsystem: Subsystem::Power,
            payload: TelemetryPayload::Power(PowerData {
                voltage: 28.0,
                current: 2.5,
                battery_level: 200.0, // Invalid > 100%
            }),
        };

        let event = monitor.check(&packet);
        assert!(event.is_some());
        let event = event.unwrap();

        // FIX VERIFIED:
        // Battery > 100% should trigger "Sensor Failure" (Critical).
        assert_eq!(event.level, AlertLevel::Critical);
        match event.condition {
            AlertCondition::SensorFailure { subsystem } => {
                assert_eq!(subsystem, "Power");
            }
            _ => panic!("Expected SensorFailure alert"),
        }
    }

    #[test]
    fn test_monitor_invalid_coordinates() {
        let monitor = Monitor::default();
        // Case 1: Invalid Right Ascension (> 360.0)
        let packet_ra = TelemetryPacket {
            timestamp: 1234567890,
            subsystem: Subsystem::StarTracker,
            payload: TelemetryPayload::StarTracker(StarTrackerReading {
                target_id: Some(Cow::Borrowed("TestStar")),
                coordinates: CelestialCoordinates {
                    right_ascension: 400.0, // Invalid
                    declination: 0.0,
                },
                confidence: 1.0,
            }),
        };

        let event_ra = monitor.check(&packet_ra);
        assert!(event_ra.is_some(), "Monitor should alert on RA > 360.0");
        let event_ra = event_ra.unwrap();
        assert_eq!(event_ra.level, AlertLevel::Critical);
        match event_ra.condition {
            AlertCondition::SensorFailure { subsystem } => {
                assert_eq!(subsystem, "StarTracker");
            }
            _ => panic!("Expected SensorFailure alert for invalid RA"),
        }

        // Case 2: Invalid Declination (< -90.0)
        let packet_dec = TelemetryPacket {
            timestamp: 1234567890,
            subsystem: Subsystem::StarTracker,
            payload: TelemetryPayload::StarTracker(StarTrackerReading {
                target_id: Some(Cow::Borrowed("TestStar")),
                coordinates: CelestialCoordinates {
                    right_ascension: 0.0,
                    declination: -100.0, // Invalid
                },
                confidence: 1.0,
            }),
        };

        let event_dec = monitor.check(&packet_dec);
        assert!(event_dec.is_some(), "Monitor should alert on Dec < -90.0");
        let event_dec = event_dec.unwrap();
        assert_eq!(event_dec.level, AlertLevel::Critical);
        match event_dec.condition {
            AlertCondition::SensorFailure { subsystem } => {
                assert_eq!(subsystem, "StarTracker");
            }
            _ => panic!("Expected SensorFailure alert for invalid Dec"),
        }
    }
}
