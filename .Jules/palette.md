## 2026-06-14 - Egui missing emoji variations
**Learning:** Found that when using text-style emojis in egui, we should add the variation selector `\u{fe0f}`. Found an issue where the `✖` icon on the Target Input's Clear button was missing it, meaning it would not be rendered as a colorful emoji in egui.
**Action:** Append the emoji variation selector to simple emojis, especially the cross mark `✖` -> `✖\u{fe0f}`.
