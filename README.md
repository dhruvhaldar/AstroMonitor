# AstroMonitor

![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)
![Language: Rust](https://img.shields.io/badge/Language-Rust-orange.svg)

**AstroMonitor** is a high-performance telemetry monitoring system designed for spacecraft subsystems. It simulates, parses, and analyzes telemetry packets from various subsystems including Power, Thermal, and Attitude & Orbit Control Systems (AOCS).

## Features

- **Multi-Subsystem Support**: Handles telemetry for Power, Thermal, AOCS, and StarTracker subsystems.
- **Real-time Parsing**: Efficiently parses binary telemetry packets.
- **Automated Monitoring**: Analyzes data streams to detect anomalies such as:
  - Low Battery Levels
  - High Thermal Readings
  - Invalid or Unknown Data
- **Simulation Mode**: Includes a built-in packet generator to simulate data streams for testing.

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version)
- Cargo (comes with Rust)

### Installation

Clone the repository:

```bash
git clone https://github.com/StartDust/AstroMonitor.git
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
Processing packet 1... Parsed: Power - PowerData { voltage: 28.0, current: 2.5, battery_level: 90.0 }
Processing packet 2... Parsed: Thermal - ThermalData { temp_celsius: 85.5 }
*** ALERT: [Warning] High Temperature Detected: 85.5°C (Time: 1627849210) ***
...
```

## Project Structure

- `src/main.rs`: Entry point, runs the simulation loop.
- `src/models.rs`: Defines data structures for Subsystems and Telemetry.
- `src/monitor.rs`: Logic for analyzing packets and generating alerts.
- `src/parser.rs`: Handles the deserialization of raw binary data into structured packets.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
