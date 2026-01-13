## 2024-05-26 - Empty States & Visual Progress
**Learning:** For list views (like logs or alerts), a dedicated "empty state" (e.g., centered, dimmed text) is far more reassuring than a blank void, which can look like a rendering bug.
**Action:** When using `ScrollArea::show_rows`, wrap it in an `if !collection.is_empty()` block and provide a helpful fallback UI for the empty case.

## 2024-05-26 - Visualizing Progress
**Learning:** `egui::ProgressBar` is a drop-in replacement for text-based progress counters (`x/y`). It provides immediate visual context ("halfway done") that text alone lacks.
**Action:** Prefer `ProgressBar` over text labels for bounded processes like simulation steps or file downloads.

## 2025-02-24 - Keyboard Shortcuts & Focus Safety
**Learning:** Global keyboard shortcuts (like Space to pause) significantly improve usability but can conflict with text inputs.
**Action:** Always wrap global key handlers in `if !ctx.wants_keyboard_input()` to ensure typing in text fields doesn't trigger unintended actions. Update tooltips to announce the shortcut (e.g., "Pause (Space)").

## 2025-02-24 - Anticipatory Tooltips
**Learning:** Tooltips on selection controls (like radio buttons) can "preview" the consequences of the selection (e.g., which form fields will appear). This reduces clicking around just to see what options are available.
**Action:** Add `.on_hover_text()` to radio buttons or dropdown items that trigger significant UI layout changes.

## 2025-02-24 - Input Constraints & Self-Documentation
**Learning:** Explicitly constraining inputs (e.g., `range(0.0..=100.0)`) and documenting those limits in label tooltips makes the interface safer and easier to learn. Users shouldn't have to guess valid ranges.
**Action:** Always add `.range()` to `DragValue` and `.char_limit()` to `TextEdit` where physical or protocol limits exist. Add `on_hover_text` to the label explaining these limits.
