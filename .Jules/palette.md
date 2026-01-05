## 2024-05-23 - Micro-interactions in egui
**Learning:** `egui`'s `.suffix()` on `DragValue` is a clean way to handle units, keeping the label clean and the unit associated with the value. Tooltips on destructive actions (like clearing logs) are essential in immediate mode GUIs where confirmation dialogs can be heavier to implement.
**Action:** Always check if inputs have units and if they can be integrated into the input widget itself. Add tooltips to all icon-only or destructive buttons.
