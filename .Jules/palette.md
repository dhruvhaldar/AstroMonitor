## 2024-05-23 - Micro-interactions in egui
**Learning:** `egui`'s `.suffix()` on `DragValue` is a clean way to handle units, keeping the label clean and the unit associated with the value. Tooltips on destructive actions (like clearing logs) are essential in immediate mode GUIs where confirmation dialogs can be heavier to implement.
**Action:** Always check if inputs have units and if they can be integrated into the input widget itself. Add tooltips to all icon-only or destructive buttons.

## 2024-05-24 - Simulation Control Visibility
**Learning:** Users need control over time-based simulations. Exposing internal `Duration` parameters as editable `u64` fields (e.g., via sliders) empowers users to adjust pacing for testing or demonstration.
**Action:** Identify hardcoded `Duration` or `Sleep` constants in simulation loops and expose them as UI controls where appropriate.

## 2024-05-25 - Input Guidance
**Learning:** For free-text inputs, `egui::TextEdit::singleline(...).hint_text("...")` provides essential context without cluttering the UI with extra labels. This follows the pattern of providing inline guidance.
**Action:** Use `.hint_text()` for any text input that might be ambiguous or requires specific formatting.
