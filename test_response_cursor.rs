use eframe::egui;

fn test_label(ui: &mut egui::Ui) {
    ui.colored_label(egui::Color32::RED, "⚠️")
      .on_hover_cursor(egui::CursorIcon::Help)
      .on_hover_ui(|ui| {
          ui.label("tooltip");
      });
}
