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

## 2024-05-29 - Input Byte Limits vs Char Limits
**Learning:** In `egui`, `TextEdit::char_limit` restricts Unicode scalars (characters), but network protocols often enforce byte limits. Multi-byte characters (e.g., emojis) can cause inputs to exceed byte limits even if they pass the character limit.
**Action:** For fields with strict protocol byte limits, always display a `.len()` (byte) counter alongside the input field with conditional coloring to warn users of potential truncation.

## 2024-10-24 - Visual Gauges using Progress Bars
**Learning:** egui's `ProgressBar` can be effectively used as a compact visual gauge for scalar inputs by overriding the fill color based on thresholds. This provides immediate "pre-flight" validation feedback without requiring text alerts.
**Action:** Use color-coded progress bars next to numeric inputs where safety thresholds exist.

## 2024-05-29 - Contextual Configuration Units
**Learning:** When configuring raw technical values (like milliseconds delay), providing the derived context (like Hz frequency) in the UI label significantly improves user understanding without cluttering the interface.
**Action:** Use `format!` to append derived units to slider labels or tooltips.
