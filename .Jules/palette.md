## 2024-05-22 - Filtered Lists in Egui
**Learning:** When implementing client-side filtering in immediate mode GUI (egui), mapping filtered indices back to the original data structure allows for efficient virtualization without duplicating the data.
**Action:** Use a transient index map (Vec<usize>) in the render loop for filtered views.

## 2024-05-23 - Immediate Mode Feedback Patterns
**Learning:** In immediate mode GUIs (egui), providing transient feedback (like "Copied!") requires storing a timestamp and conditionally rendering text/tooltips based on elapsed time.
**Action:** Use the `last_action_time: Option<Instant>` pattern and `ui.ctx().request_repaint_after()` to ensure the UI updates automatically to revert the state.

## 2024-05-27 - Inline Security Warnings
**Learning:** Displaying a conditional warning icon next to input fields when they contain restricted characters (that will be sanitized) provides immediate feedback and prevents user confusion about log discrepancies.
**Action:** Use conditional `ui.colored_label` with `.on_hover_ui` tooltips for inline validation/sanitization feedback.

## 2024-05-28 - Rich Alert Tooltips
**Learning:** By accessing the underlying `MonitorEvent` struct within `.on_hover_ui()` instead of displaying the pre-formatted string, we can provide actionable recommendations and structured data (Threshold vs Value) without cluttering the main list view.
**Action:** Always prefer structured event data for tooltips to enable rich, context-aware details.

## 2024-05-29 - Input Byte Limits vs Char Limits
**Learning:** In `egui`, `TextEdit::char_limit` restricts Unicode scalars (characters), but network protocols often enforce byte limits. Multi-byte characters (e.g., emojis) can cause inputs to exceed byte limits even if they pass the character limit.
**Action:** For fields with strict protocol byte limits, always display a `.len()` (byte) counter alongside the input field with conditional coloring to warn users of potential truncation.

## 2024-10-24 - Visual Gauges using Progress Bars
**Learning:** egui's `ProgressBar` can be effectively used as a compact visual gauge for scalar inputs by overriding the fill color based on thresholds. This provides immediate "pre-flight" validation feedback without requiring text alerts.
**Action:** Use color-coded progress bars next to numeric inputs where safety thresholds exist.

## 2024-05-29 - Contextual Configuration Units
**Learning:** When configuring raw technical values (like milliseconds delay), providing the derived context (like Hz frequency) in the UI label significantly improves user understanding without cluttering the interface.
**Action:** Use `format!` to append derived units to slider labels or tooltips.

## 2024-10-25 - Disabled Action States for Empty Collections
**Learning:** For actions acting on collections (like clearing or copying lists), users might not realize the actions do nothing if the collection is empty.
**Action:** Dynamically disable the action button and use `on_hover_ui` to show tooltips explaining the reason it's disabled (e.g. "Logs are already empty").

## 2024-10-25 - Transient Button Layout Shifts
**Learning:** In immediate mode GUIs like `egui`, buttons automatically resize to fit their text content. When providing transient feedback (e.g., changing "Action" to "✔ Done"), this causes layout shifts that feel janky to the user.
**Action:** Wrap transient buttons in `ui.add_sized([width, 0.0], ...)` to enforce a fixed width wide enough to accommodate both text states.

## 2024-10-25 - Button Keyboard Shortcut Discoverability
**Learning:** In immediate mode GUIs like `egui`, burying keyboard shortcuts in button tooltips makes them undiscoverable for power users unless they intentionally hover over the action.
**Action:** Use `.shortcut_text()` on `egui::Button` to display keyboard shortcuts directly aligned within the button UI, freeing up tooltip space for functional descriptions.

## 2024-10-25 - Validation-Driven Disabled States
**Learning:** Users shouldn't be able to submit forms or actions when inputs are in an invalid state (like exceeding byte limits), even if the backend/protocol logic handles it gracefully (e.g. via truncation). Silent truncation leads to poor user experience because the user is not explicitly told their input was changed.
**Action:** Disable submission buttons when inputs are invalid, and provide clear tooltips (e.g., `.on_disabled_hover_text()`) explaining why the action is disabled.

## 2024-10-25 - Context Menus for Individual List Items
**Learning:** For lists of items (like logs or alerts) where the user might want to interact with a specific row (e.g. copying a single log entry), a right-click context menu avoids cluttering the UI with individual buttons per row. Furthermore, a hint in the row's hover tooltip (e.g. "Right-click to copy") makes the interaction discoverable.
**Action:** Use `.context_menu()` on list item labels alongside a descriptive tooltip hint for per-item actions.

## 2024-10-25 - Destructive Button Contrast
**Learning:** Using red text on a dark red background for destructive "Confirm" buttons results in a severe WCAG contrast violation, making the text unreadable for visually impaired users.
**Action:** Always use white text (`Color32::WHITE`) on a prominent red background (`Color32::from_rgb(200, 40, 40)`) for critical, destructive confirmation buttons to ensure accessibility and clear signaling.

## 2025-03-10 - Dark Mode Support
**Learning:** Hard-coded or missing dark/light mode toggles make the application uncomfortable to use depending on the user's system preferences or environment. The egui library provides a simple built-in widget for this (`egui::widgets::global_theme_preference_switch`), but it must be explicitly placed in the UI.
**Action:** Add a dark/light mode toggle in the top-right header for better accessibility.

## 2025-03-10 - Disabled Tooltips
**Learning:** `egui::Button::on_hover_ui` and similar hover functions do not trigger when the underlying widget is disabled (`ui.add_enabled(false, ...)`). This hides important explanations for why a button is disabled.
**Action:** Use `.on_disabled_hover_text()` specifically for disabled states, keeping `.on_hover_ui()` or `.on_hover_text()` for enabled states to ensure tooltips are always visible.

## 2025-03-12 - WYSIWYC (What You See Is What You Copy) in Filtered Lists
**Learning:** When users filter a list (e.g., "Important Only") and perform a bulk action like "Copy", they intuitively expect the action to apply only to the visible items. Applying the action to the entire unfiltered dataset violates this expectation and causes frustration.
**Action:** Always ensure bulk actions (Copy, Export) respect the currently active view filters, and update tooltips to explicitly state what is being acted upon (e.g., "Copy visible logs").

## 2025-03-12 - Definitive States for Progress Indicators
**Learning:** Displaying '0s left' when a progress bar reaches 100% feels algorithmic and unfinished.
**Action:** Always provide a definitive text state like 'Completed' or 'Done' when a progress indicator reaches its maximum value.
