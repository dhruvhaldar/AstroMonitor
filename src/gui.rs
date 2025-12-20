use crate::{Monitor, Parser, Alert, AlertLevel, simulation};
use eframe::egui;
use std::time::{Duration, Instant};

pub struct AstroMonitorApp {
    monitor: Monitor,
    packets: Vec<Vec<u8>>,
    packet_index: usize,
    logs: Vec<String>,
    alerts: Vec<Alert>,
    last_update: Instant,
    simulation_speed: Duration,
    paused: bool,
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
            simulation_speed: Duration::from_millis(1000), // Slower speed for GUI
            paused: false,
        }
    }
}

impl eframe::App for AstroMonitorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Simulation Logic
        if !self.paused && self.packet_index < self.packets.len() {
            if self.last_update.elapsed() >= self.simulation_speed {
                let packet_data = &self.packets[self.packet_index];
                self.logs.push(format!("Processing packet {}...", self.packet_index + 1));

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

                self.packet_index += 1;
                self.last_update = Instant::now();
            }
            ctx.request_repaint(); // Keep refreshing to check timer
        }

        // GUI Layout
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Astro Monitor Dashboard");

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

            ui.columns(2, |columns| {
                // Logs Column
                columns[0].vertical(|ui| {
                    ui.heading("System Logs");
                    egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                        for log in &self.logs {
                            ui.label(log);
                        }
                    });
                });

                // Alerts Column
                columns[1].vertical(|ui| {
                    ui.heading("Active Alerts");
                    egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
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
        });
    }
}
