use std::borrow::Cow;

use crate::models::{
    CelestialCoordinates, PowerData, StarTrackerReading, Subsystem, TelemetryPacket,
    TelemetryPayload, ThermalData,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Buffer too short")]
    BufferTooShort,
    #[error("Invalid subsystem ID: {0}")]
    InvalidSubsystem(u8),
    #[error("UTF-8 error")]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error("Checksum mismatch")]
    ChecksumMismatch,
    #[error("Unknown error")]
    Unknown,
}

pub struct Parser;

impl Parser {
    /// Parses a raw telemetry packet and validates its checksum.
    pub fn parse<'a>(data: &'a [u8]) -> Result<TelemetryPacket<'a>, ParserError> {
        Self::parse_internal(data, true)
    }

    /// Parses a raw telemetry packet WITHOUT validating the checksum.
    ///
    /// Unlike `parse`, this function skips the O(N) checksum validation for performance
    /// when the data source is trusted (e.g. internal simulation buffers).
    /// However, it still validates UTF-8 strings to ensure memory safety.
    pub fn parse_trusted<'a>(data: &'a [u8]) -> Result<TelemetryPacket<'a>, ParserError> {
        Self::parse_internal(data, false)
    }

    fn parse_internal<'a>(
        data: &'a [u8],
        verify_checksum: bool,
    ) -> Result<TelemetryPacket<'a>, ParserError> {
        let mut offset = 0;

        if data.len() < 11 {
            return Err(ParserError::BufferTooShort);
        }

        // Timestamp (8 bytes)
        let timestamp_bytes: [u8; 8] = data[offset..offset + 8]
            .try_into()
            .map_err(|_| ParserError::BufferTooShort)?;
        let timestamp = u64::from_be_bytes(timestamp_bytes);
        offset += 8;

        // Subsystem ID (1 byte)
        let subsystem_id = data[offset];
        offset += 1;

        // Payload Length (2 bytes)
        let len_bytes: [u8; 2] = data[offset..offset + 2]
            .try_into()
            .map_err(|_| ParserError::BufferTooShort)?;
        let payload_len = u16::from_be_bytes(len_bytes) as usize;
        offset += 2;

        // Verify total length including checksum (1 byte)
        let total_packet_len = offset + payload_len + 1;
        if data.len() < total_packet_len {
            return Err(ParserError::BufferTooShort);
        }

        if verify_checksum {
            // Calculate Checksum (XOR sum of Header + Payload + ChecksumByte should be 0)
            // Bolt Optimization: Use iterator fold for better autovectorization
            let checksum = data[..total_packet_len].iter().fold(0u8, |acc, &x| acc ^ x);

            if checksum != 0 {
                return Err(ParserError::ChecksumMismatch);
            }
        }

        // Restrict subsequent reads to the declared packet length (excluding checksum)
        let data = &data[..offset + payload_len];

        let (subsystem, payload) = match subsystem_id {
            0 => {
                // Power: 3 * 8 bytes = 24 bytes
                if data.len() < offset + 24 {
                    return Err(ParserError::BufferTooShort);
                }

                let voltage_bytes = data[offset..offset + 8]
                    .try_into()
                    .map_err(|_| ParserError::BufferTooShort)?;
                let voltage = f64::from_be_bytes(voltage_bytes);
                offset += 8;

                let current_bytes = data[offset..offset + 8]
                    .try_into()
                    .map_err(|_| ParserError::BufferTooShort)?;
                let current = f64::from_be_bytes(current_bytes);
                offset += 8;

                let battery_bytes = data[offset..offset + 8]
                    .try_into()
                    .map_err(|_| ParserError::BufferTooShort)?;
                let battery_level = f64::from_be_bytes(battery_bytes);
                // offset += 8;

                (
                    Subsystem::Power,
                    TelemetryPayload::Power(PowerData {
                        voltage,
                        current,
                        battery_level,
                    }),
                )
            }
            1 => {
                // Thermal: 8 bytes
                if data.len() < offset + 8 {
                    return Err(ParserError::BufferTooShort);
                }
                let temp_bytes = data[offset..offset + 8]
                    .try_into()
                    .map_err(|_| ParserError::BufferTooShort)?;
                let temp_celsius = f64::from_be_bytes(temp_bytes);
                // offset += 8;

                (
                    Subsystem::Thermal,
                    TelemetryPayload::Thermal(ThermalData { temp_celsius }),
                )
            }
            3 => {
                // StarTracker: RA(8) + Dec(8) + Conf(8) + ID_Len(1) + ID(N)
                if data.len() < offset + 25 {
                    return Err(ParserError::BufferTooShort);
                }

                let ra_bytes = data[offset..offset + 8]
                    .try_into()
                    .map_err(|_| ParserError::BufferTooShort)?;
                let ra = f64::from_be_bytes(ra_bytes);
                offset += 8;

                let dec_bytes = data[offset..offset + 8]
                    .try_into()
                    .map_err(|_| ParserError::BufferTooShort)?;
                let dec = f64::from_be_bytes(dec_bytes);
                offset += 8;

                let conf_bytes = data[offset..offset + 8]
                    .try_into()
                    .map_err(|_| ParserError::BufferTooShort)?;
                let confidence = f64::from_be_bytes(conf_bytes);
                offset += 8;

                let id_len = data[offset] as usize;
                offset += 1;

                if data.len() < offset + id_len {
                    return Err(ParserError::BufferTooShort);
                }
                let id_bytes = &data[offset..offset + id_len];
                let target_id = if id_len > 0 {
                    // Bolt Optimization: Use Cow::Borrowed to avoid allocating a new String.
                    // The string slice refers directly to the input buffer 'data'.
                    let s = std::str::from_utf8(id_bytes)?;
                    Some(Cow::Borrowed(s))
                } else {
                    None
                };

                (
                    Subsystem::StarTracker,
                    TelemetryPayload::StarTracker(StarTrackerReading {
                        target_id,
                        coordinates: CelestialCoordinates {
                            right_ascension: ra,
                            declination: dec,
                        },
                        confidence,
                    }),
                )
            }
            _ => return Err(ParserError::InvalidSubsystem(subsystem_id)),
        };

        Ok(TelemetryPacket {
            timestamp,
            subsystem,
            payload,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn calculate_checksum(data: &[u8]) -> u8 {
        // Bolt Optimization: Use iterator fold for better autovectorization
        data.iter().fold(0, |acc, &x| acc ^ x)
    }

    #[test]
    fn test_parser_enforces_length() {
        // Create a Power packet with Length = 0, but valid payload data following.
        // Parser should now enforce the length and see 0 bytes of payload,
        // causing the Power parser to fail (expecting 24 bytes).
        let mut packet = Vec::new();
        packet.extend_from_slice(&(1234567890u64).to_be_bytes()); // Timestamp
        packet.push(0); // Subsystem: Power
        packet.extend_from_slice(&(0u16).to_be_bytes()); // Length = 0 (MALFORMED)

        // Calculate checksum for Header (length 11) + Payload (length 0)
        // Checksum covers bytes 0..11
        packet.push(calculate_checksum(&packet));

        // Payload (24 bytes) - should be ignored due to Length=0
        packet.extend_from_slice(&(28.0f64).to_be_bytes());
        packet.extend_from_slice(&(2.5f64).to_be_bytes());
        packet.extend_from_slice(&(90.0f64).to_be_bytes());

        // Attempt to parse
        let result = Parser::parse(&packet);

        assert!(
            result.is_err(),
            "Parser should reject packet with mismatched length"
        );
        match result {
            Err(ParserError::BufferTooShort) => {}
            _ => panic!("Expected BufferTooShort error"),
        }
    }

    #[test]
    fn test_parser_ignores_garbage_suffix() {
        let mut packet = Vec::new();
        packet.extend_from_slice(&(1234567890u64).to_be_bytes()); // Timestamp
        packet.push(0); // Subsystem: Power
        packet.extend_from_slice(&(24u16).to_be_bytes()); // Length = 24

        // Payload (24 bytes)
        packet.extend_from_slice(&(28.0f64).to_be_bytes());
        packet.extend_from_slice(&(2.5f64).to_be_bytes());
        packet.extend_from_slice(&(90.0f64).to_be_bytes());

        // Checksum
        packet.push(calculate_checksum(&packet));

        // Garbage
        packet.extend_from_slice(&[0xFF; 100]);

        // Attempt to parse
        let result = Parser::parse(&packet);

        assert!(result.is_ok());
    }

    #[test]
    fn test_checksum_validation() {
        let mut packet = Vec::new();
        packet.extend_from_slice(&(1627849200u64).to_be_bytes()); // Timestamp
        packet.push(0); // Subsystem: Power
        packet.extend_from_slice(&(24u16).to_be_bytes()); // Length = 24

        packet.extend_from_slice(&(28.0f64).to_be_bytes()); // Voltage
        packet.extend_from_slice(&(2.5f64).to_be_bytes()); // Current
        packet.extend_from_slice(&(90.0f64).to_be_bytes()); // Battery

        // Calculate correct checksum
        let correct_checksum = calculate_checksum(&packet);

        // Append INCORRECT checksum
        packet.push(correct_checksum.wrapping_add(1));

        let result = Parser::parse(&packet);
        match result {
            Err(ParserError::ChecksumMismatch) => {}
            _ => panic!("Expected ChecksumMismatch, got {:?}", result),
        }
    }

    #[test]
    fn benchmark_parser_performance() {
        // Create a valid Power packet
        let mut packet = Vec::new();
        packet.extend_from_slice(&(1627849200u64).to_be_bytes()); // Timestamp
        packet.push(0); // Subsystem: Power
        packet.extend_from_slice(&(24u16).to_be_bytes()); // Length = 24
        packet.extend_from_slice(&(28.0f64).to_be_bytes()); // Voltage
        packet.extend_from_slice(&(2.5f64).to_be_bytes()); // Current
        packet.extend_from_slice(&(90.0f64).to_be_bytes()); // Battery
        packet.push(calculate_checksum(&packet));

        // Create a StarTracker packet (involves string parsing)
        let mut st_packet = Vec::new();
        let target = "Alpha Centauri A - A very important star for navigation";
        let payload_len = 8 + 8 + 8 + 1 + target.len() as u16;

        st_packet.extend_from_slice(&(1627849220u64).to_be_bytes());
        st_packet.push(3); // Subsystem: StarTracker
        st_packet.extend_from_slice(&payload_len.to_be_bytes()); // Len
        st_packet.extend_from_slice(&(12.5f64).to_be_bytes()); // RA
        st_packet.extend_from_slice(&(45.0f64).to_be_bytes()); // Dec
        st_packet.extend_from_slice(&(0.95f64).to_be_bytes()); // Confidence
        st_packet.push(target.len() as u8);
        st_packet.extend_from_slice(target.as_bytes());
        st_packet.push(calculate_checksum(&st_packet));

        let iterations = 1_000_000;

        let start = std::time::Instant::now();
        for _ in 0..iterations {
            Parser::parse(&packet).unwrap();
        }
        let elapsed = start.elapsed();
        println!(
            "Parser::parse (Power) took {:?} for {} iterations",
            elapsed, iterations
        );

        let start_trusted = std::time::Instant::now();
        for _ in 0..iterations {
            Parser::parse_trusted(&packet).unwrap();
        }
        let elapsed_trusted = start_trusted.elapsed();
        println!(
            "Parser::parse_trusted (Power) took {:?} for {} iterations",
            elapsed_trusted, iterations
        );

        let start_st = std::time::Instant::now();
        for _ in 0..iterations {
            Parser::parse(&st_packet).unwrap();
        }
        let elapsed_st = start_st.elapsed();
        println!(
            "Parser::parse (StarTracker) took {:?} for {} iterations",
            elapsed_st, iterations
        );

        let start_trusted_st = std::time::Instant::now();
        for _ in 0..iterations {
            Parser::parse_trusted(&st_packet).unwrap();
        }
        let elapsed_trusted_st = start_trusted_st.elapsed();
        println!(
            "Parser::parse_trusted (StarTracker) took {:?} for {} iterations",
            elapsed_trusted_st, iterations
        );
    }
}
