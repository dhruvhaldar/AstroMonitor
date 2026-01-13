## 2024-05-23 - [Render Loop Allocations]
**Learning:** In immediate mode GUIs like `egui`, operations in the `update()` loop run every frame (e.g., 60fps). Seemingly harmless operations like `format!` inside `ui.add()` create heap allocations on every frame. Caching strings that only change when state changes (like progress indicators) prevents this constant allocation churn.
**Action:** Store formatted strings in the struct and update them only when the underlying data changes, passing a reference (`&self.cached_text`) to the widget.
