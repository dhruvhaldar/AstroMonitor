#[cfg(test)]
mod tests {
    use astro_monitor::models::{CelestialCoordinates, StarTrackerReading, Subsystem, TelemetryPacket, TelemetryPayload};
    use std::borrow::Cow;

    #[test]
    fn test_csv_bypass() {
        let malicious_id_with_tab = "\x08=cmd|' /C calc'!A0";
        let packet_with_tab = TelemetryPacket {
            timestamp: 1234567890,
            subsystem: Subsystem::StarTracker,
            payload: TelemetryPayload::StarTracker(StarTrackerReading {
                target_id: Some(Cow::Borrowed(malicious_id_with_tab)),
                coordinates: CelestialCoordinates {
                    right_ascension: 0.0,
                    declination: 0.0,
                },
                confidence: 1.0,
            }),
        };

        // Format the log packet
    }
}
