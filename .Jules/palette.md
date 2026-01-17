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

## 2025-02-25 - Visual Scannability with Icons
**Learning:** In text-heavy monitoring dashboards, prefixing labels and log messages with relevant Unicode icons (e.g., "⚡ Power", "🔴 [Critical]") significantly speeds up visual scanning and reinforces state recognition without consuming extra layout space.
**Action:** Use consistent Unicode icons for subsystem names, control states (Play/Pause), and alert levels in both UI controls and log streams.

## 2025-02-26 - Data Portability for Monitoring
**Learning:** In monitoring tools, users often need to extract logs or alerts for external analysis (e.g., sharing with a colleague or pasting into a report). A simple "Copy to Clipboard" button is a high-value, low-effort addition that prevents frustration.
**Action:** Always provide a "Copy" action (📋) alongside "Clear" actions for text-heavy lists like logs or alerts.

## 2025-02-27 - Form Submission Shortcuts
**Learning:** While global shortcuts must respect focus, "Form Submission" shortcuts (like Ctrl+Enter) *should* function while inputs are focused, as this is a standard power-user pattern.
**Action:** For submit actions, check `ui.input(...)` directly without `!wants_keyboard_input()`, and document the shortcut in the submit button's tooltip.

## 2025-02-27 - Immediate Feedback for Invisible Actions
**Learning:** Actions like "Copy to Clipboard" are invisible. Without visual feedback, users lack confidence that the action succeeded and often click multiple times.
**Action:** Implement a temporary "success state" (e.g., changing the icon to ✔ for 2 seconds) for invisible actions to provide immediate reassurance.
