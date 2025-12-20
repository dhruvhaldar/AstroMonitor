pub fn generate_simulated_packets() -> Vec<Vec<u8>> {
    let mut packets = Vec::new();

    // 1. Power Packet (Normal)
    let mut p1 = Vec::new();
    p1.extend_from_slice(&(1627849200u64).to_be_bytes()); // Timestamp
    p1.push(0); // Subsystem: Power
    p1.extend_from_slice(&(24u16).to_be_bytes()); // Len
    p1.extend_from_slice(&(28.0f64).to_be_bytes()); // Voltage
    p1.extend_from_slice(&(2.5f64).to_be_bytes()); // Current
    p1.extend_from_slice(&(90.0f64).to_be_bytes()); // Battery
    packets.push(p1);

    // 2. Thermal Packet (High Temp)
    let mut p2 = Vec::new();
    p2.extend_from_slice(&(1627849210u64).to_be_bytes());
    p2.push(1); // Subsystem: Thermal
    p2.extend_from_slice(&(8u16).to_be_bytes());
    p2.extend_from_slice(&(85.5f64).to_be_bytes()); // Temp > 80 (Threshold)
    packets.push(p2);

    // 3. Star Tracker Packet (Good Confidence)
    let mut p3 = Vec::new();
    p3.extend_from_slice(&(1627849220u64).to_be_bytes());
    p3.push(3); // Subsystem: StarTracker
    p3.extend_from_slice(&(0u16).to_be_bytes()); // Len
    p3.extend_from_slice(&(12.5f64).to_be_bytes()); // RA
    p3.extend_from_slice(&(45.0f64).to_be_bytes()); // Dec
    p3.extend_from_slice(&(0.95f64).to_be_bytes()); // Confidence
    let target = "Sirius";
    p3.push(target.len() as u8);
    p3.extend_from_slice(target.as_bytes());
    packets.push(p3);

    // 4. Power Packet (Low Battery)
    let mut p4 = Vec::new();
    p4.extend_from_slice(&(1627849230u64).to_be_bytes());
    p4.push(0); // Subsystem: Power
    p4.extend_from_slice(&(24u16).to_be_bytes());
    p4.extend_from_slice(&(24.0f64).to_be_bytes());
    p4.extend_from_slice(&(1.0f64).to_be_bytes());
    p4.extend_from_slice(&(15.0f64).to_be_bytes()); // Battery < 20 (Threshold)
    packets.push(p4);

    packets
}
