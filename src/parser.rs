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
    Utf8Error(#[from] std::string::FromUtf8Error),
    #[error("Unknown error")]
    Unknown,
}

pub struct Parser;

impl Parser {
    pub fn parse(data: &[u8]) -> Result<TelemetryPacket, ParserError> {
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
        let _len_bytes: [u8; 2] = data[offset..offset + 2]
            .try_into()
            .map_err(|_| ParserError::BufferTooShort)?;
        offset += 2;

        let (subsystem, payload) = match subsystem_id {
            0 => {
                // Power: 3 * 8 bytes = 24 bytes
                if data.len() < offset + 24 {
                    return Err(ParserError::BufferTooShort);
                }
                
                let voltage_bytes = data[offset..offset + 8].try_into().map_err(|_| ParserError::BufferTooShort)?;
                let voltage = f64::from_be_bytes(voltage_bytes);
                offset += 8;

                let current_bytes = data[offset..offset + 8].try_into().map_err(|_| ParserError::BufferTooShort)?;
                let current = f64::from_be_bytes(current_bytes);
                offset += 8;

                let battery_bytes = data[offset..offset + 8].try_into().map_err(|_| ParserError::BufferTooShort)?;
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
                let temp_bytes = data[offset..offset + 8].try_into().map_err(|_| ParserError::BufferTooShort)?;
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
                
                let ra_bytes = data[offset..offset + 8].try_into().map_err(|_| ParserError::BufferTooShort)?;
                let ra = f64::from_be_bytes(ra_bytes);
                offset += 8;
                
                let dec_bytes = data[offset..offset + 8].try_into().map_err(|_| ParserError::BufferTooShort)?;
                let dec = f64::from_be_bytes(dec_bytes);
                offset += 8;

                let conf_bytes = data[offset..offset + 8].try_into().map_err(|_| ParserError::BufferTooShort)?;
                let confidence = f64::from_be_bytes(conf_bytes);
                offset += 8;
                
                let id_len = data[offset] as usize;
                offset += 1;

                if data.len() < offset + id_len {
                    return Err(ParserError::BufferTooShort);
                }
                let id_bytes = &data[offset..offset + id_len];
                let target_id = if id_len > 0 {
                    Some(String::from_utf8(id_bytes.to_vec())?)
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
