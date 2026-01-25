## 2024-05-23 - [Render Loop Allocations]
**Learning:** In immediate mode GUIs like `egui`, operations in the `update()` loop run every frame (e.g., 60fps). Seemingly harmless operations like `format!` inside `ui.add()` create heap allocations on every frame. Caching strings that only change when state changes (like progress indicators) prevents this constant allocation churn.
**Action:** Store formatted strings in the struct and update them only when the underlying data changes, passing a reference (`&self.cached_text`) to the widget.

## 2024-10-24 - [Data Redundancy in GUI State]
**Learning:** When optimizing render loops by pre-formatting strings (caching), it's easy to accidentally duplicate data (storing both the original struct and the formatted string). If the original struct is only used for the initial format, it becomes dead weight.
**Action:** Audit cached display lists. If you store a formatted string, check if you can replace the original heavy struct with a lighter key or enum (e.g., `(Alert, String)` -> `(AlertLevel, String)`).

## 2024-11-20 - [Intermediate Allocations]
**Learning:** Structs used to pass data between systems (e.g., Logic -> GUI) often contain pre-formatted `String` fields for convenience. This forces an allocation even if the consumer immediately re-formats or discards the string.
**Action:** Prefer returning lightweight structs with enums/primitives (`MonitorEvent`) and implement `std::fmt::Display` for them. This allows the consumer to format directly into their final destination (like a log buffer), bypassing the intermediate allocation.

## 2024-11-21 - [Log String Recycling]
**Learning:** High-frequency logging in applications with circular buffers (like `VecDeque<String>`) causes frequent heap allocations and deallocations as messages are pushed and popped.
**Action:** Implement object pooling for log strings: when the buffer is full, pop the old string, clear it, and write the new message into it (using `std::fmt::write`) instead of allocating a new `String`.

## 2024-11-25 - [O(1) Status Checks]
**Learning:** Iterating over large collections (e.g., `alerts.iter().any(...)`) in the render loop to determine global system status creates unnecessary O(N) overhead every frame.
**Action:** Maintain a parallel "counts" array (e.g., `[usize; 3]`) that tracks the number of items per category (Critical, Warning, Info). Update this count only on insertion/removal, enabling O(1) status checks in the render loop.
