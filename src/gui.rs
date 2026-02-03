use crate::{
    simulation, AlertLevel, Monitor, Parser, ParserError, TelemetryPacket, TelemetryPayload,
};
use eframe::egui;
use std::collections::VecDeque;
use std::fmt::Write;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const MAX_LOGS: usize = 1000;
const MAX_ALERTS: usize = 1000;

#[derive(PartialEq)]
enum InputSubsystem {
    Power,
    Thermal,
    StarTracker,
}

// Bolt Optimization: Helper struct for compact payload formatting
struct CompactPayload<'a>(&'a TelemetryPayload<'a>);

impl<'a> std::fmt::Display for CompactPayload<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            TelemetryPayload::Power(d) => write!(
                f,
                "Power(V:{:.1} C:{:.1} B:{:.1}%)",
                d.voltage, d.current, d.battery_level
            ),
            TelemetryPayload::Thermal(d) => write!(f, "Thermal({:.1}°C)", d.temp_celsius),
            TelemetryPayload::StarTracker(d) => {
                write!(
                    f,
                    "StarTracker(RA:{:.1} Dec:{:.1} Conf:{:.2}",
                    d.coordinates.right_ascension, d.coordinates.declination, d.confidence
                )?;
                if let Some(id) = &d.target_id {
                    write!(f, " ID:{}", id)?;
                }
                write!(f, ")")
            }
            TelemetryPayload::Unknown => write!(f, "Unknown"),
        }
    }
}

pub struct AstroMonitorApp {
    monitor: Monitor,
    packets: Vec<Vec<u8>>,
    packet_index: usize,
    logs: VecDeque<String>,
    alerts: VecDeque<(AlertLevel, String)>,
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
    last_injection_time: Option<Instant>,
    restart_confirm_time: Option<Instant>,

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
            logs: VecDeque::new(),
            alerts: VecDeque::new(),
            alert_counts: [0, 0, 0],
            last_update: Instant::now(),
            simulation_delay_ms: 1000,
            paused: false,
            progress_text,
            last_log_copy_time: None,
            last_alert_copy_time: None,
            last_injection_time: None,
            restart_confirm_time: None,

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
                let result = Parser::parse(&self.packets[self.packet_index]);
                Self::process_result(
                    &mut self.logs,
                    &mut self.alerts,
                    &mut self.alert_counts,
                    &self.monitor,
                    result,
                    Some(self.packet_index + 1),
                );
                self.packet_index += 1;
                self.update_progress_text();
                self.last_update += delay; // Catch up without drift
                steps += 1;
            }

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
                        ui.label("Aggregate system status based on active alerts");
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
                let restart_clicked = if let Some(_t) =
                    self.restart_confirm_time.filter(|t| t.elapsed().as_secs() < 3)
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
                ui.add(
                    egui::Slider::new(&mut self.simulation_delay_ms, 100..=2000).text("Delay (ms)"),
                )
                .on_hover_ui(|ui| {
                    ui.label("Adjust simulation speed (delay between packets in milliseconds)");
                });

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
                        ui.heading("System Logs");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .button("🗑")
                                .on_hover_ui(|ui| {
                                    ui.label("Clear logs");
                                })
                                .clicked()
                            {
                                self.logs.clear();
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
                                // Bolt Optimization: Pre-calculate size and write to single buffer to avoid O(N) allocations
                                let total_len = self.logs.iter().map(|s| s.len()).sum::<usize>()
                                    + self.logs.len().saturating_sub(1);
                                let mut all_logs = String::with_capacity(total_len);
                                for (i, log) in self.logs.iter().enumerate() {
                                    if i > 0 {
                                        all_logs.push('\n');
                                    }
                                    all_logs.push_str(log);
                                }
                                ui.output_mut(|o| o.copied_text = all_logs);
                                self.last_log_copy_time = Some(Instant::now());
                                ui.ctx().request_repaint_after(Duration::from_secs(2));
                            }
                        });
                    });

                    // Bolt Optimization: Use virtualization for logs
                    if self.logs.is_empty() {
                        ui.vertical_centered(|ui| {
                            ui.add_space(20.0);
                            ui.label(egui::RichText::new("📄").size(24.0));
                            ui.label(egui::RichText::new("No System Logs").heading());
                            ui.label(
                                egui::RichText::new("Telemetry events will appear here").weak(),
                            );
                        });
                    } else {
                        egui::ScrollArea::both()
                            .id_salt("logs_scroll")
                            .max_height(300.0)
                            .stick_to_bottom(true)
                            .show_rows(ui, row_height, self.logs.len(), |ui, row_range| {
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
                                    // Ensure fixed height by disabling wrap/truncating
                                    ui.add(egui::Label::new(&self.logs[i]).truncate())
                                        .on_hover_ui(|ui| {
                                            ui.label(&self.logs[i]);
                                        });
                                }
                            });
                    }
                });

                // Alerts Column
                columns[1].vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.heading("Active Alerts");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .button("🗑")
                                .on_hover_ui(|ui| {
                                    ui.label("Clear alerts");
                                })
                                .clicked()
                            {
                                self.alerts.clear();
                                self.alert_counts = [0, 0, 0];
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
                                let total_len =
                                    self.alerts.iter().map(|(_, s)| s.len()).sum::<usize>()
                                        + self.alerts.len().saturating_sub(1);
                                let mut all_alerts = String::with_capacity(total_len);
                                for (i, (_, text)) in self.alerts.iter().enumerate() {
                                    if i > 0 {
                                        all_alerts.push('\n');
                                    }
                                    all_alerts.push_str(text);
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
                                    // Bolt Optimization: Use pre-formatted string to avoid formatting in render loop
                                    let (level, text) = &self.alerts[i];
                                    let color = match level {
                                        AlertLevel::Critical => egui::Color32::RED,
                                        AlertLevel::Warning => egui::Color32::YELLOW,
                                        AlertLevel::Info => egui::Color32::LIGHT_BLUE,
                                    };
                                    // Bolt Optimization: Override text color in visual style to avoid allocation
                                    // (RichText::new(text.clone()) would allocate a new String every frame)
                                    ui.style_mut().visuals.override_text_color = Some(color);
                                    // Ensure fixed height by disabling wrap/truncating
                                    ui.add(egui::Label::new(text).truncate()).on_hover_ui(|ui| {
                                        ui.label(text);
                                    });
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
                        if self.input_battery < self.monitor.min_battery_level {
                            ui.label(egui::RichText::new("⚠").color(egui::Color32::RED))
                                .on_hover_ui(|ui| {
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
                            ui.label(egui::RichText::new("⚠").color(egui::Color32::YELLOW))
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
                        if self.input_confidence < self.monitor.min_star_confidence {
                            ui.label(egui::RichText::new("ℹ").color(egui::Color32::LIGHT_BLUE))
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
                self.last_injection_time = Some(Instant::now());
                ui.ctx().request_repaint_after(Duration::from_secs(2));
            }
        });
    }
}

impl AstroMonitorApp {
    fn update_progress_text(&mut self) {
        // Bolt Optimization: Reuse the existing string buffer to avoid allocation
        self.progress_text.clear();
        let _ = write!(
            self.progress_text,
            "{}/{}",
            self.packet_index,
            self.packets.len()
        );
    }

    fn add_to_log(logs: &mut VecDeque<String>, args: std::fmt::Arguments<'_>) {
        let mut buffer = if logs.len() >= MAX_LOGS {
            logs.pop_front().unwrap_or_default()
        } else {
            String::new()
        };
        buffer.clear();
        // Bolt Optimization: Write directly to recycled buffer to avoid allocation
        let _ = std::fmt::write(&mut buffer, args);
        logs.push_back(buffer);
    }

    // Bolt Optimization: Static processing function to allow split borrowing
    fn process_result(
        logs: &mut VecDeque<String>,
        alerts: &mut VecDeque<(AlertLevel, String)>,
        alert_counts: &mut [usize; 3],
        monitor: &Monitor,
        result: Result<TelemetryPacket<'_>, ParserError>,
        index: Option<usize>,
    ) {
        // Bolt Optimization: Combined log message to reduce string allocations and VecDeque operations by 50%
        match result {
            Ok(packet) => {
                let ts = packet.timestamp;
                let s = ts % 60;
                let m = (ts / 60) % 60;
                let h = (ts / 3600) % 24;

                // Bolt Optimization: Use CompactPayload to reduce log string size and formatting overhead.
                // Replaces verbose debug output (Power { ... }) with compact representation (Power(...))
                if let Some(idx) = index {
                    Self::add_to_log(
                        logs,
                        format_args!(
                            "[{:02}:{:02}:{:02}] Packet {}: {}",
                            h,
                            m,
                            s,
                            idx,
                            CompactPayload(&packet.payload)
                        ),
                    );
                } else {
                    Self::add_to_log(
                        logs,
                        format_args!(
                            "[{:02}:{:02}:{:02}] Manual Packet: {}",
                            h,
                            m,
                            s,
                            CompactPayload(&packet.payload)
                        ),
                    );
                }

                // Bolt Optimization: Use `check` to get a lightweight MonitorEvent instead of `analyze`
                // which avoids allocating a String for the alert message before it's needed.
                // We format directly into the log and display strings, saving 1 allocation per alert.
                if let Some(event) = monitor.check(&packet) {
                    Self::add_to_log(
                        logs,
                        format_args!("*** ALERT: [{:?}] {} ***", event.level, event.condition),
                    );

                    // Bolt Optimization: String recycling for alerts
                    let mut buffer = if alerts.len() >= MAX_ALERTS {
                        if let Some((old_level, old_string)) = alerts.pop_front() {
                            let old_idx = match old_level {
                                AlertLevel::Info => 0,
                                AlertLevel::Warning => 1,
                                AlertLevel::Critical => 2,
                            };
                            if alert_counts[old_idx] > 0 {
                                alert_counts[old_idx] -= 1;
                            }
                            old_string
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    };
                    buffer.clear();

                    // Bolt Optimization: Pre-format display text to avoid allocation in render loop
                    let icon = match event.level {
                        AlertLevel::Critical => "🔴",
                        AlertLevel::Warning => "⚠️",
                        AlertLevel::Info => "ℹ️",
                    };

                    // Write to recycled buffer
                    let _ = write!(
                        buffer,
                        "{} [{:?}] {} (Time: {:02}:{:02}:{:02})",
                        icon, event.level, event.condition, h, m, s
                    );

                    let new_idx = match event.level {
                        AlertLevel::Info => 0,
                        AlertLevel::Warning => 1,
                        AlertLevel::Critical => 2,
                    };
                    alert_counts[new_idx] += 1;
                    alerts.push_back((event.level, buffer));
                }
            }
            Err(e) => {
                if let Some(idx) = index {
                    Self::add_to_log(logs, format_args!("Packet {}: Error parsing: {}", idx, e));
                } else {
                    Self::add_to_log(logs, format_args!("Manual Packet: Error parsing: {}", e));
                }
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

        AstroMonitorApp::process_result(
            &mut logs,
            &mut alerts,
            &mut alert_counts,
            &monitor,
            Ok(packet),
            Some(1),
        );

        assert_eq!(logs.len(), 1);
        // We expect "[20:20:00]" to be at the start
        assert!(logs[0].starts_with("[20:20:00]"));
        // Now using compact formatting, so check for new format
        assert!(logs[0].contains("Packet 1: Power(V:28.0"));

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
        let (_, alert_msg) = &alerts[0];
        // Alert message should contain formatted time
        assert!(alert_msg.contains("(Time: 20:20:00)"));
    }
}
