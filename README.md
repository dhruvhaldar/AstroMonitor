# AstroMonitor

![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)
![Language: Rust](https://img.shields.io/badge/Language-Rust-orange.svg)

**AstroMonitor** is a high-performance telemetry monitoring system designed for spacecraft subsystems. It simulates, parses, and analyzes telemetry packets from various subsystems including Power, Thermal, and Attitude & Orbit Control Systems (AOCS).

## Features

- **Multi-Subsystem Support**: Handles telemetry for Power, Thermal, StarTracker, AOCS, Propulsion, and Science subsystems.
- **Real-time Parsing**: Efficiently parses binary telemetry packets.
- **Automated Monitoring**: Analyzes data streams to detect anomalies such as:
  - Low Battery Levels
  - High Thermal Readings
  - High Angular Velocity (Tumbling)
  - Low Fuel Levels
  - Large Data Sizes
  - Invalid or Unknown Data
- **Simulation Mode**: Includes a built-in packet generator to simulate data streams for testing.

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version)
- Cargo (comes with Rust)

### Installation

Clone the repository:

```bash
git clone https://github.com/dhruvhaldar/AstroMonitor.git
cd AstroMonitor
```

### Usage

To run the application with the simulated data stream:

```bash
cargo run
```

You should see output indicating packet processing and any triggered alerts:

```text
Starting Astro Monitor...
Processing packet 1... Parsed: Power - Power(PowerData { voltage: 28.0, current: 2.5, battery_level: 90.0 })
Processing packet 2... Parsed: Thermal - Thermal(ThermalData { temp_celsius: 85.5 })
*** ALERT: [Warning] High Temperature: 85.50C (Threshold: 80.00C) (Time: 1627849210) ***
Processing packet 3... Parsed: StarTracker - StarTracker(StarTrackerReading { target_id: Some("Sirius"), coordinates: CelestialCoordinates { right_ascension: 12.5, declination: 45.0 }, confidence: 0.95 })
Processing packet 4... Parsed: Power - Power(PowerData { voltage: 24.0, current: 1.0, battery_level: 15.0 })
*** ALERT: [Critical] Low Battery: 15.00% (Threshold: 20.00%) (Time: 1627849230) ***
Processing packet 5... Parsed: Aocs - Aocs(AocsData { mode: Detumbling, quaternion: [0.0, 0.0, 0.0, 1.0], angular_velocity: [0.8, 0.8, 0.2] })
*** ALERT: [Critical] High Angular Velocity: 1.15 (Threshold: 1.00) (Time: 1627849240) ***
Processing packet 6... Parsed: Propulsion - Propulsion(PropulsionData { fuel_level: 5.0, pressure: 200.0, engine_status: On })
*** ALERT: [Critical] Low Fuel Level: 5.00% (Threshold: 10.00%) (Time: 1627849250) ***
Processing packet 7... Parsed: Science - Science(ScienceData { instrument_id: "Spectrometer-A", wavelength: 500.0, exposure_time: 1000, data_size: 2000000 })
*** ALERT: [Warning] Large Data Size: 2000000 bytes (Threshold: 1000000 bytes) (Time: 1627849260) ***
```

## Project Structure

- `src/main.rs`: Entry point, runs the simulation loop.
- `src/models.rs`: Defines data structures for Subsystems and Telemetry.
- `src/monitor.rs`: Logic for analyzing packets and generating alerts.
- `src/parser.rs`: Handles the deserialization of raw binary data into structured packets.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
