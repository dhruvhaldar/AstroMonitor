## 2026-06-14 - Egui missing emoji variations
**Learning:** Found that when using text-style emojis in egui, we should add the variation selector `\u{fe0f}`. Found an issue where the `✖` icon on the Target Input's Clear button was missing it, meaning it would not be rendered as a colorful emoji in egui.
**Action:** Append the emoji variation selector to simple emojis, especially the cross mark `✖` -> `✖\u{fe0f}`.

## 2026-06-14 - Disabled State UX for Form Presets
**Learning:** Found that static form preset buttons ("Nominal" or "Trigger Alert") do not provide intuitive feedback when clicked repeatedly if the inputs are already set correctly. The lack of visual state causes user confusion.
**Action:** Use `ui.add_enabled_ui(!is_preset_active)` and `.on_disabled_hover_text(...)` to dynamically disable preset buttons and provide actionable explanations when form values already match the target preset state.
## 2024-06-20 - Use Custom Cursors for Help Buttons in egui
**Learning:** In `egui`, standard buttons (`egui::Button`) use the default pointer cursor on hover. When a button's primary action is to display an informational tooltip (like a `?` icon) rather than trigger a functional state change, relying solely on the hover text isn't enough semantic feedback.
**Action:** Chain `.on_hover_cursor(egui::CursorIcon::Help)` immediately after creating the button element (and before `.on_hover_ui`) to explicitly change the cursor to a question mark icon, making the interface more intuitive.
