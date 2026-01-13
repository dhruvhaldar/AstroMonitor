use crate::{simulation, Alert, AlertLevel, Monitor, Parser, ParserError, TelemetryPacket};
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
    alerts: Vec<(Alert, String)>,
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
            let delay = Duration::from_millis(self.simulation_delay_ms);

            if self.last_update.elapsed() >= delay {
                // Bolt Optimization: Parse packet first to avoid cloning the packet data vector.
                // We borrow `self.packets` (immutable) then `self` (mutable) separately.
                let result = Parser::parse(&self.packets[self.packet_index]);
                self.process_packet_result(result, Some(self.packet_index + 1));
                self.packet_index += 1;
                self.last_update = Instant::now();
            }

            // Bolt Optimization: Prevent busy loop by scheduling repaint only when needed
            // Calculate time until next packet should be processed
            let time_to_next = delay.saturating_sub(self.last_update.elapsed());
            ctx.request_repaint_after(time_to_next);
        }

        // GUI Layout
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Astro Monitor Dashboard");

            // Control Bar
            ui.horizontal(|ui| {
                if ui
                    .button(if self.paused { "Resume" } else { "Pause" })
                    .on_hover_text("Pause or resume the simulation updates. (Space)")
                    .clicked()
                {
                    self.paused = !self.paused;
                }
                // Handle keyboard shortcut (Space to toggle pause)
                if ui.input(|i| i.key_pressed(egui::Key::Space)) && !ui.ctx().wants_keyboard_input()
                {
                    self.paused = !self.paused;
                }
                if ui
                    .button("Restart Simulation")
                    .on_hover_text("⚠ Clears all logs, alerts, and restarts the simulation.")
                    .clicked()
                {
                    self.packet_index = 0;
                    self.logs.clear();
                    self.alerts.clear();
                    self.last_update = Instant::now();
                    self.paused = false;
                }
                ui.add(
                    egui::Slider::new(&mut self.simulation_delay_ms, 100..=2000).text("Delay (ms)"),
                )
                .on_hover_text("Adjust simulation speed (delay between packets in milliseconds)");

                let progress = self.packet_index as f32 / self.packets.len() as f32;
                ui.add(egui::ProgressBar::new(progress).text(format!(
                    "{}/{}",
                    self.packet_index,
                    self.packets.len()
                )));
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
                    if self.logs.is_empty() {
                        ui.vertical_centered(|ui| {
                            ui.add_space(20.0);
                            ui.label(egui::RichText::new("No system logs").italics().weak());
                        });
                    } else {
                        egui::ScrollArea::both()
                            .id_salt("logs_scroll")
                            .max_height(300.0)
                            .stick_to_bottom(true)
                            .show_rows(ui, row_height, self.logs.len(), |ui, row_range| {
                                for i in row_range {
                                    // Ensure fixed height by disabling wrap/truncating
                                    ui.add(egui::Label::new(&self.logs[i]).truncate())
                                        .on_hover_text(&self.logs[i]);
                                }
                            });
                    }
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
                    if self.alerts.is_empty() {
                        ui.vertical_centered(|ui| {
                            ui.add_space(20.0);
                            ui.label(egui::RichText::new("No active alerts").italics().weak());
                        });
                    } else {
                        egui::ScrollArea::both()
                            .id_salt("alerts_scroll")
                            .max_height(300.0)
                            .stick_to_bottom(true)
                            .show_rows(ui, row_height, self.alerts.len(), |ui, row_range| {
                                for i in row_range {
                                    // Bolt Optimization: Use pre-formatted string to avoid formatting in render loop
                                    let (alert, text) = &self.alerts[i];
                                    let color = match alert.level {
                                        AlertLevel::Critical => egui::Color32::RED,
                                        AlertLevel::Warning => egui::Color32::YELLOW,
                                        AlertLevel::Info => egui::Color32::LIGHT_BLUE,
                                    };
                                    // Bolt Optimization: Override text color in visual style to avoid allocation
                                    // (RichText::new(text.clone()) would allocate a new String every frame)
                                    ui.style_mut().visuals.override_text_color = Some(color);
                                    // Ensure fixed height by disabling wrap/truncating
                                    ui.add(egui::Label::new(text).truncate())
                                        .on_hover_text(text);
                                    // Reset color for safety (though loop re-sets it)
                                    ui.style_mut().visuals.override_text_color = None;
                                }
                            });
                    }
                });
            });

            ui.separator();

            // Manual Input Section
            ui.heading("Manual Packet Injection");
            ui.horizontal(|ui| {
                ui.radio_value(&mut self.input_subsystem, InputSubsystem::Power, "Power")
                    .on_hover_text("Configure Voltage, Current, and Battery parameters");
                ui.radio_value(
                    &mut self.input_subsystem,
                    InputSubsystem::Thermal,
                    "Thermal",
                )
                .on_hover_text("Configure Temperature sensor parameters");
                ui.radio_value(
                    &mut self.input_subsystem,
                    InputSubsystem::StarTracker,
                    "Star Tracker",
                )
                .on_hover_text("Configure RA, Dec, Confidence, and Target identification");
            });

            match self.input_subsystem {
                InputSubsystem::Power => {
                    ui.horizontal(|ui| {
                        ui.label("Voltage:").on_hover_text("Range: 0.0 - 120.0 V");
                        ui.add(
                            egui::DragValue::new(&mut self.input_voltage)
                                .speed(0.1)
                                .range(0.0..=120.0)
                                .suffix(" V"),
                        );
                        ui.label("Current:").on_hover_text("Range: 0.0 - 50.0 A");
                        ui.add(
                            egui::DragValue::new(&mut self.input_current)
                                .speed(0.1)
                                .range(0.0..=50.0)
                                .suffix(" A"),
                        );
                        ui.label("Battery:").on_hover_text("Range: 0 - 100 %");
                        ui.add(
                            egui::DragValue::new(&mut self.input_battery)
                                .speed(0.1)
                                .range(0.0..=100.0)
                                .suffix(" %"),
                        );
                    });
                }
                InputSubsystem::Thermal => {
                    ui.horizontal(|ui| {
                        ui.label("Temperature:")
                            .on_hover_text("Range: -273.15 - 1000.0 C");
                        ui.add(
                            egui::DragValue::new(&mut self.input_temp)
                                .speed(0.5)
                                .range(-273.15..=1000.0)
                                .suffix(" C"),
                        );
                    });
                }
                InputSubsystem::StarTracker => {
                    ui.horizontal(|ui| {
                        ui.label("RA:")
                            .on_hover_text("Right Ascension (0.0 - 360.0°)");
                        ui.add(
                            egui::DragValue::new(&mut self.input_ra)
                                .speed(0.1)
                                .range(0.0..=360.0)
                                .suffix("°"),
                        );
                        ui.label("Dec:")
                            .on_hover_text("Declination (-90.0 - 90.0°)");
                        ui.add(
                            egui::DragValue::new(&mut self.input_dec)
                                .speed(0.1)
                                .range(-90.0..=90.0)
                                .suffix("°"),
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("Confidence:").on_hover_text("Range: 0.0 - 1.0");
                        ui.add(
                            egui::DragValue::new(&mut self.input_confidence)
                                .speed(0.01)
                                .range(0.0..=1.0),
                        );
                        ui.label("Target:").on_hover_text("Max 32 characters");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.input_target)
                                .hint_text("e.g. Sirius")
                                .char_limit(32),
                        );
                    });
                }
            }

            if ui
                .button("Inject Packet")
                .on_hover_text("Construct and process a telemetry packet with the above values")
                .clicked()
            {
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
    fn process_packet_result(
        &mut self,
        result: Result<TelemetryPacket, ParserError>,
        index: Option<usize>,
    ) {
        // Bolt Optimization: Combined log message to reduce string allocations and VecDeque operations by 50%
        match result {
            Ok(packet) => {
                let log_message = if let Some(idx) = index {
                    format!(
                        "Packet {}: Parsed {:?} - {:?}",
                        idx, packet.subsystem, packet.payload
                    )
                } else {
                    format!(
                        "Manual Packet: Parsed {:?} - {:?}",
                        packet.subsystem, packet.payload
                    )
                };
                self.add_log(log_message);

                if let Some(alert) = self.monitor.analyze(&packet) {
                    self.add_log(format!(
                        "*** ALERT: [{:?}] {} ***",
                        alert.level, alert.message
                    ));
                    // Bolt Optimization: Pre-format display text to avoid allocation in render loop
                    let display_text = format!(
                        "[{:?}] {} (Time: {})",
                        alert.level, alert.message, alert.timestamp
                    );
                    self.alerts.push((alert, display_text));
                }
            }
            Err(e) => {
                let log_message = if let Some(idx) = index {
                    format!("Packet {}: Error parsing: {}", idx, e)
                } else {
                    format!("Manual Packet: Error parsing: {}", e)
                };
                self.add_log(log_message);
            }
        }
    }

    fn create_manual_packet(&self) -> Vec<u8> {
        let mut packet = Vec::new();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
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
