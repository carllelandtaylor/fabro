I'll start by examining the affected files to understand the current implementation.All 10 occurrences confirmed across the 4 named files, matching the report exactly. Now let me check `color_if` in `shared.rs` (the shared helper used by workflow/list.rs, runs/list.rs, run/checkpoints.rs) versus model.rs's local copy, and confirm the `Cell`/`CellStruct` API surface used so I understand what's available for the fix phase.I now have a full trace of the bug. Here's the summary.

## Expected vs. Actual Behavior

- **Expected:** Secondary/dim table text (descriptions, IDs, aliases, provider names, status for pending/dead runs) should be legible against both light and dark terminal backgrounds.
- **Actual:** Ten call sites hardcode `Color::Ansi256(8)` (a fixed dark-grey palette index), which is legible only against light backgrounds and is nearly invisible on dark themes. In `runs/list.rs`, this same fixed grey is also used as the *only* signal for run status (`Submitted`/`Pending`/`Dead`), with bold explicitly suppressed for that case — so on a dark terminal those statuses are effectively unreadable, not just dim.

## Affected Area

CLI table rendering across four `fabro` commands, all under `lib/apps/fabro-cli/src/commands/`:

| File | Lines | Usage |
|---|---|---|
| `runs/list.rs` | 128, 138, 148 | RUN ID / PARENT / GOAL columns |
| `runs/list.rs` | 178, 184, 185 | `status_cell()` — status color + bold-suppression for `Submitted`/`Pending`/`Dead` |
| `model.rs` | 135, 138 | PROVIDER / ALIASES columns (local `color_if` at line 110, distinct from the shared one) |
| `workflow/list.rs` | 113 | DESCRIPTION column (the reported repro case) |
| `run/checkpoints.rs` | 93 | checkpoint detail column |

Confirmed via `rg "Ansi256(8)"` — exactly 10 matches, matching the report precisely.

## Root Cause Trace

- All five files build cell colors through `color_if(use_color, Color)` → `Option<Color>`, feeding `cli_table::CellStruct::foreground_color`. `color_if` exists in two places: the shared one at `lib/apps/fabro-cli/src/shared/utilities.rs:166` (used by `runs/list.rs`, `run/checkpoints.rs`, `workflow/list.rs`) and a duplicate local one in `model.rs:110`.
- The correct primitive already exists and is proven in the same codebase: `fabro_util::terminal::Styles::dim` (`lib/foundation/fabro-util/src/terminal.rs:28`, `Style::new().dim()`) emits SGR 2 (faint) relative to the terminal's current foreground, so it adapts to any theme. `workflow/list.rs:85,88` already uses `styles.dim` for non-table text in `print_section` — just not for the table cells built at line 113.
- `cli-table 0.5.0` is pinned in `Cargo.lock` but its source isn't present in the local `~/.cargo/registry/src` cache (not yet fetched/vendored in this sandbox), so I could not directly inspect `CellStruct`'s attribute surface to resolve the open question (does it expose a faint/dim toggle like `.bold(bool)`, or only `Color`/`.bold`?). That inspection is needed before the fix chooses between options (a) a native dim attribute, (b) pre-styled string + alignment risk, or (c) a mid-tone `Color`. I'll need `cargo fetch`/registry access (or crates.io docs.rs source) in the fix phase to settle this rather than guessing.
- All paths already gate through `use_color`/`color_if`, so whichever fix is chosen, the `--no-color`/non-TTY path is unaffected by construction.

## Optional Status Write

No issue tracker helper (`facto-helper.sh`) is present on this machine, and no issue number was given (free-text bug report). Skipping the status write per the best-effort/warn-and-continue instruction — nothing to update.

Locate phase complete. The fix phase should start by resolving the `cli-table::CellStruct` attribute question (fetch the crate source, e.g. via `cargo fetch` or docs.rs for `cli-table 0.5.0`) before picking between options (a)/(b)/(c) in the bug report.