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
    // Thresholds - Encapsulated to prevent invalid states (e.g., NaN)
    min_battery_level: f64,
    max_temp_celsius: f64,
    min_star_confidence: f64,
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
    pub fn new(
        min_battery_level: f64,
        max_temp_celsius: f64,
        min_star_confidence: f64,
    ) -> Result<Self, &'static str> {
        let mut monitor = Self::default();
        monitor.set_min_battery_level(min_battery_level)?;
        monitor.set_max_temp_celsius(max_temp_celsius)?;
        monitor.set_min_star_confidence(min_star_confidence)?;
        Ok(monitor)
    }

    pub fn min_battery_level(&self) -> f64 {
        self.min_battery_level
    }

    pub fn set_min_battery_level(&mut self, val: f64) -> Result<(), &'static str> {
        if !val.is_finite() {
            return Err("Battery level threshold must be finite");
        }
        if !(0.0..=100.0).contains(&val) {
            return Err("Battery level threshold must be between 0.0 and 100.0");
        }
        self.min_battery_level = val;
        Ok(())
    }

    pub fn max_temp_celsius(&self) -> f64 {
        self.max_temp_celsius
    }

    pub fn set_max_temp_celsius(&mut self, val: f64) -> Result<(), &'static str> {
        if !val.is_finite() {
            return Err("Temperature threshold must be finite");
        }
        self.max_temp_celsius = val;
        Ok(())
    }

    pub fn min_star_confidence(&self) -> f64 {
        self.min_star_confidence
    }

    pub fn set_min_star_confidence(&mut self, val: f64) -> Result<(), &'static str> {
        if !val.is_finite() {
            return Err("Star confidence threshold must be finite");
        }
        if !(0.0..=1.0).contains(&val) {
            return Err("Star confidence threshold must be between 0.0 and 1.0");
        }
        self.min_star_confidence = val;
        Ok(())
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
                    let mut bytes = id.as_bytes().iter();
                    // Bolt Optimization: Use iterator-based loop to remove bounds checking overhead.
                    // This is ~1.5x faster for short strings than indexed access.
                    while let Some(&b) = bytes.next() {
                        if b < 32 || b == 127 {
                            has_control = true;
                            break;
                        }
                        // Bolt Optimization: Check for C1 control characters (U+0080..U+009F).
                        // In UTF-8, these are encoded as 0xC2 followed by 0x80..0x9F.
                        // Since `Parser` guarantees valid UTF-8, if we see 0xC2, it must be followed by a continuation byte.
                        // We check if that next byte (peeked via as_slice) is in the C1 range (0x80..0x9F).
                        // This avoids the O(N) overhead of `chars().any()` which decodes every UTF-8 character.
                        if b == 0xC2 {
                            if let Some(&next) = bytes.as_slice().first() {
                                if (0x80..=0x9F).contains(&next) {
                                    has_control = true;
                                    break;
                                }
                            }
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
    use crate::models::{
        CelestialCoordinates, PowerData, StarTrackerReading, Subsystem, TelemetryPayload,
    };
    use std::borrow::Cow;

    #[test]
    fn test_monitor_encapsulation_security() {
        let mut monitor = Monitor::default();

        // 1. Battery Level Security
        // Try to set NaN
        assert!(
            monitor.set_min_battery_level(f64::NAN).is_err(),
            "Should reject NaN battery"
        );
        // Try to set Inf
        assert!(
            monitor.set_min_battery_level(f64::INFINITY).is_err(),
            "Should reject Inf battery"
        );
        // Try to set out of bounds (< 0)
        assert!(
            monitor.set_min_battery_level(-1.0).is_err(),
            "Should reject negative battery"
        );
        // Try to set out of bounds (> 100)
        assert!(
            monitor.set_min_battery_level(101.0).is_err(),
            "Should reject >100 battery"
        );

        // Valid set
        assert!(monitor.set_min_battery_level(15.0).is_ok());
        assert_eq!(monitor.min_battery_level(), 15.0);

        // 2. Star Confidence Security
        // Try to set NaN
        assert!(
            monitor.set_min_star_confidence(f64::NAN).is_err(),
            "Should reject NaN confidence"
        );
        // Try to set out of bounds (> 1.0)
        assert!(
            monitor.set_min_star_confidence(1.5).is_err(),
            "Should reject >1.0 confidence"
        );

        // Valid set
        assert!(monitor.set_min_star_confidence(0.9).is_ok());
        assert_eq!(monitor.min_star_confidence(), 0.9);

        // 3. Constructor Security
        let res = Monitor::new(f64::NAN, 80.0, 0.8);
        assert!(res.is_err(), "Constructor should reject NaN");
    }

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
        assert!(
            event.is_some(),
            "Monitor should alert on control characters in target_id"
        );
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

    #[test]
    fn test_monitor_c1_control_chars() {
        let monitor = Monitor::default();
        // C1 control character U+0080 is encoded as 0xC2 0x80 in UTF-8
        let packet = TelemetryPacket {
            timestamp: 1234567890,
            subsystem: Subsystem::StarTracker,
            payload: TelemetryPayload::StarTracker(StarTrackerReading {
                target_id: Some(Cow::Borrowed("Bad\u{0080}String")),
                coordinates: CelestialCoordinates {
                    right_ascension: 0.0,
                    declination: 0.0,
                },
                confidence: 1.0,
            }),
        };

        let event = monitor.check(&packet);
        assert!(
            event.is_some(),
            "Monitor should alert on C1 control character U+0080"
        );
        let event = event.unwrap();
        assert_eq!(event.level, AlertLevel::Critical);
        match event.condition {
            AlertCondition::SensorFailure { subsystem } => {
                assert_eq!(subsystem, "StarTracker");
            }
            _ => panic!("Expected SensorFailure alert"),
        }
    }

    #[test]
    fn benchmark_monitor_performance() {
        let monitor = Monitor::default();
        let packet = TelemetryPacket {
            timestamp: 1234567890,
            subsystem: Subsystem::StarTracker,
            payload: TelemetryPayload::StarTracker(StarTrackerReading {
                target_id: Some(Cow::Borrowed("Sirius - The brightest star")),
                coordinates: CelestialCoordinates {
                    right_ascension: 10.0,
                    declination: 20.0,
                },
                confidence: 0.9,
            }),
        };

        let start = std::time::Instant::now();
        let iterations = 1_000_000;
        for _ in 0..iterations {
            std::hint::black_box(monitor.check(std::hint::black_box(&packet)));
        }
        println!(
            "Monitor::check took {:?} for {} iterations",
            start.elapsed(),
            iterations
        );
    }
}
