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

## 2025-02-24 - [Optimize String Concatenation]
**Learning:** `iter().cloned().collect::<Vec<_>>().join("\n")` is a performance anti-pattern. It allocates a new String for every item (clone), a Vec to hold them, and a final String. This is O(N) allocations.
**Action:** Use `String::with_capacity(total_len)` followed by a loop with `push_str()` to perform the operation with exactly 1 allocation and 0 copies (beyond the move to the buffer).

## 2025-02-26 - [Avoid RichText for Simple Colors]
**Learning:** `egui::RichText::new("...")` allocates a `String` even for string literals. In a 60Hz loop, this adds up (e.g., 3 warnings = 180 allocations/sec).
**Action:** Use `ui.colored_label(color, text)` instead of `ui.label(RichText::new(text).color(color))` when standard font weight is sufficient. This avoids the allocation completely for literals.
