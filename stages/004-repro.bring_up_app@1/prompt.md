Goal: BUG: CLI table text is unreadable on dark terminal themes.

SYMPTOM
Secondary text in `fabro` CLI tables is hardcoded to `Color::Ansi256(8)` ("bright
black" — a fixed dark grey from the palette). That assumes a light terminal
background; on a dark theme it renders nearly invisible against the background.
Reproduced by the user with `fabro workflow list`, where the DESCRIPTION column
is unreadable.

LOCATIONS — 10 occurrences in 4 files, all under
lib/apps/fabro-cli/src/commands/
  runs/list.rs         6  (lines 128, 138, 148, 178, 184, 185)
  model.rs             2  (lines 135, 138)
  workflow/list.rs     1  (line 113)
  run/checkpoints.rs   1  (line 93)

runs/list.rs is the worst case: line 178 uses Ansi256(8) as a *status* color and
line 184 suppresses bold for it, so a run status is conveyed entirely by a color
the user cannot see. This is not purely cosmetic.

SUSPECTED CAUSE
An absolute palette index is used where a terminal-relative attribute belongs.
The codebase already has the right primitive: `fabro_util::terminal::Styles::dim`
= `Style::new().dim()` at lib/foundation/fabro-util/src/terminal.rs:28. It emits
SGR 2 (faint) against the terminal's own foreground, so it adapts to the theme.
workflow/list.rs already uses `styles.dim` for the section path in
`print_section` — the table cells in the same file just don't.

OPEN QUESTION — resolve this before choosing the fix
Does cli-table 0.5's `CellStruct` expose a faint/dim attribute the way it exposes
`.bold(bool)`? Check the actual crate source in the cargo registry rather than
guessing. Three ways it can go:
  a) If CellStruct has a faint/dim attribute, use it. Cleanest.
  b) Otherwise pre-apply the dim style to the string before `.cell()`. Risk:
     cli-table's column width measurement may count the embedded ANSI escape
     codes and mis-align the table. Verify alignment if taking this path.
  c) As a last resort pick a mid-tone color readable on both light and dark
     backgrounds — but this still absolutizes what should be relative.

Note all styling here already goes through the `use_color` flag via
`color_if()`, so the no-color path is unaffected either way.

CONTRIBUTION PATH
Per CONTRIBUTING.md this qualifies as a "bug fix / small improvement" — send a
PR directly, no issue needed. The `origin` remote is the fork
carllelandtaylor/fabro; `upstream` is fabro-sh/fabro and its push URL is
DISABLED, so push branches to `origin` only. Branch from main and open the PR
against fabro-sh/fabro main, e.g.

  gh pr create --repo fabro-sh/fabro --base main --head carllelandtaylor:<branch>

REQUIRED CHECKS (CONTRIBUTING.md's list is stale relative to CI — use these)
  cargo nextest run --workspace
  cargo +nightly-2026-04-14 fmt --check --all
  cargo +nightly-2026-04-14 clippy --workspace --all-targets -- -D warnings
The pinned nightly matters: stable rustfmt will not match CI.

VERIFICATION
No existing test covers this. Add one asserting the faint attribute (or whatever
the chosen mechanism emits) so this cannot regress. Manually verify the rendered
output against both a light and a dark terminal profile.

Run ID: 01KZCGE2MHFXK30C1P69P16B1H
Pipeline progress: 2 of 284 stages completed

## Stage: understand_locate
- Status: succeeded
- Handler: agent
- Response:
  > I'll start by examining the affected files to understand the current implementation.All 10 occurrences confirmed across the 4 named files, matching the report exactly. Now let me check `color_if` in `shared.rs` (the shared helper used by workflow/list.rs, runs/list.rs, run/checkpoints.rs) versus model.rs's local copy, and confirm the `Cell`/`CellStruct` API surface used so I understand what's available for the fix phase.I now have a full trace of the bug. Here's the summary.
  > 
  > ## Expected vs. Actual Behavior
  > 
  > - **Expected:** Secondary/dim table text (descriptions, IDs, aliases, provider names, status for pending/dead runs) should be legible against both light and dark terminal backgrounds.
  > - **Actual:** Ten call sites hardcode `Color::Ansi256(8)` (a fixed dark-grey palette index), which is legible only against light backgrounds and is nearly invisible on dark themes. In `runs/list.rs`, this same fixed grey is also used as the *only* signal for run status (`Submitted`/`Pending`/`Dead`), with bold explicitly suppressed for that case — so on a dark terminal those statuses are effectively unreadable, not just dim.
  > 
  > ## Affected Area
  > 
  > CLI table rendering across four `fabro` commands, all under `lib/apps/fabro-cli/src/commands/`:
  > 
  > | File | Lines | Usage |
  > |---|---|---|
  > | `runs/list.rs` | 128, 138, 148 | RUN ID / PARENT / GOAL columns |
  > | `runs/list.rs` | 178, 184, 185 | `status_cell()` — status color + bold-suppression for `Submitted`/`Pending`/`Dead` |
  > | `model.rs` | 135, 138 | PROVIDER / ALIASES columns (local `color_if` at line 110, distinct from the shared one) |
  > | `workflow/list.rs` | 113 | DESCRIPTION column (the reported repro case) |
  > | `run/checkpoints.rs` | 93 | checkpoint detail column |
  > 
  > Confirmed via `rg "Ansi256(8)"` — exactly 10 matches, matching the report precisely.
  > 
  > ## Root Cause Trace
  > 
  > - All five files build cell colors through `color_if(use_color, Color)` → `Option<Color>`, feeding `cli_table::CellStruct::foreground_color`. `color_if` exists in two places: the shared one at `lib/apps/fabro-cli/src/shared/utilities.rs:166` (used by `runs/list.rs`, `run/checkpoints.rs`, `workflow/list.rs`) and a duplicate local one in `model.rs:110`.
  > - The correct primitive already exists and is proven in the same codebase: `fabro_util::terminal::Styles::dim` (`lib/foundation/fabro-util/src/terminal.rs:28`, `Style::new().dim()`) emits SGR 2 (faint) relative to the terminal's current foreground, so it adapts to any theme. `workflow/list.rs:85,88` already uses `styles.dim` for non-table text in `print_section` — just not for the table cells built at line 113.
  > - `cli-table 0.5.0` is pinned in `Cargo.lock` but its source isn't present in the local `~/.cargo/registry/src` cache (not yet fetched/vendored in this sandbox), so I could not directly inspect `CellStruct`'s attribute surface to resolve the open question (does it expose a faint/dim toggle like `.bold(bool)`, or only `Color`/`.bold`?). That inspection is needed before the fix chooses between options (a) a native dim attribute, (b) pre-styled string + alignment risk, or (c) a mid-tone `Color`. I'll need `cargo fetch`/registry access (or crates.io docs.rs source) in the fix phase to settle this rather than guessing.
  > - All paths already gate through `use_color`/`color_if`, so whichever fix is chosen, the `--no-color`/non-TTY path is unaffected by construction.
  > 
  > ## Optional Status Write
  > 
  > No issue tracker helper (`facto-helper.sh`) is present on this machine, and no issue number was given (free-text bug report). Skipping the status write per the best-effort/warn-and-continue instruction — nothing to update.
  > 
  > Locate phase complete. The fix phase should start by resolving the `cli-table::CellStruct` attribute question (fetch the crate source, e.g. via `cargo fetch` or docs.rs for `cli-table 0.5.0`) before picking between options (a)/(b)/(c) in the bug report.

## Stage: repro.understand_report
- Status: succeeded
- Handler: agent
- Files touched: lib/apps/fabro-cli/src/commands/model.rs, lib/apps/fabro-cli/src/commands/run/checkpoints.rs, lib/apps/fabro-cli/src/commands/runs/list.rs, lib/apps/fabro-cli/src/commands/workflow/list.rs
- Response:
  > I'll check the cli-table crate source to resolve the open question about dim/faint attribute support, then look at the actual bug-report locations.Network access works. Now let's find the cli-table source that was fetched.Confirmed — `CellStruct` implements the `Style` trait which includes `.dimmed(bool)` (cell.rs:199-202), same shape as `.bold(bool)`. Let me check `style.rs` to see how it maps to the terminal escape codes.Resolved: option **(a)**. `CellStruct::dimmed(bool)` exists natively (same shape as `.bold(bool)`) and maps directly to `ColorSpec::set_dimmed()` → SGR 2 (faint), the exact same underlying mechanism as `fabro_util::terminal::Styles::dim`. No ANSI-embedding alignment risk, no color-index guessing. Now let me look at all the call sites.All four files already import `cli_table::Style`, so `.dimmed(bool)` is available everywhere without new imports. Now let's make the fixes.Now the `status_cell` function:Now model.rs:Now workflow/list.rs:Now run/checkpoints.rs:I'll wait for that build in the background and check the test files for adding regression coverage in the meantime.This is still a fresh full workspace build (first build likely). Let me write the regression tests while it compiles in the background.Now let's wait for the build to complete.Good, the pinned nightly is installed. I'll wait for the background build to finish before running fmt/clippy/tests.I'll wait for the build to finish. In the meantime, let's double check the `run_status_kind` usage and confirm nothing else references `Color::Ansi256(8)` that I might have missed.Good — all real occurrences are gone; the only remaining match is the assertion message in my new test. Now waiting for the build to complete.Good, `Color` import stays used everywhere. Now I'll wait for the build to finish (already monitoring in background).I've applied the fix (resolved via the native `.dimmed(bool)` on `CellStruct`, matching the same SGR-2 "faint" mechanism as `Styles::dim`) across all four files and added a regression test in `runs/list.rs`. Now waiting on the background build to finish before running fmt/clippy/tests.I'll pause here and wait for the build-completion notification before proceeding to run the full check suite.


Bring the app up and pick a driver. Find how this project runs: look for a project-specific run skill or instructions first, then the README and docs, and defer to the built-in run skill where it fits the project type. Identify the start command or commands and any prerequisites — dependencies, environment variables, migrations, seed data. Start the necessary services and confirm the app is actually up before driving it. Then pick the driver that matches the affected area: for a WEB UI use the Playwright tools — browser_navigate, browser_snapshot, browser_take_screenshot, and the interaction tools browser_click, browser_type and the rest; for a CLI or API run the command directly or curl the endpoint; for library or internal logic write a small targeted script or a focused test that exercises the code path. Report which driver you chose and confirm the app responded.