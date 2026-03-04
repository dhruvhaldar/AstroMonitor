#[cfg(test)]
mod tests {
    use astro_monitor::models::{CelestialCoordinates, StarTrackerReading, Subsystem, TelemetryPacket, TelemetryPayload};
    use std::borrow::Cow;
    use astro_monitor::gui::AstroMonitorApp;

    #[test]
    fn test_csv_bypass() {
        let malicious_id_with_tab = "\x0B=cmd|' /C calc'!A0";
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
        // ... Wait, format_log_packet is private, we can't call it easily from here if it's not exposed.
    }
}
