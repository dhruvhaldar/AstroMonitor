#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::*;
    use crate::monitor::*;
    use crate::parser::*;

    #[test]
    fn test_parse_power() {
        let mut data = Vec::new();
        let timestamp: u64 = 1627849200;
        data.extend_from_slice(&timestamp.to_be_bytes()); // Timestamp
        data.push(0); // Subsystem: Power
        data.extend_from_slice(&(24u16).to_be_bytes()); // Len (ignored by logic but placeholder)

        let voltage = 28.5f64;
        let current = 2.0f64;
        let battery = 95.0f64;

        data.extend_from_slice(&voltage.to_be_bytes());
        data.extend_from_slice(&current.to_be_bytes());
        data.extend_from_slice(&battery.to_be_bytes());

        let result = Parser::parse(&data).unwrap();

        assert_eq!(result.timestamp, timestamp);
        assert_eq!(result.subsystem, Subsystem::Power);

        if let TelemetryPayload::Power(p) = result.payload {
            assert_eq!(p.voltage, voltage);
            assert_eq!(p.current, current);
            assert_eq!(p.battery_level, battery);
        } else {
            panic!("Wrong payload type");
        }
    }

    #[test]
    fn test_parse_star_tracker() {
        let mut data = Vec::new();
        let timestamp: u64 = 1627849200;
        data.extend_from_slice(&timestamp.to_be_bytes());
        data.push(3); // Subsystem: StarTracker
        data.extend_from_slice(&(0u16).to_be_bytes()); // Placeholder Len

        let ra = 120.5f64;
        let dec = -30.2f64;
        let conf = 0.99f64;
        let target_id = "AlphaCentauri";

        data.extend_from_slice(&ra.to_be_bytes());
        data.extend_from_slice(&dec.to_be_bytes());
        data.extend_from_slice(&conf.to_be_bytes());
        data.push(target_id.len() as u8);
        data.extend_from_slice(target_id.as_bytes());

        let result = Parser::parse(&data).unwrap();

        if let TelemetryPayload::StarTracker(s) = result.payload {
            assert_eq!(s.coordinates.right_ascension, ra);
            assert_eq!(s.coordinates.declination, dec);
            assert_eq!(s.confidence, conf);
            assert_eq!(s.target_id, Some(target_id.to_string()));
        } else {
            panic!("Wrong payload type");
        }
    }

    #[test]
    fn test_monitor_alerts() {
        let monitor = Monitor::default();

        // Critical Battery
        let packet = TelemetryPacket {
            timestamp: 100,
            subsystem: Subsystem::Power,
            payload: TelemetryPayload::Power(PowerData {
                voltage: 20.0,
                current: 1.0,
                battery_level: 10.0, // Below 20.0 default
            }),
        };

        let alert = monitor.analyze(&packet);
        assert!(alert.is_some());
        assert_eq!(alert.unwrap().level, AlertLevel::Critical);

        // Good Battery
        let packet_good = TelemetryPacket {
            timestamp: 101,
            subsystem: Subsystem::Power,
            payload: TelemetryPayload::Power(PowerData {
                voltage: 28.0,
                current: 1.0,
                battery_level: 50.0,
            }),
        };
        assert!(monitor.analyze(&packet_good).is_none());
    }
}
