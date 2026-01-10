# Bolt's Journal

## 2025-02-18 - [Fix Busy Loop in GUI Update]
**Learning:** `egui`'s `ctx.request_repaint()` causes immediate re-render, leading to 100% CPU usage (busy loop) if used unconditionally in the `update` loop while waiting for a timer.
**Action:** Use `ctx.request_repaint_after(duration)` to schedule the next frame only when necessary, freeing up CPU resources during idle periods.

## 2025-02-19 - [Pre-format Alert Strings]
**Learning:** Immediate mode GUIs (like `egui`) redraw every frame. Using `format!` inside a render loop (even a virtualized one) allocates memory dozens of times per second for visible rows.
**Action:** Pre-format static display strings (like log entries or alerts) when the data is first received/created, and store the formatted string alongside the data. Render using the cached string.
