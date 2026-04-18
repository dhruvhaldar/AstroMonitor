## 2024-04-16 - Egui Label Cursor Interaction
**Learning:** In `egui`, `egui::Label` elements do not change the mouse cursor on hover by default. To visually signal interactivity (like opening a context menu), explicitly enable hover sensing using `.sense(egui::Sense::hover())` and assign an appropriate cursor using `.on_hover_cursor(...)`. For elements with a `.context_menu()`, `egui::CursorIcon::ContextMenu` is appropriate. For simple tooltips, `egui::CursorIcon::Help` should be used to avoid misleading users.
**Action:** Always test hover interactions for custom UI elements and ensure the correct cursor icon is applied when adding interactivity like context menus or tooltips to non-standard interactive widgets like labels.

## 2024-05-30 - Add Help Cursors to Interactive Labels
**Learning:** In egui, labels with tooltips do not change the mouse cursor on hover by default, hiding their interactivity. This is particularly problematic for status text and warning icons.
**Action:** Use `ui.add(egui::Label::new(...).sense(egui::Sense::hover())).on_hover_cursor(egui::CursorIcon::Help)` when attaching tooltips to labels to provide clear visual feedback.
