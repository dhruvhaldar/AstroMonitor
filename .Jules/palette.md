## 2024-05-26 - Empty States & Visual Progress
**Learning:** For list views (like logs or alerts), a dedicated "empty state" (e.g., centered, dimmed text) is far more reassuring than a blank void, which can look like a rendering bug.
**Action:** When using `ScrollArea::show_rows`, wrap it in an `if !collection.is_empty()` block and provide a helpful fallback UI for the empty case.

## 2024-05-26 - Visualizing Progress
**Learning:** `egui::ProgressBar` is a drop-in replacement for text-based progress counters (`x/y`). It provides immediate visual context ("halfway done") that text alone lacks.
**Action:** Prefer `ProgressBar` over text labels for bounded processes like simulation steps or file downloads.
