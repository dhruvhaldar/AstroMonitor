## 2024-05-23 - Inline Validation Feedback
**Learning:** Users benefit significantly from knowing alert thresholds *before* submitting data. In `egui`, conditional rendering of small icons (⚠️, ℹ️) with tooltips next to input fields is a highly effective, low-overhead pattern for inline validation.
**Action:** When designing input forms that have critical thresholds, always check `value` vs `threshold` in the render loop and display a contextual warning icon if the limit is exceeded.

## 2024-05-24 - Status Aggregation
**Learning:** Users need a high-level "at a glance" system health indicator, especially when logs and alerts are scrollable and might hide critical information off-screen.
**Action:** Implement a prominent Status Indicator in the header that aggregates the highest severity of all active alerts. This connects individual events to the overall system state and provides immediate feedback for actions (like clearing alerts).

## 2025-10-26 - Action Confirmation Feedback
**Learning:** For actions that trigger background processes (like manual data injection) where the primary output is a log entry that might be missed, users need immediate confirmation on the control itself.
**Action:** Implement temporary state (timestamp + status) to briefly transform the trigger button into a success/error indicator (e.g., "Injected! ✔") before reverting to its original state.
