use crate::{Monitor, Parser, Alert, AlertLevel, simulation};
use eframe::egui;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[derive(PartialEq)]
enum InputSubsystem {
    Power,
    Thermal,
    StarTracker,
}

pub struct AstroMonitorApp {
    monitor: Monitor,
    packets: Vec<Vec<u8>>,
    packet_index: usize,
    logs: Vec<String>,
    alerts: Vec<Alert>,
    last_update: Instant,
    simulation_speed: Duration,
    paused: bool,

    // Input fields
    input_subsystem: InputSubsystem,
    input_voltage: f64,
    input_current: f64,
    input_battery: f64,
    input_temp: f64,
    input_ra: f64,
    input_dec: f64,
    input_confidence: f64,
    input_target: String,
}

impl Default for AstroMonitorApp {
    fn default() -> Self {
        Self {
            monitor: Monitor::default(),
            packets: simulation::generate_simulated_packets(),
            packet_index: 0,
            logs: Vec::new(),
            alerts: Vec::new(),
            last_update: Instant::now(),
            simulation_speed: Duration::from_millis(1000),
            paused: false,

            // Default input values
            input_subsystem: InputSubsystem::Power,
            input_voltage: 28.0,
            input_current: 2.5,
            input_battery: 95.0,
            input_temp: 25.0,
            input_ra: 0.0,
            input_dec: 0.0,
            input_confidence: 1.0,
            input_target: "Unknown".to_string(),
        }
    }
}

impl eframe::App for AstroMonitorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Simulation Logic
        if !self.paused && self.packet_index < self.packets.len() {
            if self.last_update.elapsed() >= self.simulation_speed {
                let packet_data = self.packets[self.packet_index].clone();
                self.process_packet(&packet_data, Some(self.packet_index + 1));
                self.packet_index += 1;
                self.last_update = Instant::now();
            }
            ctx.request_repaint();
        }

        // GUI Layout
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Astro Monitor Dashboard");

            // Control Bar
            ui.horizontal(|ui| {
                if ui.button(if self.paused { "Resume" } else { "Pause" }).clicked() {
                    self.paused = !self.paused;
                }
                if ui.button("Restart Simulation").clicked() {
                    self.packet_index = 0;
                    self.logs.clear();
                    self.alerts.clear();
                    self.last_update = Instant::now();
                    self.paused = false;
                }
                ui.label(format!("Progress: {}/{}", self.packet_index, self.packets.len()));
            });

            ui.separator();

            // Main Columns
            ui.columns(2, |columns| {
                // Logs Column
                columns[0].vertical(|ui| {
                    ui.heading("System Logs");
                    egui::ScrollArea::vertical()
                        .id_salt("logs_scroll")
                        .max_height(300.0)
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            for log in &self.logs {
                                ui.label(log);
                            }
                        });
                });

                // Alerts Column
                columns[1].vertical(|ui| {
                    ui.heading("Active Alerts");
                    egui::ScrollArea::vertical()
                        .id_salt("alerts_scroll")
                        .max_height(300.0)
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            for alert in &self.alerts {
                                let text = format!("[{:?}] {} (Time: {})", alert.level, alert.message, alert.timestamp);
                                let color = match alert.level {
                                    AlertLevel::Critical => egui::Color32::RED,
                                    AlertLevel::Warning => egui::Color32::YELLOW,
                                    AlertLevel::Info => egui::Color32::LIGHT_BLUE,
                                };
                                ui.colored_label(color, text);
                            }
                        });
                });
            });

            ui.separator();

            // Manual Input Section
            ui.heading("Manual Packet Injection");
            ui.horizontal(|ui| {
                ui.radio_value(&mut self.input_subsystem, InputSubsystem::Power, "Power");
                ui.radio_value(&mut self.input_subsystem, InputSubsystem::Thermal, "Thermal");
                ui.radio_value(&mut self.input_subsystem, InputSubsystem::StarTracker, "Star Tracker");
            });

            match self.input_subsystem {
                InputSubsystem::Power => {
                    ui.horizontal(|ui| {
                        ui.label("Voltage (V):");
                        ui.add(egui::DragValue::new(&mut self.input_voltage).speed(0.1));
                        ui.label("Current (A):");
                        ui.add(egui::DragValue::new(&mut self.input_current).speed(0.1));
                        ui.label("Battery (%):");
                        ui.add(egui::DragValue::new(&mut self.input_battery).speed(0.1).range(0.0..=100.0));
                    });
                }
                InputSubsystem::Thermal => {
                    ui.horizontal(|ui| {
                        ui.label("Temperature (C):");
                        ui.add(egui::DragValue::new(&mut self.input_temp).speed(0.5));
                    });
                }
                InputSubsystem::StarTracker => {
                     ui.horizontal(|ui| {
                        ui.label("RA:");
                        ui.add(egui::DragValue::new(&mut self.input_ra).speed(0.1));
                        ui.label("Dec:");
                        ui.add(egui::DragValue::new(&mut self.input_dec).speed(0.1));
                    });
                     ui.horizontal(|ui| {
                        ui.label("Confidence:");
                        ui.add(egui::DragValue::new(&mut self.input_confidence).speed(0.01).range(0.0..=1.0));
                        ui.label("Target:");
                        ui.text_edit_singleline(&mut self.input_target);
                    });
                }
            }

            if ui.button("Inject Packet").clicked() {
                let packet = self.create_manual_packet();
                self.process_packet(&packet, None);
            }
        });
    }
}

impl AstroMonitorApp {
    fn process_packet(&mut self, packet_data: &[u8], index: Option<usize>) {
         let prefix = if let Some(idx) = index {
            format!("Processing packet {}...", idx)
        } else {
            "Processing manual packet...".to_string()
        };
        self.logs.push(prefix);

        match Parser::parse(packet_data) {
            Ok(packet) => {
                self.logs.push(format!("Parsed: {:?} - {:?}", packet.subsystem, packet.payload));

                if let Some(alert) = self.monitor.analyze(&packet) {
                    self.logs.push(format!(
                        "*** ALERT: [{:?}] {} ***",
                        alert.level, alert.message
                    ));
                    self.alerts.push(alert);
                }
            }
            Err(e) => {
                self.logs.push(format!("Error parsing packet: {}", e));
            }
        }
    }

    fn create_manual_packet(&self) -> Vec<u8> {
        let mut packet = Vec::new();
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        packet.extend_from_slice(&timestamp.to_be_bytes());

        match self.input_subsystem {
            InputSubsystem::Power => {
                packet.push(0); // Subsystem ID
                packet.extend_from_slice(&(24u16).to_be_bytes()); // Len
                packet.extend_from_slice(&self.input_voltage.to_be_bytes());
                packet.extend_from_slice(&self.input_current.to_be_bytes());
                packet.extend_from_slice(&self.input_battery.to_be_bytes());
            }
            InputSubsystem::Thermal => {
                packet.push(1); // Subsystem ID
                packet.extend_from_slice(&(8u16).to_be_bytes()); // Len
                packet.extend_from_slice(&self.input_temp.to_be_bytes());
            }
            InputSubsystem::StarTracker => {
                packet.push(3); // Subsystem ID
                // Calculate len: 3*8 (f64) + 1 (u8) + target.len()
                let len = 24 + 1 + self.input_target.len() as u16;
                packet.extend_from_slice(&len.to_be_bytes()); // Len

                packet.extend_from_slice(&self.input_ra.to_be_bytes());
                packet.extend_from_slice(&self.input_dec.to_be_bytes());
                packet.extend_from_slice(&self.input_confidence.to_be_bytes());
                packet.push(self.input_target.len() as u8);
                packet.extend_from_slice(self.input_target.as_bytes());
            }
        }
        packet
    }
}
