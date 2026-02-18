## 2024-05-22 - Filtered Lists in Egui
**Learning:** When implementing client-side filtering in immediate mode GUI (egui), mapping filtered indices back to the original data structure allows for efficient virtualization without duplicating the data.
**Action:** Use a transient index map (Vec<usize>) in the render loop for filtered views.

## 2024-05-23 - Immediate Mode Feedback Patterns
**Learning:** In immediate mode GUIs (egui), providing transient feedback (like "Copied!") requires storing a timestamp and conditionally rendering text/tooltips based on elapsed time.
**Action:** Use the `last_action_time: Option<Instant>` pattern and `ui.ctx().request_repaint_after()` to ensure the UI updates automatically to revert the state.
