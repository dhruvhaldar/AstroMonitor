## 2024-05-23 - Inline Validation Feedback
**Learning:** Users benefit significantly from knowing alert thresholds *before* submitting data. In `egui`, conditional rendering of small icons (⚠️, ℹ️) with tooltips next to input fields is a highly effective, low-overhead pattern for inline validation.
**Action:** When designing input forms that have critical thresholds, always check `value` vs `threshold` in the render loop and display a contextual warning icon if the limit is exceeded.

## 2024-05-24 - Status Aggregation
**Learning:** Users need a high-level "at a glance" system health indicator, especially when logs and alerts are scrollable and might hide critical information off-screen.
**Action:** Implement a prominent Status Indicator in the header that aggregates the highest severity of all active alerts. This connects individual events to the overall system state and provides immediate feedback for actions (like clearing alerts).

## 2024-05-25 - Transient Action Feedback
**Learning:** Transient visual feedback (e.g., swapping button text to "✔ Sent!") is essential for confirming manual actions in data-heavy interfaces where the result (a log entry) might be visually noisy or lost.
**Action:** For all manual injection or submission actions, implement a temporary state change (approx 2s) on the trigger element to confirm success without requiring the user to scan logs.

## 2024-05-26 - Zebra Striping for Density
**Learning:** In data-heavy lists (logs/alerts), pure text becomes hard to scan. Zebra striping (alternating background colors) significantly improves readability and row tracking without adding visual clutter.
**Action:** For all virtualized lists (`show_rows`), implement a conditional background fill (`rect_filled`) for odd-indexed rows using a faint background color.
