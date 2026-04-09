## 2025-02-14 - Replace O(N) Substring Search with O(1) Enum Match in Render Loop
**Learning:** In egui's immediate-mode render loops, performing substring searches (like `.contains("Critical")`) on potentially large strings to determine display state (like color) for every visible row creates a significant O(N) CPU bottleneck per frame.
**Action:** Extract semantic state (such as `is_error` or `AlertLevel`) upon data creation and store it in the core data structure (like `LogEntry`). This transforms the slow O(N) string traversal into a fast O(1) pattern match during the UI render loop.

## 2025-03-02 - [Avoid String allocations in `format!` for static literals in immediate mode UI]
**Learning:** `format!("...{}", ...)` inside an immediate-mode render loop evaluates every frame and allocates a new `String` on the heap, even if the resulting string is relatively static.
**Action:** Lift the logic into conditional variables storing complete static `&str` literals instead of interpolating inside `format!()` macros whenever possible.
