use crate::{
    simulation, AlertCondition, AlertLevel, Monitor, MonitorEvent, Parser, ParserError,
    TelemetryPacket, TelemetryPayload,
};
use eframe::egui;
use log::{debug, error, info, warn};
use std::collections::VecDeque;
use std::fmt::Write;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const MAX_LOGS: usize = 1000;
const MAX_ALERTS: usize = 1000;

#[derive(PartialEq)]
enum InputSubsystem {
    Power,
    Thermal,
    StarTracker,
}

#[derive(Clone)]
enum ResolvedLogText<'a> {
    Borrowed(&'a str),
    Shared(Arc<String>),
}

impl<'a> AsRef<str> for ResolvedLogText<'a> {
    fn as_ref(&self) -> &str {
        match self {
            Self::Borrowed(s) => s,
            Self::Shared(arc) => arc.as_str(),
        }
    }
}

impl<'a> From<ResolvedLogText<'a>> for egui::WidgetText {
    fn from(val: ResolvedLogText<'a>) -> Self {
        match val {
            ResolvedLogText::Borrowed(s) => s.into(),
            ResolvedLogText::Shared(arc) => arc.as_str().into(),
        }
    }
}

#[derive(Clone)]
enum LogEntry {
    Packet(String),
    SimulatedPacket(usize),
    Message(String),
    Alert(String),
}

impl std::fmt::Display for LogEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogEntry::Packet(s) => write!(f, "{}", s),
            LogEntry::SimulatedPacket(idx) => write!(f, "Packet {}", idx + 1),
            LogEntry::Message(s) => write!(f, "{}", s),
            LogEntry::Alert(s) => write!(f, "{}", s),
        }
    }
}

impl LogEntry {
    fn into_string(self) -> String {
        match self {
            LogEntry::Packet(s) => s,
            LogEntry::SimulatedPacket(_) => String::new(), // Cannot convert index back to formatted string without context
            LogEntry::Message(s) => s,
            LogEntry::Alert(s) => s,
        }
    }
}

#[derive(Clone)]
struct AlertEntry {
    event: MonitorEvent,
    text: String,
}

pub struct AstroMonitorApp {
    monitor: Monitor,
    packets: Vec<Vec<u8>>,
    packet_index: usize,
    logs: VecDeque<LogEntry>,
    alerts: VecDeque<AlertEntry>,
    alert_counts: [usize; 3], // [Info, Warning, Critical]
    last_update: Instant,
    simulation_delay_ms: u64,
    paused: bool,
    progress_text: String,

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

    // Feedback state
    last_log_copy_time: Option<Instant>,
    last_alert_copy_time: Option<Instant>,
    last_log_clear_time: Option<Instant>,
    last_alert_clear_time: Option<Instant>,
    last_injection_time: Option<Instant>,
    restart_confirm_time: Option<Instant>,
    log_clear_confirm: Option<Instant>,
    alert_clear_confirm: Option<Instant>,
    last_nominal_apply_time: Option<Instant>,
    last_alert_apply_time: Option<Instant>,

    // Filters
    filter_logs_important: bool,

    // Bolt Optimization: Persistent buffer for filtered logs indices to avoid allocation loop
    filtered_log_indices: Vec<usize>,
    logs_mutation_counter: u64,
    cached_filter_counter: u64,

    // Cached Tooltips (Bolt Optimization)
    // Note: These must be updated if `monitor` thresholds are changed at runtime.
    cached_battery_tooltip: String,
    cached_temp_tooltip: String,
    cached_star_tooltip: String,
}

impl Default for AstroMonitorApp {
    fn default() -> Self {
        let packets = simulation::generate_simulated_packets();
        let progress_text = format!("0/{}", packets.len());
        let monitor = Monitor::default();

        // Bolt Optimization: Pre-format tooltip strings to avoid allocation in render loop
        let cached_battery_tooltip = format!(
            "Values below {:.0}% will trigger a Critical alert",
            monitor.min_battery_level
        );
        let cached_temp_tooltip = format!(
            "Values above {:.0}°C will trigger a Warning alert",
            monitor.max_temp_celsius
        );
        let cached_star_tooltip = format!(
            "Values below {:.2} will trigger an Info alert",
            monitor.min_star_confidence
        );

        Self {
            monitor,
            packets,
            packet_index: 0,
            // Bolt Optimization: Pre-allocate collections to avoid reallocations during startup
            logs: VecDeque::with_capacity(MAX_LOGS),
            alerts: VecDeque::with_capacity(MAX_ALERTS),
            alert_counts: [0, 0, 0],
            last_update: Instant::now(),
            simulation_delay_ms: 1000,
            paused: false,
            progress_text,
            last_log_copy_time: None,
            last_alert_copy_time: None,
            last_log_clear_time: None,
            last_alert_clear_time: None,
            last_injection_time: None,
            restart_confirm_time: None,
            log_clear_confirm: None,
            alert_clear_confirm: None,
            last_nominal_apply_time: None,
            last_alert_apply_time: None,

            filter_logs_important: false,
            filtered_log_indices: Vec::with_capacity(MAX_LOGS),
            logs_mutation_counter: 0,
            cached_filter_counter: 0,

            cached_battery_tooltip,
            cached_temp_tooltip,
            cached_star_tooltip,

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
            let mut steps = 0;
            let max_steps = 10; // Bolt Optimization: Prevent freeze/spiral of death

            // Bolt Optimization: Fixed timestep loop to decouple simulation speed from frame rate
            while self.last_update.elapsed() >= delay
                && self.packet_index < self.packets.len()
                && steps < max_steps
            {
                // Parse packet first to avoid cloning the packet data vector.
                // Bolt Optimization: Use parse_trusted because self.packets comes from internal simulation.
                // SAFETY: The method is now safe (performs UTF-8 validation).
                let result = Parser::parse_trusted(&self.packets[self.packet_index]);
                Self::process_result(
                    &mut self.logs,
                    &mut self.alerts,
                    &mut self.alert_counts,
                    &self.monitor,
                    result,
                    Some(self.packet_index + 1),
                );
                self.packet_index += 1;
                // Bolt Optimization: Increment logs mutation counter
                self.logs_mutation_counter = self.logs_mutation_counter.wrapping_add(1);

                // Bolt Optimization: Moved update_progress_text outside the loop to update once per frame instead of per packet
                self.last_update += delay; // Catch up without drift
                steps += 1;
            }
            self.update_progress_text();

            // If we hit the limit, reset to avoid backlog
            if steps >= max_steps {
                self.last_update = Instant::now();
            }

            // Bolt Optimization: Prevent busy loop by scheduling repaint only when needed
            // Calculate time until next packet should be processed
            let time_to_next = delay.saturating_sub(self.last_update.elapsed());
            ctx.request_repaint_after(time_to_next);
        }

        // GUI Layout
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Astro Monitor Dashboard");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Palette UX Enhancement: Help & Shortcuts
                    ui.add(egui::Button::new("?").frame(false))
                        .on_hover_ui(|ui| {
                            ui.heading("Help & Shortcuts");
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Space").strong());
                                ui.label("Toggle Pause/Resume");
                            });
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Ctrl + Enter").strong());
                                ui.label("Inject Manual Packet");
                            });
                            ui.separator();
                            ui.label(egui::RichText::new("Tips:").strong());
                            ui.label("• Hold Shift while dragging sliders for precision.");
                            ui.label("• Right-click sliders to type exact values.");
                            ui.label("• Hover over 'System Status' for alert breakdown.");
                        });

                    ui.separator();

                    // Bolt Optimization: O(1) status check using cached alert counts
                    let (status_text, status_color) = if self.alert_counts[2] > 0 {
                        ("SYSTEM CRITICAL 🔴", egui::Color32::RED)
                    } else if self.alert_counts[1] > 0 {
                        ("System Warning ⚠️", egui::Color32::YELLOW)
                    } else if self.alert_counts[0] > 0 {
                        ("System Info ℹ", egui::Color32::LIGHT_BLUE)
                    } else {
                        ("System Nominal 🟢", egui::Color32::GREEN)
                    };
                    ui.label(
                        egui::RichText::new(status_text)
                            .color(status_color)
                            .strong(),
                    )
                    .on_hover_ui(|ui| {
                        ui.label("Aggregate system status based on active alerts:");
                        ui.label(
                            egui::RichText::new(format!("Critical: {}", self.alert_counts[2]))
                                .color(egui::Color32::RED),
                        );
                        ui.label(
                            egui::RichText::new(format!("Warning:  {}", self.alert_counts[1]))
                                .color(egui::Color32::YELLOW),
                        );
                        ui.label(
                            egui::RichText::new(format!("Info:     {}", self.alert_counts[0]))
                                .color(egui::Color32::LIGHT_BLUE),
                        );
                    });
                });
            });

            // Control Bar
            ui.horizontal(|ui| {
                if ui
                    .button(if self.paused {
                        "▶ Resume"
                    } else {
                        "⏸ Pause"
                    })
                    .on_hover_ui(|ui| {
                        ui.label("Pause or resume the simulation updates. (Space)");
                    })
                    .clicked()
                {
                    self.paused = !self.paused;
                    if !self.paused {
                        self.last_update = Instant::now();
                    }
                }
                // Handle keyboard shortcut (Space to toggle pause)
                if ui.input(|i| i.key_pressed(egui::Key::Space)) && !ui.ctx().wants_keyboard_input()
                {
                    self.paused = !self.paused;
                    if !self.paused {
                        self.last_update = Instant::now();
                    }
                }
                let restart_clicked = if let Some(_t) = self
                    .restart_confirm_time
                    .filter(|t| t.elapsed().as_secs() < 3)
                {
                    let btn = ui.add(
                        egui::Button::new(
                            egui::RichText::new("⚠ Confirm?").color(egui::Color32::RED),
                        )
                        .fill(egui::Color32::from_rgb(50, 0, 0)),
                    );
                    if btn
                        .on_hover_ui(|ui| {
                            ui.label("Click again to confirm full system restart.");
                        })
                        .clicked()
                    {
                        self.restart_confirm_time = None;
                        true
                    } else {
                        false
                    }
                } else {
                    if ui
                        .button("↻ Restart")
                        .on_hover_ui(|ui| {
                            ui.label("⚠ Clears all logs, alerts, and restarts the simulation.");
                        })
                        .clicked()
                    {
                        self.restart_confirm_time = Some(Instant::now());
                    }
                    false
                };

                if restart_clicked {
                    self.packet_index = 0;
                    self.update_progress_text();
                    self.logs.clear();
                    self.alerts.clear();
                    self.alert_counts = [0, 0, 0];
                    self.last_update = Instant::now();
                    self.paused = false;
                }
                // Palette UX Enhancement: Display frequency (Hz) alongside delay (ms)
                let freq = 1000.0 / self.simulation_delay_ms as f64;
                if ui
                    .add(
                        egui::Slider::new(&mut self.simulation_delay_ms, 100..=2000)
                            .text(format!("Delay (ms) [{:.1} Hz]", freq)),
                    )
                    .on_hover_ui(|ui| {
                        ui.label("Adjust simulation speed (delay between packets in milliseconds)");
                        ui.label(
                            egui::RichText::new(format!("Frequency: {:.1} Hz", freq))
                                .strong()
                                .color(egui::Color32::LIGHT_BLUE),
                        );
                    })
                    .changed()
                {
                    Self::format_progress_text(
                        &mut self.progress_text,
                        self.packet_index,
                        self.packets.len(),
                        self.simulation_delay_ms,
                    );
                }

                let progress = self.packet_index as f32 / self.packets.len() as f32;
                // Bolt Optimization: Use cached progress text to avoid formatting/allocation every frame
                ui.add(egui::ProgressBar::new(progress).text(&self.progress_text))
                    .on_hover_ui(|ui| {
                        ui.label("Simulation Progress");
                    });
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
                        ui.heading(format!("System Logs ({})", self.logs.len()));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let (clear_icon, clear_tooltip, confirm_mode) = if let Some(_t) =
                                self.log_clear_confirm.filter(|t| t.elapsed().as_secs() < 3)
                            {
                                ("⚠", "Click again to confirm clear logs", true)
                            } else if let Some(_t) =
                                self.last_log_clear_time.filter(|t| t.elapsed().as_secs() < 2)
                            {
                                ("✔", "Cleared!", false)
                            } else {
                                ("🗑", "Clear logs", false)
                            };

                            let btn = if confirm_mode {
                                ui.add(
                                    egui::Button::new(
                                        egui::RichText::new(clear_icon).color(egui::Color32::RED),
                                    )
                                    .fill(egui::Color32::from_rgb(50, 0, 0)),
                                )
                            } else {
                                ui.button(clear_icon)
                            };

                            if btn
                                .on_hover_ui(|ui| {
                                    ui.label(clear_tooltip);
                                })
                                .clicked()
                            {
                                if confirm_mode {
                                    self.logs.clear();
                                    self.logs_mutation_counter =
                                        self.logs_mutation_counter.wrapping_add(1);
                                    self.log_clear_confirm = None;
                                    self.last_log_clear_time = Some(Instant::now());
                                    ui.ctx().request_repaint_after(Duration::from_secs(2));
                                } else if self.last_log_clear_time.is_none()
                                    || self.last_log_clear_time.unwrap().elapsed().as_secs() >= 2
                                {
                                    self.log_clear_confirm = Some(Instant::now());
                                    ui.ctx().request_repaint();
                                }
                            }
                            let (icon, tooltip) = if let Some(_t) = self
                                .last_log_copy_time
                                .filter(|t| t.elapsed().as_secs() < 2)
                            {
                                ("✔", "Copied!")
                            } else {
                                ("📋", "Copy logs to clipboard")
                            };
                            if ui
                                .button(icon)
                                .on_hover_ui(|ui| {
                                    ui.label(tooltip);
                                })
                                .clicked()
                            {
                                // Bolt Optimization: Pre-calculate size estimate and write to single buffer
                                let mut all_logs = String::with_capacity(self.logs.len() * 80);
                                for (i, log) in self.logs.iter().enumerate() {
                                    if i > 0 {
                                        all_logs.push('\n');
                                    }
                                    match log {
                                        LogEntry::SimulatedPacket(idx) => {
                                            if let Some(packet_data) = self.packets.get(*idx) {
                                                // Bolt Optimization: Use parse_trusted to skip checksum validation for internal data
                                                // SAFETY: The method is now safe (performs UTF-8 validation).
                                                if let Ok(packet) = Parser::parse_trusted(packet_data) {
                                                    Self::format_log_packet(
                                                        &mut all_logs,
                                                        packet.timestamp,
                                                        Some(idx + 1),
                                                        &packet.payload,
                                                    );
                                                } else {
                                                    all_logs.push_str("Error parsing packet");
                                                }
                                            } else {
                                                all_logs.push_str("Invalid Packet Index");
                                            }
                                        }
                                        LogEntry::Packet(s) => all_logs.push_str(s),
                                        LogEntry::Message(s) => all_logs.push_str(s),
                                        LogEntry::Alert(s) => all_logs.push_str(s),
                                    }
                                }
                                ui.output_mut(|o| o.copied_text = all_logs);
                                self.last_log_copy_time = Some(Instant::now());
                                ui.ctx().request_repaint_after(Duration::from_secs(2));
                            }

                            ui.separator();

                            if ui
                                .selectable_label(self.filter_logs_important, "⚠ Important Only")
                                .on_hover_text("Show only Alerts and Messages, hiding raw telemetry packets.")
                                .clicked()
                            {
                                self.filter_logs_important = !self.filter_logs_important;
                            }
                        });
                    });

                    // Bolt Optimization: Use virtualization for logs
                    // Palette Optimization: Filter logs if toggle is active
                    // Bolt Optimization: Re-use buffer for filtered indices to avoid allocation loop
                    if self.filter_logs_important {
                        // Only recompute if logs changed since last cache update
                        if self.logs_mutation_counter != self.cached_filter_counter {
                            self.filtered_log_indices.clear();
                            self.filtered_log_indices.extend(
                                self.logs
                                    .iter()
                                    .enumerate()
                                    .filter(|(_, entry)| !matches!(entry, LogEntry::Packet(_)))
                                    .map(|(i, _)| i),
                            );
                            self.cached_filter_counter = self.logs_mutation_counter;
                        }
                    }

                    let count = if self.filter_logs_important {
                        self.filtered_log_indices.len()
                    } else {
                        self.logs.len()
                    };

                    if self.logs.is_empty() {
                        ui.vertical_centered(|ui| {
                            ui.add_space(20.0);
                            ui.label(egui::RichText::new("📄").size(24.0));
                            ui.label(egui::RichText::new("No System Logs").heading());
                            ui.label(
                                egui::RichText::new("Telemetry events will appear here").weak(),
                            );
                        });
                    } else if count == 0 && self.filter_logs_important {
                        ui.vertical_centered(|ui| {
                            ui.add_space(20.0);
                            ui.label(egui::RichText::new("🔍").size(24.0));
                            ui.label(egui::RichText::new("No Important Logs").heading());
                            ui.label(
                                egui::RichText::new("Only routine telemetry packets found").weak(),
                            );
                        });
                    } else {
                        egui::ScrollArea::both()
                            .id_salt("logs_scroll")
                            .max_height(300.0)
                            .stick_to_bottom(true)
                            .show_rows(ui, row_height, count, |ui, row_range| {
                                for i in row_range {
                                    if i % 2 == 1 {
                                        let rect = egui::Rect::from_min_size(
                                            ui.cursor().min,
                                            egui::vec2(ui.available_width(), row_height),
                                        );
                                        ui.painter().rect_filled(
                                            rect,
                                            0.0,
                                            ui.visuals().faint_bg_color,
                                        );
                                    }

                                    let actual_index = if self.filter_logs_important {
                                        self.filtered_log_indices[i]
                                    } else {
                                        i
                                    };

                                    // Bolt Optimization: Use pre-formatted string directly to avoid allocation
                                    let text =
                                        self.resolve_log_entry_text(&self.logs[actual_index], ui);
                                    // Ensure fixed height by disabling wrap/truncating
                                    ui.add(egui::Label::new(text.as_ref()).truncate())
                                        .on_hover_ui(|ui| {
                                            ui.label(text);
                                        });
                                }
                            });
                    }
                });

                // Alerts Column
                columns[1].vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.heading(format!("Active Alerts ({})", self.alerts.len()));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let (clear_icon, clear_tooltip, confirm_mode) = if let Some(_t) =
                                self.alert_clear_confirm.filter(|t| t.elapsed().as_secs() < 3)
                            {
                                ("⚠", "Click again to confirm clear alerts", true)
                            } else if let Some(_t) =
                                self.last_alert_clear_time.filter(|t| t.elapsed().as_secs() < 2)
                            {
                                ("✔", "Cleared!", false)
                            } else {
                                ("🗑", "Clear alerts", false)
                            };

                            let btn = if confirm_mode {
                                ui.add(
                                    egui::Button::new(
                                        egui::RichText::new(clear_icon).color(egui::Color32::RED),
                                    )
                                    .fill(egui::Color32::from_rgb(50, 0, 0)),
                                )
                            } else {
                                ui.button(clear_icon)
                            };

                            if btn
                                .on_hover_ui(|ui| {
                                    ui.label(clear_tooltip);
                                })
                                .clicked()
                            {
                                if confirm_mode {
                                    self.alerts.clear();
                                    self.alert_counts = [0, 0, 0];
                                    self.alert_clear_confirm = None;
                                    self.last_alert_clear_time = Some(Instant::now());
                                    ui.ctx().request_repaint_after(Duration::from_secs(2));
                                } else if self.last_alert_clear_time.is_none()
                                    || self.last_alert_clear_time.unwrap().elapsed().as_secs() >= 2
                                {
                                    self.alert_clear_confirm = Some(Instant::now());
                                    ui.ctx().request_repaint();
                                }
                            }
                            let (icon, tooltip) = if let Some(_t) = self
                                .last_alert_copy_time
                                .filter(|t| t.elapsed().as_secs() < 2)
                            {
                                ("✔", "Copied!")
                            } else {
                                ("📋", "Copy alerts to clipboard")
                            };
                            if ui
                                .button(icon)
                                .on_hover_ui(|ui| {
                                    ui.label(tooltip);
                                })
                                .clicked()
                            {
                                // Bolt Optimization: Pre-calculate size and write to single buffer to avoid O(N) allocations
                                let mut all_alerts = String::with_capacity(self.alerts.len() * 80);
                                for (i, entry) in self.alerts.iter().enumerate() {
                                    if i > 0 {
                                        all_alerts.push('\n');
                                    }
                                    // Use cached alert string
                                    all_alerts.push_str(&entry.text);
                                }
                                ui.output_mut(|o| o.copied_text = all_alerts);
                                self.last_alert_copy_time = Some(Instant::now());
                                ui.ctx().request_repaint_after(Duration::from_secs(2));
                            }
                        });
                    });

                    // Bolt Optimization: Use virtualization for alerts
                    if self.alerts.is_empty() {
                        ui.vertical_centered(|ui| {
                            ui.add_space(20.0);
                            ui.label(egui::RichText::new("✅").size(24.0));
                            ui.label(
                                egui::RichText::new("All Systems Nominal")
                                    .heading()
                                    .color(egui::Color32::GREEN),
                            );
                            ui.label(egui::RichText::new("No active alerts detected").weak());
                        });
                    } else {
                        egui::ScrollArea::both()
                            .id_salt("alerts_scroll")
                            .max_height(300.0)
                            .stick_to_bottom(true)
                            .show_rows(ui, row_height, self.alerts.len(), |ui, row_range| {
                                for i in row_range {
                                    if i % 2 == 1 {
                                        let rect = egui::Rect::from_min_size(
                                            ui.cursor().min,
                                            egui::vec2(ui.available_width(), row_height),
                                        );
                                        ui.painter().rect_filled(
                                            rect,
                                            0.0,
                                            ui.visuals().faint_bg_color,
                                        );
                                    }

                                    let entry = &self.alerts[i];
                                    let color = match entry.event.level {
                                        AlertLevel::Critical => egui::Color32::RED,
                                        AlertLevel::Warning => egui::Color32::YELLOW,
                                        AlertLevel::Info => egui::Color32::LIGHT_BLUE,
                                    };

                                    // Bolt Optimization: Use RichText with cached string
                                    ui.add(egui::Label::new(egui::RichText::new(&entry.text).color(color)).truncate())
                                        .on_hover_ui(|ui| {
                                            Self::render_alert_tooltip(ui, &entry.event);
                                        });
                                }
                            });
                    }
                });
            });

            ui.separator();

            // Manual Input Section
            ui.heading("Manual Packet Injection");
            ui.horizontal(|ui| {
                ui.radio_value(&mut self.input_subsystem, InputSubsystem::Power, "⚡ Power")
                    .on_hover_ui(|ui| {
                        ui.label("Configure Voltage, Current, and Battery parameters");
                    });
                ui.radio_value(
                    &mut self.input_subsystem,
                    InputSubsystem::Thermal,
                    "🌡 Thermal",
                )
                .on_hover_ui(|ui| {
                    ui.label("Configure Temperature sensor parameters");
                });
                ui.radio_value(
                    &mut self.input_subsystem,
                    InputSubsystem::StarTracker,
                    "🔭 Star Tracker",
                )
                .on_hover_ui(|ui| {
                    ui.label("Configure RA, Dec, Confidence, and Target identification");
                });
            });

            // Palette UX Enhancement: Quick Presets
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Presets:").strong());

                let (nom_text, nom_tooltip) = if let Some(_t) = self
                    .last_nominal_apply_time
                    .filter(|t| t.elapsed().as_secs() < 1)
                {
                    ("✔ Restored!", "Nominal values applied")
                } else {
                    ("Nominal 🟢", "Set inputs to safe, nominal values")
                };

                if ui
                    .button(nom_text)
                    .on_hover_ui(|ui| {
                        ui.label(nom_tooltip);
                    })
                    .clicked()
                {
                    self.apply_preset(false);
                    self.last_nominal_apply_time = Some(Instant::now());
                    ui.ctx().request_repaint_after(Duration::from_secs(1));
                }

                let (alert_text, alert_tooltip) = if let Some(_t) = self
                    .last_alert_apply_time
                    .filter(|t| t.elapsed().as_secs() < 1)
                {
                    ("✔ Triggered!", "Alert values applied")
                } else {
                    (
                        "Trigger Alert ⚠",
                        "Set inputs to values that will trigger an alert based on current thresholds",
                    )
                };

                if ui
                    .button(alert_text)
                    .on_hover_ui(|ui| {
                        ui.label(alert_tooltip);
                    })
                    .clicked()
                {
                    self.apply_preset(true);
                    self.last_alert_apply_time = Some(Instant::now());
                    ui.ctx().request_repaint_after(Duration::from_secs(1));
                }
            });

            match self.input_subsystem {
                InputSubsystem::Power => {
                    ui.horizontal(|ui| {
                        ui.label("Voltage:");
                        ui.add(
                            egui::DragValue::new(&mut self.input_voltage)
                                .speed(0.1)
                                .suffix(" V"),
                        )
                        .on_hover_ui(|ui| {
                            ui.label("Bus Voltage (V)");
                        });
                        ui.label("Current:");
                        ui.add(
                            egui::DragValue::new(&mut self.input_current)
                                .speed(0.1)
                                .suffix(" A"),
                        )
                        .on_hover_ui(|ui| {
                            ui.label("Bus Current (A)");
                        });
                        ui.label("Battery:");
                        ui.add(
                            egui::DragValue::new(&mut self.input_battery)
                                .speed(0.1)
                                .range(0.0..=100.0)
                                .suffix(" %"),
                        )
                        .on_hover_ui(|ui| {
                            ui.label("Battery Level (0-100%)");
                        });

                        // Palette UX Enhancement: Visual Battery Bar
                        let battery_pct = self.input_battery as f32 / 100.0;
                        let battery_color = if self.input_battery < self.monitor.min_battery_level {
                            egui::Color32::RED
                        } else if self.input_battery < 50.0 {
                            egui::Color32::YELLOW
                        } else {
                            egui::Color32::GREEN
                        };
                        ui.add(
                            egui::ProgressBar::new(battery_pct)
                                .fill(battery_color)
                                .desired_width(100.0)
                                .show_percentage(),
                        )
                        .on_hover_text("Visual indicator of battery health");

                        if self.input_battery < self.monitor.min_battery_level {
                            // Bolt Optimization: Use colored_label to avoid RichText allocation
                            ui.colored_label(egui::Color32::RED, "⚠").on_hover_ui(|ui| {
                                ui.label(&self.cached_battery_tooltip);
                            });
                        }
                    });
                }
                InputSubsystem::Thermal => {
                    ui.horizontal(|ui| {
                        ui.label("Temperature:");
                        ui.add(
                            egui::DragValue::new(&mut self.input_temp)
                                .speed(0.5)
                                .suffix(" C"),
                        )
                        .on_hover_ui(|ui| {
                            ui.label("Sensor Temperature (°C)");
                        });
                        if self.input_temp > self.monitor.max_temp_celsius {
                            // Bolt Optimization: Use colored_label to avoid RichText allocation
                            ui.colored_label(egui::Color32::YELLOW, "⚠")
                                .on_hover_ui(|ui| {
                                    ui.label(&self.cached_temp_tooltip);
                                });
                        }
                    });
                }
                InputSubsystem::StarTracker => {
                    ui.horizontal(|ui| {
                        ui.label("RA:");
                        ui.add(
                            egui::DragValue::new(&mut self.input_ra)
                                .speed(0.1)
                                .range(0.0..=360.0)
                                .suffix("°"),
                        )
                        .on_hover_text("Right Ascension (0° - 360°)");
                        ui.label("Dec:");
                        ui.add(
                            egui::DragValue::new(&mut self.input_dec)
                                .speed(0.1)
                                .range(-90.0..=90.0)
                                .suffix("°"),
                        )
                        .on_hover_text("Declination (-90° - +90°)");
                    });
                    ui.horizontal(|ui| {
                        ui.label("Confidence:");
                        ui.add(
                            egui::DragValue::new(&mut self.input_confidence)
                                .speed(0.01)
                                .range(0.0..=1.0),
                        )
                        .on_hover_ui(|ui| {
                            ui.label("Star Match Confidence (0.0-1.0)");
                        });

                        // Palette UX Enhancement: Visual Confidence Bar
                        let conf_color =
                            if self.input_confidence < self.monitor.min_star_confidence {
                                egui::Color32::RED
                            } else {
                                egui::Color32::GREEN
                            };
                        ui.add(
                            egui::ProgressBar::new(self.input_confidence as f32)
                                .fill(conf_color)
                                .desired_width(100.0),
                        )
                        .on_hover_text("Visual indicator of star match confidence");

                        if self.input_confidence < self.monitor.min_star_confidence {
                            // Bolt Optimization: Use colored_label to avoid RichText allocation
                            ui.colored_label(egui::Color32::LIGHT_BLUE, "ℹ")
                                .on_hover_ui(|ui| {
                                    ui.label(&self.cached_star_tooltip);
                                });
                        }
                        ui.label("Target:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.input_target)
                                .hint_text("e.g. Sirius")
                                .char_limit(255),
                        )
                        .on_hover_text("Target ID (max 255 characters)");

                        // Palette UX Enhancement: Byte Counter
                        let len = self.input_target.len();
                        let limit = 255;
                        let color = if len > limit {
                            egui::Color32::RED
                        } else if len > 230 {
                            egui::Color32::YELLOW
                        } else {
                            ui.visuals().weak_text_color()
                        };

                        ui.label(egui::RichText::new(format!("{}/{}", len, limit)).color(color))
                            .on_hover_ui(|ui| {
                                ui.label("Protocol limit: 255 bytes.");
                                ui.label("Input exceeding this will be truncated.");
                            });

                        // Palette UX Enhancement: Inline Security Warning
                        if self.input_target.starts_with(&['=', '+', '-', '@'][..]) {
                            ui.colored_label(egui::Color32::YELLOW, "⚠")
                                .on_hover_ui(|ui| {
                                    ui.label(
                                        egui::RichText::new("Sanitization Active")
                                            .strong()
                                            .color(egui::Color32::YELLOW),
                                    );
                                    ui.label("Input starts with a restricted character (=, +, -, @).");
                                    ui.label(
                                        "It will be escaped with a quote (') in logs to prevent CSV injection.",
                                    );
                                });
                        }
                    });
                }
            }

            let (button_text, button_tooltip) = if let Some(_t) = self
                .last_injection_time
                .filter(|t| t.elapsed().as_secs() < 2)
            {
                ("✔ Sent!", "Packet injected successfully")
            } else {
                (
                    "Inject Packet",
                    "Construct and process a telemetry packet with the above values (Ctrl+Enter)",
                )
            };

            if ui
                .button(button_text)
                .on_hover_ui(|ui| {
                    ui.label(button_tooltip);
                })
                .clicked()
                || (ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Enter)))
            {
                let packet = self.create_manual_packet();
                let result = Parser::parse(&packet);
                Self::process_result(
                    &mut self.logs,
                    &mut self.alerts,
                    &mut self.alert_counts,
                    &self.monitor,
                    result,
                    None,
                );
                self.logs_mutation_counter = self.logs_mutation_counter.wrapping_add(1);
                self.last_injection_time = Some(Instant::now());
                ui.ctx().request_repaint_after(Duration::from_secs(2));
            }
        });
    }
}

impl AstroMonitorApp {
    fn update_progress_text(&mut self) {
        Self::format_progress_text(
            &mut self.progress_text,
            self.packet_index,
            self.packets.len(),
            self.simulation_delay_ms,
        );
    }

    fn format_progress_text(buffer: &mut String, current: usize, total: usize, delay: u64) {
        // Bolt Optimization: Reuse the existing string buffer to avoid allocation
        buffer.clear();
        let percentage = if total > 0 {
            (current as f32 / total as f32) * 100.0
        } else {
            0.0
        };

        let remaining_packets = total.saturating_sub(current);
        let remaining_ms = remaining_packets as u64 * delay;
        // Round to nearest second
        let total_seconds = (remaining_ms + 500) / 1000;

        if total_seconds > 60 {
            let m = total_seconds / 60;
            let s = total_seconds % 60;
            let _ = write!(
                buffer,
                "{}/{} ({:.0}%) - {}m {}s left",
                current, total, percentage, m, s
            );
        } else {
            let _ = write!(
                buffer,
                "{}/{} ({:.0}%) - {}s left",
                current, total, percentage, total_seconds
            );
        }
    }

    // Bolt Optimization: Helper to resolve log entry text (deferred formatting)
    fn resolve_log_entry_text<'a>(&self, entry: &'a LogEntry, ui: &egui::Ui) -> ResolvedLogText<'a> {
        match entry {
            LogEntry::SimulatedPacket(idx) => {
                let packet_idx = *idx;
                let id = egui::Id::new("log_fmt").with(packet_idx);

                // Bolt Optimization: Check temporary cache first to avoid re-parsing and re-formatting every frame.
                // egui::util::cache::Memory::data behaves like an LRU cache: data not accessed for a frame is dropped.
                // This is perfect for virtualized lists where only visible items are accessed.
                if let Some(cached) = ui.ctx().data(|d| d.get_temp::<Arc<String>>(id)) {
                    return ResolvedLogText::Shared(cached);
                }

                if let Some(packet_data) = self.packets.get(packet_idx) {
                    // Bolt Optimization: Use parse_trusted to skip checksum validation for internal data
                    // SAFETY: The method is now safe (performs UTF-8 validation).
                    if let Ok(packet) = Parser::parse_trusted(packet_data) {
                        let mut s = String::with_capacity(64);
                        Self::format_log_packet(
                            &mut s,
                            packet.timestamp,
                            Some(packet_idx + 1),
                            &packet.payload,
                        );
                        let arc = Arc::new(s);
                        // Store in cache for next frame
                        ui.ctx().data_mut(|d| d.insert_temp(id, arc.clone()));
                        ResolvedLogText::Shared(arc)
                    } else {
                        ResolvedLogText::Borrowed("Error parsing packet")
                    }
                } else {
                    ResolvedLogText::Borrowed("Invalid Packet Index")
                }
            }
            LogEntry::Packet(s) => ResolvedLogText::Borrowed(s),
            LogEntry::Message(s) => ResolvedLogText::Borrowed(s),
            LogEntry::Alert(s) => ResolvedLogText::Borrowed(s),
        }
    }

    // Bolt Optimization: Helper to recycle string buffers from full log queue
    fn get_recycled_log_buffer(logs: &mut VecDeque<LogEntry>) -> String {
        if logs.len() >= MAX_LOGS {
            if let Some(entry) = logs.pop_front() {
                let mut s = entry.into_string();
                if s.capacity() < 128 {
                    // Not a recycled buffer (e.g. from SimulatedPacket), or was too small.
                    // Allocate new buffer to ensure performant writes.
                    return String::with_capacity(128);
                }
                s.clear();
                return s;
            }
        }
        String::with_capacity(128)
    }

    // Bolt Optimization: Pre-format log packets to avoid formatting in render loop
    fn format_log_packet(
        f: &mut String,
        timestamp: u64,
        index: Option<usize>,
        payload: &TelemetryPayload<'_>,
    ) {
        let s = timestamp % 60;
        let m = (timestamp / 60) % 60;
        let h = (timestamp / 3600) % 24;

        let _ = write!(f, "[{:02}:{:02}:{:02}] ", h, m, s);
        if let Some(idx) = index {
            let _ = write!(f, "Packet {}: ", idx);
        } else {
            let _ = write!(f, "Manual Packet: ");
        }

        match payload {
            TelemetryPayload::Power(d) => {
                let _ = write!(
                    f,
                    "Power(V:{:.1} C:{:.1} B:{:.1}%)",
                    d.voltage, d.current, d.battery_level
                );
            }
            TelemetryPayload::Thermal(d) => {
                let _ = write!(f, "Thermal({:.1}°C)", d.temp_celsius);
            }
            TelemetryPayload::StarTracker(d) => {
                let _ = write!(
                    f,
                    "StarTracker(RA:{:.1} Dec:{:.1} Conf:{:.2}",
                    d.coordinates.right_ascension, d.coordinates.declination, d.confidence
                );
                if let Some(id) = &d.target_id {
                    // Security Fix: Sanitize ID to prevent Log Injection AND CSV Injection
                    let prefix = if id.starts_with(&['=', '+', '-', '@'][..]) {
                        "'"
                    } else {
                        ""
                    };
                    let _ = write!(f, " ID:{}{}", prefix, id.escape_debug());
                }
                let _ = write!(f, ")");
            }
            TelemetryPayload::Unknown => {
                let _ = write!(f, "Unknown");
            }
        }
    }

    fn format_log_alert(f: &mut String, event: &MonitorEvent) {
        let _ = write!(f, "*** ALERT: [{:?}] {} ***", event.level, event.condition);
    }

    fn render_alert_tooltip(ui: &mut egui::Ui, event: &MonitorEvent) {
        let (color, title, icon) = match event.level {
            AlertLevel::Critical => (egui::Color32::RED, "Critical Alert", "🔴"),
            AlertLevel::Warning => (egui::Color32::YELLOW, "System Warning", "⚠️"),
            AlertLevel::Info => (egui::Color32::LIGHT_BLUE, "System Info", "ℹ"),
        };

        ui.heading(egui::RichText::new(format!("{} {}", icon, title)).color(color));
        ui.separator();

        match &event.condition {
            AlertCondition::LowBattery { value, threshold } => {
                ui.label(format!("Battery Level: {:.1}%", value));
                ui.label(format!("Threshold: < {:.1}%", threshold));
                ui.label(
                    egui::RichText::new("Action: Initiate power saving mode.")
                        .weak()
                        .italics(),
                );
            }
            AlertCondition::HighTemperature { value, threshold } => {
                ui.label(format!("Temperature: {:.1}°C", value));
                ui.label(format!("Threshold: > {:.1}°C", threshold));
                ui.label(
                    egui::RichText::new("Action: Check active cooling system.")
                        .weak()
                        .italics(),
                );
            }
            AlertCondition::LowStarConfidence { value, threshold } => {
                ui.label(format!("Confidence: {:.2}", value));
                ui.label(format!("Threshold: < {:.2}", threshold));
                ui.label(
                    egui::RichText::new("Action: Recalibrate star tracker.")
                        .weak()
                        .italics(),
                );
            }
            AlertCondition::SensorFailure { subsystem } => {
                ui.label(format!("Subsystem: {}", subsystem));
                ui.label("Status: Invalid Data / Sensor Failure");
                ui.label(
                    egui::RichText::new("Action: Run diagnostics immediately.")
                        .weak()
                        .italics(),
                );
            }
        }

        ui.separator();
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Timestamp:").strong());
            let ts = event.timestamp;
            let s = ts % 60;
            let m = (ts / 60) % 60;
            let h = (ts / 3600) % 24;
            ui.monospace(format!("{:02}:{:02}:{:02} UTC", h, m, s));
        });
    }

    fn add_log_message(logs: &mut VecDeque<LogEntry>, args: std::fmt::Arguments<'_>) {
        let mut buffer = Self::get_recycled_log_buffer(logs);
        // Bolt Optimization: Write directly to recycled buffer to avoid allocation
        let _ = std::fmt::write(&mut buffer, args);

        if buffer.contains("Error") {
            error!("{}", buffer);
        } else {
            info!("{}", buffer);
        }

        logs.push_back(LogEntry::Message(buffer));
    }

    // Bolt Optimization: Static processing function to allow split borrowing
    fn process_result(
        logs: &mut VecDeque<LogEntry>,
        alerts: &mut VecDeque<AlertEntry>,
        alert_counts: &mut [usize; 3],
        monitor: &Monitor,
        result: Result<TelemetryPacket<'_>, ParserError>,
        index: Option<usize>,
    ) {
        // Bolt Optimization: Combined log message to reduce string allocations and VecDeque operations by 50%
        match result {
            Ok(packet) => {
                // Check for alerts before consuming payload
                let alert_event = monitor.check(&packet);

                if let Some(idx) = index {
                    // Bolt Optimization: Store index only to avoid allocation and formatting overhead
                    if logs.len() >= MAX_LOGS {
                        logs.pop_front();
                    }
                    // packet_index is 0-based. The `index` passed is `packet_index + 1`.
                    // We store `idx - 1` to get the 0-based index into self.packets.
                    logs.push_back(LogEntry::SimulatedPacket(idx - 1));
                    debug!("Processed simulated packet {}", idx);
                } else {
                    // Manual Packet: Format immediately
                    let mut packet_text = Self::get_recycled_log_buffer(logs);
                    Self::format_log_packet(
                        &mut packet_text,
                        packet.timestamp,
                        index,
                        &packet.payload,
                    );
                    info!("{}", packet_text);
                    logs.push_back(LogEntry::Packet(packet_text));
                }

                // Bolt Optimization: Use `check` to get a lightweight MonitorEvent instead of `analyze`
                // which avoids allocating a String for the alert message before it's needed.
                // We format directly into the log and display strings, saving 1 allocation per alert.
                if let Some(event) = alert_event {
                    // Bolt Optimization: Format alert string immediately for log
                    let mut alert_text = Self::get_recycled_log_buffer(logs);
                    Self::format_log_alert(&mut alert_text, &event);

                    match event.level {
                        AlertLevel::Critical => error!("{}", alert_text),
                        AlertLevel::Warning => warn!("{}", alert_text),
                        AlertLevel::Info => info!("{}", alert_text),
                    }

                    logs.push_back(LogEntry::Alert(alert_text));

                    // Bolt Optimization: Store MonitorEvent directly to avoid string formatting and allocation
                    if alerts.len() >= MAX_ALERTS {
                        if let Some(old_entry) = alerts.pop_front() {
                            let old_idx = match old_entry.event.level {
                                AlertLevel::Info => 0,
                                AlertLevel::Warning => 1,
                                AlertLevel::Critical => 2,
                            };
                            if alert_counts[old_idx] > 0 {
                                alert_counts[old_idx] -= 1;
                            }
                        }
                    }

                    let new_idx = match event.level {
                        AlertLevel::Info => 0,
                        AlertLevel::Warning => 1,
                        AlertLevel::Critical => 2,
                    };
                    alert_counts[new_idx] += 1;

                    // Bolt Optimization: Cache formatted alert string to avoid formatting in render loop
                    let ts = event.timestamp;
                    let s = ts % 60;
                    let m = (ts / 60) % 60;
                    let h = (ts / 3600) % 24;
                    let icon = match event.level {
                        AlertLevel::Critical => "🔴",
                        AlertLevel::Warning => "⚠️",
                        AlertLevel::Info => "ℹ️",
                    };
                    let ui_text = format!(
                        "{} [{:?}] {} (Time: {:02}:{:02}:{:02})",
                        icon, event.level, event.condition, h, m, s
                    );

                    alerts.push_back(AlertEntry {
                        event,
                        text: ui_text,
                    });
                }
            }
            Err(e) => {
                if let Some(idx) = index {
                    Self::add_log_message(
                        logs,
                        format_args!("Packet {}: Error parsing: {}", idx, e),
                    );
                } else {
                    Self::add_log_message(
                        logs,
                        format_args!("Manual Packet: Error parsing: {}", e),
                    );
                }
            }
        }
    }

    fn apply_preset(&mut self, trigger_alert: bool) {
        if trigger_alert {
            match self.input_subsystem {
                InputSubsystem::Power => {
                    // Trigger Critical Low Battery
                    self.input_battery = self.monitor.min_battery_level - 5.0;
                    if self.input_battery < 0.0 {
                        self.input_battery = 0.0;
                    }
                }
                InputSubsystem::Thermal => {
                    // Trigger Warning High Temp
                    self.input_temp = self.monitor.max_temp_celsius + 5.0;
                }
                InputSubsystem::StarTracker => {
                    // Trigger Info Low Confidence
                    self.input_confidence = self.monitor.min_star_confidence - 0.1;
                    if self.input_confidence < 0.0 {
                        self.input_confidence = 0.0;
                    }
                }
            }
        } else {
            // Nominal
            match self.input_subsystem {
                InputSubsystem::Power => {
                    self.input_voltage = 28.0;
                    self.input_current = 2.5;
                    self.input_battery = 95.0;
                }
                InputSubsystem::Thermal => {
                    self.input_temp = 25.0;
                }
                InputSubsystem::StarTracker => {
                    self.input_ra = 0.0;
                    self.input_dec = 0.0;
                    self.input_confidence = 1.0;
                    self.input_target = "Sirius".to_string();
                }
            }
        }
    }

    fn create_manual_packet(&self) -> Vec<u8> {
        // Bolt Optimization: Pre-allocate vector to avoid reallocations during packet construction
        let mut packet = Vec::with_capacity(256);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
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

                // Security Fix: Ensure target_id length fits in u8 (max 255 bytes)
                // Truncate to 255 bytes while respecting UTF-8 boundaries
                let mut target_bytes = self.input_target.as_bytes();
                if target_bytes.len() > 255 {
                    // Find the last valid char boundary before/at 255
                    let mut limit = 255;
                    while limit > 0 && !self.input_target.is_char_boundary(limit) {
                        limit -= 1;
                    }
                    target_bytes = &target_bytes[..limit];
                }

                // Calculate len: 3*8 (f64) + 1 (u8) + target_bytes.len()
                let len = 24 + 1 + target_bytes.len() as u16;
                packet.extend_from_slice(&len.to_be_bytes()); // Len

                packet.extend_from_slice(&self.input_ra.to_be_bytes());
                packet.extend_from_slice(&self.input_dec.to_be_bytes());
                packet.extend_from_slice(&self.input_confidence.to_be_bytes());
                packet.push(target_bytes.len() as u8);
                packet.extend_from_slice(target_bytes);
            }
        }
        packet.push(simulation::calculate_checksum(&packet));
        packet
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{PowerData, Subsystem, TelemetryPayload};

    #[test]
    fn test_process_result_formatting() {
        let mut logs = VecDeque::new();
        let mut alerts = VecDeque::new();
        let mut alert_counts = [0, 0, 0];
        let monitor = Monitor::default();

        // 1627849200 = 20:20:00 UTC
        let timestamp = 1627849200;
        let packet = TelemetryPacket {
            timestamp,
            subsystem: Subsystem::Power,
            payload: TelemetryPayload::Power(PowerData {
                voltage: 28.0,
                current: 2.5,
                battery_level: 95.0,
            }),
        };

        // Case 1: Manual Packet (Index None) -> Formatted string
        AstroMonitorApp::process_result(
            &mut logs,
            &mut alerts,
            &mut alert_counts,
            &monitor,
            Ok(packet.clone()),
            None,
        );

        assert_eq!(logs.len(), 1);
        let log_str = logs[0].to_string();
        assert!(log_str.starts_with("[20:20:00]"));
        assert!(log_str.contains("Manual Packet: Power(V:28.0"));

        // Case 2: Simulated Packet (Index Some) -> Index storage
        AstroMonitorApp::process_result(
            &mut logs,
            &mut alerts,
            &mut alert_counts,
            &monitor,
            Ok(packet),
            Some(1),
        );

        assert_eq!(logs.len(), 2);
        match &logs[1] {
            LogEntry::SimulatedPacket(idx) => assert_eq!(*idx, 0),
            _ => panic!("Expected SimulatedPacket for indexed input"),
        }

        // Now test alert formatting (low battery)
        let packet_alert = TelemetryPacket {
            timestamp,
            subsystem: Subsystem::Power,
            payload: TelemetryPayload::Power(PowerData {
                voltage: 24.0,
                current: 1.0,
                battery_level: 10.0, // Low battery
            }),
        };

        AstroMonitorApp::process_result(
            &mut logs,
            &mut alerts,
            &mut alert_counts,
            &monitor,
            Ok(packet_alert),
            Some(2),
        );

        // Alert should be generated
        assert_eq!(alerts.len(), 1);
        let entry = &alerts[0];
        // Check event data via entry.event
        assert_eq!(entry.event.timestamp, timestamp);
        assert_eq!(entry.event.level, AlertLevel::Critical);

        // Check that Alert was added to logs
        // Manual(1) + Simulated(1) + Simulated(1) + Alert(1) = 4 logs
        assert_eq!(logs.len(), 4);
        let alert_log_str = logs[3].to_string();
        assert!(alert_log_str.contains("*** ALERT: [Critical]"));
    }

    #[test]
    fn test_create_manual_packet_truncation() {
        let mut app = AstroMonitorApp::default();
        app.input_subsystem = InputSubsystem::StarTracker;

        // Create a string longer than 255 bytes
        let long_id = "a".repeat(300);
        app.input_target = long_id.clone();

        let packet = app.create_manual_packet();

        // Packet structure:
        // Timestamp (8) + Subsystem (1) + Len (2) + RA (8) + Dec (8) + Conf (8) + ID_Len (1) + ID (...)
        // Header = 11 bytes.
        // Payload start at 11.
        // RA/Dec/Conf = 24 bytes.
        // ID_Len at 11 + 24 = 35.

        let id_len_byte = packet[35];

        // Without fix, this would be 300 % 256 = 44.
        // With fix, it should be 255.

        // Also check the packet length header.
        // Len field is at index 9 (2 bytes).
        let len_bytes: [u8; 2] = packet[9..11].try_into().unwrap();
        let payload_len = u16::from_be_bytes(len_bytes);

        // Payload should be 24 + 1 + id_len.
        // If truncated: 24 + 1 + 255 = 280.
        // If not truncated: 24 + 1 + 300 = 325.

        assert_eq!(id_len_byte, 255, "ID length byte should be capped at 255");
        assert_eq!(
            payload_len, 280,
            "Payload length should reflect truncated size"
        );
        // 11 (Header) + 280 (Payload) + 1 (Checksum)
        assert_eq!(packet.len(), 11 + 280 + 1, "Total packet size should match");
    }

    #[test]
    fn test_format_progress_text() {
        let mut buffer = String::new();

        // Case 1: Start (0/1000), 1000ms delay
        // Remaining: 1000 * 1000ms = 1000s = 16m 40s
        AstroMonitorApp::format_progress_text(&mut buffer, 0, 1000, 1000);
        assert_eq!(buffer, "0/1000 (0%) - 16m 40s left");

        // Case 2: Middle (500/1000)
        // Remaining: 500 * 1000ms = 500s = 8m 20s
        AstroMonitorApp::format_progress_text(&mut buffer, 500, 1000, 1000);
        assert_eq!(buffer, "500/1000 (50%) - 8m 20s left");

        // Case 3: Near End (990/1000), short time
        // Remaining: 10 * 1000ms = 10s
        AstroMonitorApp::format_progress_text(&mut buffer, 990, 1000, 1000);
        assert_eq!(buffer, "990/1000 (99%) - 10s left");

        // Case 4: Finished (1000/1000)
        // Remaining: 0s
        AstroMonitorApp::format_progress_text(&mut buffer, 1000, 1000, 1000);
        assert_eq!(buffer, "1000/1000 (100%) - 0s left");

        // Case 5: Empty (0/0)
        AstroMonitorApp::format_progress_text(&mut buffer, 0, 0, 1000);
        assert_eq!(buffer, "0/0 (0%) - 0s left");

        // Case 6: High Delay (2000ms), 90s left
        // 45 packets left * 2000ms = 90s = 1m 30s
        AstroMonitorApp::format_progress_text(&mut buffer, 55, 100, 2000);
        assert_eq!(buffer, "55/100 (55%) - 1m 30s left");
    }

    #[test]
    fn test_apply_preset() {
        let mut app = AstroMonitorApp::default();

        // 1. Power Subsystem
        app.input_subsystem = InputSubsystem::Power;
        app.apply_preset(false); // Nominal
        assert_eq!(app.input_voltage, 28.0);
        assert_eq!(app.input_battery, 95.0);

        app.apply_preset(true); // Alert
        assert_eq!(app.input_battery, app.monitor.min_battery_level - 5.0);

        // 2. Thermal Subsystem
        app.input_subsystem = InputSubsystem::Thermal;
        app.apply_preset(false); // Nominal
        assert_eq!(app.input_temp, 25.0);

        app.apply_preset(true); // Alert
        assert_eq!(app.input_temp, app.monitor.max_temp_celsius + 5.0);

        // 3. StarTracker Subsystem
        app.input_subsystem = InputSubsystem::StarTracker;
        app.apply_preset(false); // Nominal
        assert_eq!(app.input_confidence, 1.0);
        assert_eq!(app.input_target, "Sirius");

        app.apply_preset(true); // Alert
        assert_eq!(app.input_confidence, app.monitor.min_star_confidence - 0.1);
    }
}

#[cfg(test)]
mod security_tests {
    use super::*;
    use crate::models::{CelestialCoordinates, StarTrackerReading, Subsystem, TelemetryPayload};
    use std::borrow::Cow;

    #[test]
    fn test_log_injection_sanitization() {
        // Construct a packet with a newline in the target ID
        let malicious_id = "Sirius\n[FAKE LOG] Critical Error";
        let packet = TelemetryPacket {
            timestamp: 1234567890,
            subsystem: Subsystem::StarTracker,
            payload: TelemetryPayload::StarTracker(StarTrackerReading {
                target_id: Some(Cow::Borrowed(malicious_id)),
                coordinates: CelestialCoordinates {
                    right_ascension: 0.0,
                    declination: 0.0,
                },
                confidence: 1.0,
            }),
        };

        // Format the log packet
        let mut log_output = String::new();
        AstroMonitorApp::format_log_packet(
            &mut log_output,
            packet.timestamp,
            Some(1),
            &packet.payload,
        );

        // Assert that the newline is ESCAPED (vulnerability fixed)
        assert!(!log_output.contains('\n'), "Security Check Failed: Log output contains a raw newline! Log Injection Vulnerability Present.");
        // The output should contain the escaped form "\n" (backslash n)
        // Note: In Rust string literal, "\\n" represents "\n".
        assert!(
            log_output.contains("\\n"),
            "Log output should contain escaped newline (\\n)"
        );
        assert!(
            log_output.contains("[FAKE LOG]"),
            "Log output should still contain the text content"
        );
    }

    #[test]
    fn test_csv_injection_sanitization() {
        // Construct a packet with a malicious CSV payload in the target ID
        let malicious_id = "=1+1";
        let packet = TelemetryPacket {
            timestamp: 1234567890,
            subsystem: Subsystem::StarTracker,
            payload: TelemetryPayload::StarTracker(StarTrackerReading {
                target_id: Some(Cow::Borrowed(malicious_id)),
                coordinates: CelestialCoordinates {
                    right_ascension: 0.0,
                    declination: 0.0,
                },
                confidence: 1.0,
            }),
        };

        // Format the log packet
        let mut log_output = String::new();
        AstroMonitorApp::format_log_packet(
            &mut log_output,
            packet.timestamp,
            Some(1),
            &packet.payload,
        );

        // Assert that the formula is ESCAPED by prepending a quote
        // The expected behavior is that the output starts with a single quote to prevent execution
        // e.g. "ID:'=1+1" instead of "ID:=1+1"
        assert!(
            log_output.contains("ID:'=1+1"),
            "CSV Injection Vulnerability: Output '{}' should contain escaped formula (ID:'=1+1)",
            log_output
        );
    }
}
