## 2025-02-14 - Replace O(N) Substring Search with O(1) Enum Match in Render Loop
**Learning:** In egui's immediate-mode render loops, performing substring searches (like `.contains("Critical")`) on potentially large strings to determine display state (like color) for every visible row creates a significant O(N) CPU bottleneck per frame.
**Action:** Extract semantic state (such as `is_error` or `AlertLevel`) upon data creation and store it in the core data structure (like `LogEntry`). This transforms the slow O(N) string traversal into a fast O(1) pattern match during the UI render loop.

## 2025-03-02 - [Avoid String allocations in `format!` for static literals in immediate mode UI]
**Learning:** `format!("...{}", ...)` inside an immediate-mode render loop evaluates every frame and allocates a new `String` on the heap, even if the resulting string is relatively static.
**Action:** Lift the logic into conditional variables storing complete static `&str` literals instead of interpolating inside `format!()` macros whenever possible.

## 2024-05-09 - Avoid per-frame `format!` allocations for static UI elements
**Learning:** `format!("{} {}", icon, title)` inside a UI rendering loop like `render_alert_tooltip` allocates a new `String` on the heap every frame when the tooltip is active, despite both variables effectively resolving to static string combinations.
**Action:** Lift the logic to map directly to combined static string literals (`&str`) within a `match` block instead of separating the words and using `format!()` macros. This completely eliminates the heap allocation on this hot path.

## 2025-05-29 - Pre-format dynamic integer arrays to prevent `format!` allocations
**Learning:** Formatting integer arrays dynamically using `format!("{}", self.alert_counts[i])` within an immediate-mode UI render loop will allocate 3 new strings per frame when rendering UI components that use these counts.
**Action:** Add parallel string array caches (`cached_alert_counts_text`) that are updated only when the underlying `alert_counts` change, effectively completely eliminating per-frame allocations.

## 2025-10-18 - Fix memory/cache miss on `get_temp` in egui render loops
**Learning:** In egui, values accessed from the temporary cache via `ui.ctx().data(|d| d.get_temp(...))` are dropped at the end of the frame unless they are explicitly re-inserted using `insert_temp` during the same frame. Returning early on a cache hit without re-inserting causes cache flapping (parsing/allocating every alternate frame).
**Action:** Always re-insert values accessed from egui's temporary cache using `ui.ctx().data_mut(|d| d.insert_temp(id, cached.clone()))` if you return early to ensure they persist for subsequent frames.

## 2025-06-25 - Replace `is_finite` + `contains` with simple boolean bound checks
**Learning:** Checking ranges for floating point numbers using `!val.is_finite() || !(MIN..=MAX).contains(&val)` is slightly slower due to the overhead of trait method calls and bounds checks. A direct simple negated range check like `!(val >= MIN && val <= MAX)` naturally evaluates to `true` (triggering an alert) for NaNs and out of bounds values because any comparison with `NaN` evaluates to `false`.
**Action:** Replace `is_finite` + `contains` bounds checks with combined direct `!(val >= MIN && val <= MAX)` expressions.
## 2025-10-25 - Avoid per-frame string evaluations in immediate-mode rendering loops
**Learning:** In immediate-mode GUIs like `egui`, evaluating complex string functions (like security checks) every frame creates unnecessary CPU overhead. Text fields rarely change compared to the 60+ Hz render loop.
**Action:** Store a boolean cache flag in the application state and only recalculate it when the widget's `.changed()` response is true.
