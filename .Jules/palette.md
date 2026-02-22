## 2024-05-22 - Filtered Lists in Egui
**Learning:** When implementing client-side filtering in immediate mode GUI (egui), mapping filtered indices back to the original data structure allows for efficient virtualization without duplicating the data.
**Action:** Use a transient index map (Vec<usize>) in the render loop for filtered views.

## 2024-05-23 - Immediate Mode Feedback Patterns
**Learning:** In immediate mode GUIs (egui), providing transient feedback (like "Copied!") requires storing a timestamp and conditionally rendering text/tooltips based on elapsed time.
**Action:** Use the `last_action_time: Option<Instant>` pattern and `ui.ctx().request_repaint_after()` to ensure the UI updates automatically to revert the state.

## 2024-05-27 - Inline Security Warnings
**Learning:** Displaying a conditional warning icon next to input fields when they contain restricted characters (that will be sanitized) provides immediate feedback and prevents user confusion about log discrepancies.
**Action:** Use conditional `ui.colored_label` with `.on_hover_ui` tooltips for inline validation/sanitization feedback.

## 2024-05-28 - Rich Alert Tooltips
**Learning:** By accessing the underlying `MonitorEvent` struct within `.on_hover_ui()` instead of displaying the pre-formatted string, we can provide actionable recommendations and structured data (Threshold vs Value) without cluttering the main list view.
**Action:** Always prefer structured event data for tooltips to enable rich, context-aware details.
