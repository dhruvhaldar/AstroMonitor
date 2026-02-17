## 2025-02-18 - [Decoupling Simulation from Rendering]
**Learning:** In a `eframe`/`egui` application, the update loop is tied to the frame rate (vsync). A naive simulation loop that checks `elapsed >= delay` and processes only *one* step per frame will throttle the simulation speed if the frame rate is slower than the simulation frequency (e.g. 100Hz simulation on 60Hz display).
**Action:** Implement a fixed-timestep `while` loop within `update()` that processes multiple simulation steps per frame to catch up with real-time, ensuring simulation throughput is independent of rendering frame rate. Always use `last_update += delay` to prevent time drift.

## 2025-02-18 - [Redundant UI State Updates in Simulation Loops]
**Learning:** Updating UI-specific state (like formatted progress text) inside a tight simulation loop is wasteful because the UI only renders once per frame. If the simulation processes multiple steps per frame (to catch up or fast-forward), intermediate UI state updates are discarded and burn CPU cycles unnecessarily.
**Action:** Move UI state calculation (e.g., `update_progress_text()`) *outside* the simulation loop so it only runs once per frame, reflecting the final state after all simulation steps are processed.

## 2025-02-18 - [Caching Formatted Strings in Immediate Mode GUIs]
**Learning:** In immediate mode GUIs like `egui`, constructing complex strings (e.g., using `format!`) inside the render loop for list items causes allocations every frame for every visible item. This creates significant allocator pressure even when the data hasn't changed.
**Action:** Pre-format and cache display strings in the data model (e.g., `AlertEntry { event, text }`) at the time of creation/update, so the render loop only borrows the string (`&entry.text`).
