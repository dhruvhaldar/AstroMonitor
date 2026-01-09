# Bolt's Journal

## 2025-02-18 - [Fix Busy Loop in GUI Update]
**Learning:** `egui`'s `ctx.request_repaint()` causes immediate re-render, leading to 100% CPU usage (busy loop) if used unconditionally in the `update` loop while waiting for a timer.
**Action:** Use `ctx.request_repaint_after(duration)` to schedule the next frame only when necessary, freeing up CPU resources during idle periods.
