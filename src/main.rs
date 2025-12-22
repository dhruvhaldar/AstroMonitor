use astro_monitor::gui::AstroMonitorApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    env_logger::init();
    println!("Starting Astro Monitor GUI...");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Astro Monitor",
        options,
        Box::new(|_cc| Ok(Box::new(AstroMonitorApp::default()))),
    )
}
