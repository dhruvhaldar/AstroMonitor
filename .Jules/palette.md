## 2024-05-23 - Inline Validation Feedback
**Learning:** Users benefit significantly from knowing alert thresholds *before* submitting data. In `egui`, conditional rendering of small icons (⚠️, ℹ️) with tooltips next to input fields is a highly effective, low-overhead pattern for inline validation.
**Action:** When designing input forms that have critical thresholds, always check `value` vs `threshold` in the render loop and display a contextual warning icon if the limit is exceeded.

## 2024-05-24 - Status Aggregation
**Learning:** Users need a high-level "at a glance" system health indicator, especially when logs and alerts are scrollable and might hide critical information off-screen.
**Action:** Implement a prominent Status Indicator in the header that aggregates the highest severity of all active alerts. This connects individual events to the overall system state and provides immediate feedback for actions (like clearing alerts).

## 2026-01-22 - Action Confirmation Feedback
**Learning:** For discrete actions that don't produce immediate visual changes (like 'Inject Packet' or 'Copy'), users lack confidence that the action succeeded. Temporary state changes on the action trigger itself (e.g. changing button text to "✔ Sent!") provide critical reassurance without needing invasive toast notifications.
**Action:** For manual triggers, implement a `last_action_time` state and transiently replace the button label/icon with a success indicator for ~2 seconds.
