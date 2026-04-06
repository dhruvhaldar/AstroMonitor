## 2025-02-14 - Replace O(N) Substring Search with O(1) Enum Match in Render Loop
**Learning:** In egui's immediate-mode render loops, performing substring searches (like `.contains("Critical")`) on potentially large strings to determine display state (like color) for every visible row creates a significant O(N) CPU bottleneck per frame.
**Action:** Extract semantic state (such as `is_error` or `AlertLevel`) upon data creation and store it in the core data structure (like `LogEntry`). This transforms the slow O(N) string traversal into a fast O(1) pattern match during the UI render loop.
