## 2024-05-23 - Inline Validation Feedback
**Learning:** Users benefit significantly from knowing alert thresholds *before* submitting data. In `egui`, conditional rendering of small icons (⚠️, ℹ️) with tooltips next to input fields is a highly effective, low-overhead pattern for inline validation.
**Action:** When designing input forms that have critical thresholds, always check `value` vs `threshold` in the render loop and display a contextual warning icon if the limit is exceeded.

## 2024-05-24 - Status Aggregation
**Learning:** Users need a high-level "at a glance" system health indicator, especially when logs and alerts are scrollable and might hide critical information off-screen.
**Action:** Implement a prominent Status Indicator in the header that aggregates the highest severity of all active alerts. This connects individual events to the overall system state and provides immediate feedback for actions (like clearing alerts).

## 2024-05-25 - Transient Action Feedback
**Learning:** Transient visual feedback (e.g., swapping button text to "✔ Sent!") is essential for confirming manual actions in data-heavy interfaces where the result (a log entry) might be visually noisy or lost.
**Action:** For all manual injection or submission actions, implement a temporary state change (approx 2s) on the trigger element to confirm success without requiring the user to scan logs.

## 2025-02-18 - Input Field Context in egui
**Learning:** In egui, `DragValue` widgets lack placeholder text support, making context difficult to convey. However, they support tooltips via `.on_hover_ui()`. This is an effective pattern for adding unit/range context to compact numeric inputs.
**Action:** When using compact numeric inputs in egui, always attach a tooltip explaining the unit and valid range.

## 2025-02-25 - Destructive Action Confirmation
**Learning:** For destructive actions (like "Restart Simulation") in immediate mode GUIs, a full modal dialog is often overkill. A "Click-to-Confirm" pattern—where the button changes state (color/text) for a few seconds—provides sufficient safety while maintaining flow.
**Action:** Implement destructive buttons with a transient confirmation state tracked by a timestamp. On first click, arm the button (change to red/warning); on second click (within timeout), execute.

## 2025-05-23 - Improved Empty States
**Learning:** Users often feel unsure if a system is working when they see blank lists (Logs/Alerts). "No logs" can be interpreted as "Broken" rather than "Nothing happened yet".
**Action:** Replace text-only empty states with Icon + Heading + Subtext pattern to provide context and reassurance (especially for "No Alerts").

## 2025-06-15 - Header Information Density
**Learning:** Users monitor lists (Logs/Alerts) more effectively when headers provide quantitative context. A simple count in the header allows users to assess system activity volume without needing to scroll or decipher empty states.
**Action:** Append counts to list headers (e.g., "Logs (12)") to provide immediate, high-level status visibility.

## 2025-07-22 - Icon-Only Button Safety
**Learning:** Icon-only buttons (like Trash 🗑) are high-risk for accidental clicks because they lack descriptive text and occupy small targets. Applying the 'Click-to-Confirm' pattern with a distinct icon change (e.g., to ⚠) is crucial for safety without sacrificing space.
**Action:** For all destructive icon-only buttons, implement a 2-step confirmation with icon swap to prevent data loss.
