use crate::{Monitor, Parser, Alert, AlertLevel, simulation, TelemetryPacket, ParserError};
use eframe::egui;
use std::collections::VecDeque;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const MAX_LOGS: usize = 1000;

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
    logs: VecDeque<String>,
    alerts: Vec<Alert>,
    last_update: Instant,
    simulation_delay_ms: u64,
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
            logs: VecDeque::new(),
            alerts: Vec::new(),
            last_update: Instant::now(),
            simulation_delay_ms: 1000,
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
            if self.last_update.elapsed() >= Duration::from_millis(self.simulation_delay_ms) {
                // Bolt Optimization: Parse packet first to avoid cloning the packet data vector.
                // We borrow `self.packets` (immutable) then `self` (mutable) separately.
                let result = Parser::parse(&self.packets[self.packet_index]);
                self.process_packet_result(result, Some(self.packet_index + 1));
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
                if ui.button(if self.paused { "Resume" } else { "Pause" })
                    .on_hover_text("Pause or resume the simulation updates.")
                    .clicked()
                {
                    self.paused = !self.paused;
                }
                if ui.button("Restart Simulation")
                    .on_hover_text("⚠ Clears all logs, alerts, and restarts the simulation.")
                    .clicked()
                {
                    self.packet_index = 0;
                    self.logs.clear();
                    self.alerts.clear();
                    self.last_update = Instant::now();
                    self.paused = false;
                }
                ui.add(egui::Slider::new(&mut self.simulation_delay_ms, 100..=2000).text("Delay (ms)"))
                    .on_hover_text("Adjust simulation speed (delay between packets in milliseconds)");

                ui.label(format!("Progress: {}/{}", self.packet_index, self.packets.len()));
            });

            ui.separator();

            // Calculate row height for virtualization
            let text_style = egui::TextStyle::Body;
            let row_height = ui.text_style_height(&text_style);

            // Main Columns
            ui.columns(2, |columns| {
                // Logs Column
                columns[0].vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.heading("System Logs");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("🗑").on_hover_text("Clear logs").clicked() {
                                self.logs.clear();
                            }
                        });
                    });

                    // Bolt Optimization: Use virtualization for logs
                    egui::ScrollArea::both()
                        .id_salt("logs_scroll")
                        .max_height(300.0)
                        .stick_to_bottom(true)
                        .show_rows(ui, row_height, self.logs.len(), |ui, row_range| {
                            for i in row_range {
                                // Ensure fixed height by disabling wrap/truncating
                                ui.add(egui::Label::new(&self.logs[i]).truncate());
                            }
                        });
                });

                // Alerts Column
                columns[1].vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.heading("Active Alerts");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("🗑").on_hover_text("Clear alerts").clicked() {
                                self.alerts.clear();
                            }
                        });
                    });

                    // Bolt Optimization: Use virtualization for alerts
                    egui::ScrollArea::both()
                        .id_salt("alerts_scroll")
                        .max_height(300.0)
                        .stick_to_bottom(true)
                        .show_rows(ui, row_height, self.alerts.len(), |ui, row_range| {
                            for i in row_range {
                                let alert = &self.alerts[i];
                                let text = format!("[{:?}] {} (Time: {})", alert.level, alert.message, alert.timestamp);
                                let color = match alert.level {
                                    AlertLevel::Critical => egui::Color32::RED,
                                    AlertLevel::Warning => egui::Color32::YELLOW,
                                    AlertLevel::Info => egui::Color32::LIGHT_BLUE,
                                };
                                // Ensure fixed height by disabling wrap/truncating
                                ui.add(egui::Label::new(egui::RichText::new(text).color(color)).truncate());
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
                        ui.label("Voltage:");
                        ui.add(egui::DragValue::new(&mut self.input_voltage).speed(0.1).suffix(" V"));
                        ui.label("Current:");
                        ui.add(egui::DragValue::new(&mut self.input_current).speed(0.1).suffix(" A"));
                        ui.label("Battery:");
                        ui.add(egui::DragValue::new(&mut self.input_battery).speed(0.1).range(0.0..=100.0).suffix(" %"));
                    });
                }
                InputSubsystem::Thermal => {
                    ui.horizontal(|ui| {
                        ui.label("Temperature:");
                        ui.add(egui::DragValue::new(&mut self.input_temp).speed(0.5).suffix(" C"));
                    });
                }
                InputSubsystem::StarTracker => {
                     ui.horizontal(|ui| {
                        ui.label("RA:");
                        ui.add(egui::DragValue::new(&mut self.input_ra).speed(0.1).suffix("°"));
                        ui.label("Dec:");
                        ui.add(egui::DragValue::new(&mut self.input_dec).speed(0.1).suffix("°"));
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
                let result = Parser::parse(&packet);
                self.process_packet_result(result, None);
            }
        });
    }
}

impl AstroMonitorApp {
    fn add_log(&mut self, message: String) {
        if self.logs.len() >= MAX_LOGS {
            self.logs.pop_front();
        }
        self.logs.push_back(message);
    }

    // Bolt Optimization: Accepts parsed result to avoid borrowing conflicts and unnecessary cloning
    fn process_packet_result(&mut self, result: Result<TelemetryPacket, ParserError>, index: Option<usize>) {
        let prefix = if let Some(idx) = index {
            format!("Processing packet {}...", idx)
        } else {
            "Processing manual packet...".to_string()
        };
        self.add_log(prefix);

        match result {
            Ok(packet) => {
                self.add_log(format!("Parsed: {:?} - {:?}", packet.subsystem, packet.payload));

                if let Some(alert) = self.monitor.analyze(&packet) {
                    self.add_log(format!(
                        "*** ALERT: [{:?}] {} ***",
                        alert.level, alert.message
                    ));
                    self.alerts.push(alert);
                }
            }
            Err(e) => {
                self.add_log(format!("Error parsing packet: {}", e));
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
