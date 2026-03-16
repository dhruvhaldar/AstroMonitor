## 2024-05-15 - [Fast Path for Control Character Checks]
**Learning:** Checking for complex multi-byte Unicode control sequences (like BiDi characters) byte-by-byte using an iterator can be extremely slow in Rust when the input string is predominantly ASCII.
**Action:** When validating strings for complex Unicode control characters, always add a fast path first using `.bytes().any()` to check for basic C0/DEL controls, and then check `!id.is_ascii()` before iterating through the string to perform the expensive multi-byte sequence checks. This leverages LLVM autovectorization for the `any()` call and completely bypasses the complex logic for standard ASCII strings, resulting in a ~100x speedup for pure ASCII strings and a ~2.5x speedup for UTF-8 strings.

## 2024-05-15 - [Iterator fold() vs Loop Autovectorization in Checksums]
**Learning:** While historically `.iter().fold()` was considered better for LLVM autovectorization than standard `for` loops in Rust (especially for simple operations like XOR checksums), modern LLVM versions optimize both patterns into identical, highly vectorized machine code in release builds. Micro-benchmarks showed no measurable performance difference between the two approaches for standard telemetry packet sizes (24-255 bytes).
**Action:** Do not prioritize refactoring standard `for` loops to `.iter().fold()` solely for performance reasons unless explicitly bottlenecked by outdated compiler versions. Choose the pattern that best fits the surrounding code's idiomatic style (e.g. functional vs imperative) rather than chasing theoretical micro-optimizations.
## 2024-05-15 - [Avoid Redundant String Formatting in Render Loop]
**Learning:** Operations placed outside of state-changing conditionals in a GUI `update` loop will execute every single frame. Even if these operations reuse buffers and don't allocate memory on the heap (e.g. `write!`), string formatting still consumes CPU cycles for parsing and character writing. Performing this work 60 times a second for static data is a redundant computation bottleneck.
**Action:** Ensure that string formatting operations that depend on simulation state are guarded by a check to verify that the state actually mutated during the current frame (e.g. `if steps > 0`), completely eliminating redundant computations.

## 2024-05-18 - [Beware of Semantics When Removing Bounds Checks]
**Learning:** Removing explicit `.is_finite()` checks on floating point values before range checks (like `.contains()`) changes the semantics of error reporting. While `.contains()` intrinsically evaluates to `false` for `NaN` and `Inf`, omitting `.is_finite()` causes these specific error states to fall through to invalid threshold alerts rather than triggering the proper `SensorFailure` condition.
**Action:** Always verify that a "redundant" check does not map to a distinct, required code path or error condition before removing it. Do not prioritize minor performance gains if they break established error handling logic.

## 2024-05-18 - [Cache Instant::now() Outside Simulation Loops]
**Learning:** Inside a fixed-timestep `while` loop designed to process backlogged simulation steps, calling `Instant::elapsed()` inside the loop condition causes repeated syscalls to `Instant::now()` on every iteration. This adds overhead and can contribute to a "spiral of death" where processing the loop takes enough time to trigger even more iterations.
**Action:** Cache `Instant::now()` immediately *before* the loop starts and use `.saturating_duration_since()` to accurately process the accumulated time backlog without continuously querying the system clock.

## 2024-05-18 - [Eliminate Redundant Bounds Check Branches in Hot Path Parsing]
**Learning:** Using `.map_err()` to convert slice length mismatches into custom errors (e.g. `try_into().map_err(|_| ParserError::BufferTooShort)`) on a hot parser path forces the compiler to maintain the `Err` branch, even if the surrounding code explicitly validated the bounds just one line prior (e.g. `if data.len() < 24`). This adds unnecessary branch evaluation overhead to what could be a simple memory read.
**Action:** When parsing binary structures where the slice boundaries are already mathematically proven and verified to be safe earlier in the function, prefer using `.unwrap()` on `.try_into()` array conversions. This signals to the compiler that the length check will never fail, cleanly eliminating the redundant error branching and yielding a measurable (~10%) performance boost on the hot parsing path.

## 2024-05-18 - [Avoid Redundant String Cloning in egui Render Loop]
**Learning:** `egui`'s `WidgetText` implementation automatically deep-clones `String` objects if a `&String` reference is passed (e.g. `RichText::new(&my_string)`). In virtualized lists or frequently updating components (like `ProgressBar::text`), this creates a massive hidden string allocation bottleneck every single frame per visible item.
**Action:** Always cast owned strings to string slices (`&str`) using `.as_str()` before passing them to `egui` widgets (e.g. `RichText::new(my_string.as_str())`) to bypass the `&String` -> `String` deep-copy semantics and guarantee zero-cost text borrowing on the hot render path.

## 2024-05-18 - [Eliminate Redundant Syscalls in Hot Loops]
**Learning:** Functions that internally utilize system clocks like `last_time.elapsed()` execute a new syscall to `Instant::now()`. Inside a fixed-timestep `while` simulation loop processing multiple packets per frame, this triggers invisible, redundant syscall overhead per generated alert.
**Action:** When a method is called frequently within a simulation loop, pre-calculate `Instant::now()` once before the loop (e.g. `let now = Instant::now()`) and pass `now` as an argument down to the processing functions (e.g. `process_result(..., now)`), utilizing `.saturating_duration_since(*last_time)` to completely eliminate redundant `clock_gettime()` syscall bottlenecks on the hot path.

## 2024-05-18 - [Cache Instant::now() for UI Timeouts in Render Loop]
**Learning:** Using `t.elapsed()` or calling `Instant::now()` directly inside `filter` checks for UI feedback (like "Copied!" timeouts) inside an immediate-mode `update` loop (which executes at 60 FPS) leads to numerous redundant system calls (e.g., `clock_gettime()`) per frame, wasting CPU cycles even when no interaction is occurring.
**Action:** Always pre-calculate `let current_frame_time = Instant::now();` once at the very beginning of the frame's `update` method. Replace all internal `.elapsed()` timeout checks with `current_frame_time.saturating_duration_since(*t)`. This guarantees exactly one clock syscall per frame for all UI timing logic.
