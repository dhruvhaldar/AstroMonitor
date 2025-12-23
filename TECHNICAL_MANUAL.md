# AstroMonitor Technical Manual

## 1. Project Overview

**AstroMonitor** is a Rust-based telemetry monitoring system designed for spacecraft operations. It simulates the ingestion of binary telemetry packets, parses them into structured data, and analyzes them in real-time to detect critical system anomalies.

The system is designed to handle multiple spacecraft subsystems:

- **Power System** (Voltage, Current, Battery)
- **Thermal System** (Temperature)
- **Star Tracker** (Navigation/Attitude determination)

## 2. System Architecture & Workflow

The application follows a linear data processing pipeline:

### 2.1 Simulation (Data Source)

- **Source**: `src/main.rs`
- **Functionality**: Generates a stream of raw binary data (`Vec<u8>`).
- **Mock Data**: Simulates scenarios like Normal Operation, High Temperature (Warning), and Low Battery (Critical).

### 2.2 Parsing Layer

- **Source**: `src/parser.rs`
- **Functionality**: deserializes raw bytes into valid Rust structs (`TelemetryPacket`).
- **Packet Structure**:
  - **Header**:
    - `Timestamp` (8 bytes, u64 big-endian)
    - `SubsystemID` (1 byte, u8)
    - `Length` (2 bytes, u16 big-endian)
  - **Payload**: Variable length depending on the subsystem.

### 2.3 Monitoring & Analysis Layer

- **Source**: `src/monitor.rs`
- **Functionality**: Evaluates the parsed `TelemetryPacket` against predefined safety thresholds.
- **Output**: Generates an `Alert` if a threshold is breached.

---

## 3. Variable Reference & Data Structures

This section details the specific variables and data types used within the application.

### 3.1 Core Packet Structure (`TelemetryPacket`)

The envelope containing all telemetry data.
| Variable Name | Type | Description |
| :--- | :--- | :--- |
| `timestamp` | `u64` | Unix timestamp of when the packet was generated. |
| `subsystem` | `enum Subsystem` | Identifies the source system (`Power`, `Thermal`, `StarTracker`). |
| `payload` | `enum TelemetryPayload` | The actual sensor data specific to the subsystem. |

### 3.2 Subsystem Payloads (`src/models.rs`)

#### **Power System (`PowerData`)**

Tracks electrical health.
| Variable Name | Type | Unit | Description |
| :--- | :--- | :--- | :--- |
| `voltage` | `f64` | Volts (V) | System bus voltage. |
| `current` | `f64` | Amperes (A) | System current draw. |
| `battery_level`| `f64` | Percent (%)| Remaining battery capacity. |

#### **Thermal System (`ThermalData`)**

Tracks temperature sensors.
| Variable Name | Type | Unit | Description |
| :--- | :--- | :--- | :--- |
| `temp_celsius` | `f64` | Celsius (C) | Sensor temperature. |

#### **Star Tracker (`StarTrackerReading`)**

Optical navigation sensor data.
| Variable Name | Type | Unit | Description |
| :--- | :--- | :--- | :--- |
| `coordinates` | `struct CelestialCoordinates` | N/A | Contains `right_ascension` and `declination` (degrees). |
| `confidence` | `f64` | 0.0 - 1.0 | Quality/certainty of the star match (1.0 = 100%). |
| `target_id` | `Option<String>` | Text | Name of the identified star (e.g., "Sirius"). |

### 3.3 Monitoring Configuration (`Monitor`)

Variables used to determine system health state. defaults are defined in `src/monitor.rs`.

| Variable Name         | Default Value | Condition for Alert    | Alert Level  |
| :-------------------- | :------------ | :--------------------- | :----------- |
| `min_battery_level`   | `20.0` (%)    | `battery_level < 20.0` | **Critical** |
| `max_temp_celsius`    | `80.0` (C)    | `temp_celsius > 80.0`  | **Warning**  |
| `min_star_confidence` | `0.8` (80%)   | `confidence < 0.8`     | **Info**     |
