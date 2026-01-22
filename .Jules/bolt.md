# Bolt's Journal

## 2025-02-18 - [Fix Busy Loop in GUI Update]
**Learning:** `egui`'s `ctx.request_repaint()` causes immediate re-render, leading to 100% CPU usage (busy loop) if used unconditionally in the `update` loop while waiting for a timer.
**Action:** Use `ctx.request_repaint_after(duration)` to schedule the next frame only when necessary, freeing up CPU resources during idle periods.

## 2025-02-19 - [Pre-format Alert Strings]
**Learning:** Immediate mode GUIs (like `egui`) redraw every frame. Using `format!` inside a render loop (even a virtualized one) allocates memory dozens of times per second for visible rows.
**Action:** Pre-format static display strings (like log entries or alerts) when the data is first received/created, and store the formatted string alongside the data. Render using the cached string.

## 2025-02-21 - [Cache Static Tooltips]
**Learning:** `egui`'s `.on_hover_text(format!(...))` allocates a new String every frame. For tooltips based on static configuration (like monitor thresholds), these strings should be pre-formatted and cached during initialization.
**Action:** Move static formatted strings to struct fields (e.g., `cached_tooltip`) and use `.on_hover_text(&self.cached_tooltip)` in the update loop.

## 2025-02-24 - [O(1) Status Checks & Buffer Recycling]
**Learning:** Iterating over large collections (O(N)) every frame to determine aggregate status (e.g., "Any Critical Alerts?") kills performance. Also, frequent log/alert updates cause steady-state heap fragmentation.
**Action:** Maintain separate "count" variables for aggregate states to allow O(1) checks. Use bounded `VecDeque` with string buffer recycling (popping old string, clearing, and reusing) to zero-out allocations in steady state.
