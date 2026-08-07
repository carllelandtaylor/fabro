Confirmed against the plan and current repo state:

- **Step 1** (`runs/list.rs`) — implemented and committed (`9cfa0c8` includes it, matches plan).
- **Step 2** (`workflow/list.rs`) — implemented and committed, but validation **failed**: `cargo nextest run -p fabro-cli -- workflow::list` shows `goal_cell_no_color_has_no_escape_bytes` panicking, because the test's `render_cell` helper hardcodes `ColorChoice::Always` instead of branching on `use_color` like production code does.
- **Step 3** (`run/checkpoints.rs`) — not started; `Ansi256(8)` still present at line 93.
- **Step 4** (`model.rs`) — not started; `Ansi256(8)` still present at lines 135, 138.

Ending this stage as **FAILED**. Steps remain: **Step 2 (fix the `workflow/list.rs` test helper's `ColorChoice` bug)** is the immediate next step, and Steps 3 (`run/checkpoints.rs`) and 4 (`model.rs`) are still unimplemented. This is only the first failed validation attempt for Step 2, not the second, so it routes back to the implement stage for a fix rather than being carried forward as an accepted-failing step.