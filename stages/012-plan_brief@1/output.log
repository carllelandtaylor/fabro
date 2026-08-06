Now I have all the detail needed. Let me write the brief for the planning stage.The brief is written to `.facto/brief.md`. Here it is:

# Fix planning brief

This brief is the sole input to the planning stage. Plan a FIX for the root cause below — not a feature, not a refactor beyond what the fix requires.

## 1. Root cause

**One-line statement:** Ten call sites across four `fabro-cli` command files style secondary/status table cells with `.foreground_color(color_if(use_color, Color::Ansi256(8)))` — an absolute, non-adaptive 256-color palette index — instead of `cli_table::CellStruct::dimmed(bool)`, the terminal-relative dim/faint (SGR 2) mechanism the crate already provides for exactly this purpose; on dark-background terminals `Ansi256(8)` renders as near-invisible text.

**Supporting trace:**
- `cli_table` 0.5.0's `CellStruct` (via the `Style` trait, already imported in all four affected files) exposes `fn dimmed(self, bool) -> Self`, confirmed at `~/.cargo/registry/.../cli-table-0.5.0/src/style.rs:18,79-82`. It sets `ColorSpec::set_dimmed`, which `termcolor` 1.4.1's `Ansi::set_color` (confirmed `src/lib.rs` ~line 1525) turns into literal `\x1b[2m` (SGR 2, "faint" — relative to the terminal's own foreground), not an absolute palette color.
- The project already has an equivalent terminal-relative primitive in scope in three of the four files: `fabro_util::terminal::Styles::dim` at `lib/foundation/fabro-util/src/terminal.rs:28`. `workflow/list.rs::print_section` already calls `styles.dim.apply_to(...)` two lines above the buggy table cell in the same function — the correct primitive is already in scope there, just not applied to the cell.
- Reproduction confirmed color reaches the output stream correctly (the adjacent NAME cell renders plain `\x1b[36m` cyan in the same row), isolating the defect specifically to the `Ansi256(8)` call sites.

**Files and functions involved (all ten confirmed occurrences):**
- `lib/apps/fabro-cli/src/commands/runs/list.rs` — `fn list_command`: lines 128, 138, 148; `fn status_cell` (lines 171–186): lines 178, 184 (sentinel comparison), 185 — most severe, status conveyed only by unreadable color.
- `lib/apps/fabro-cli/src/commands/model.rs` — `fn model_row`: lines 135, 138.
- `lib/apps/fabro-cli/src/commands/workflow/list.rs` — `fn print_section`: line 113.
- `lib/apps/fabro-cli/src/commands/run/checkpoints.rs` — `fn print_timeline`: line 93.

Shared primitive available for reuse: `fabro_util::terminal::Styles::dim` at `lib/foundation/fabro-util/src/terminal.rs:28`.

**Proposed mechanism:** Replace `.foreground_color(color_if(use_color, Color::Ansi256(8)))` with `.dimmed(use_color)` at all ten sites. `runs/list.rs::status_cell` requires restructuring (introduce an explicit `is_dim` flag) since `Ansi256(8)` also doubles as a bold-suppression sentinel there — not a pure search-and-replace. After the fix, no `Color::Ansi256(8)` should remain anywhere in `lib/apps/fabro-cli/src`.

## 2. Logged reproduction steps (must be re-runnable exactly as recorded)

**Preconditions:** Repo at current branch tip, clean tree, cwd = repo root, no server/settings/credentials needed. `TERM=xterm-256color CLICOLOR_FORCE=1` must be set (sandbox has no real tty; a real dark-background terminal reproduces with no env vars).

**Ordered actions:**
1. `cargo build -p fabro-cli`
2. `TERM=xterm-256color CLICOLOR_FORCE=1 ./target/debug/fabro workflow list 2>&1 | cat -v`

**Observed (wrong):** DESCRIPTION cell emits `\x1b[38;5;8m`; adjacent NAME cell emits normal `\x1b[36m`, proving the defect is isolated to the Ansi256(8) sites.

**Expected:** DESCRIPTION cell (and the other nine sites) emit `\x1b[2m` instead.

**Reliability:** 3/3 deterministic, byte-identical. `fabro ps` (`runs/list.rs`) has an identical code path but requires a server unavailable here — verified statically, must still be covered by fix + test.

The plan's validation must re-run these two commands and confirm `\x1b[38;5;8m` → `\x1b[2m`.

## 3. No-regression constraint (hard, applies to every step)

- `use_color`/`color_if` gating for `NO_COLOR`/non-tty must be preserved — `.dimmed(use_color)` must mirror the existing gating, not bypass it.
- All non-`Ansi256(8)` colors (Cyan, per-status colors) untouched.
- `status_cell`'s status-distinguishing behavior preserved — only the dim/gray mechanism changes.
- No step may delete a color/status case or narrow existing behavior to make the bug disappear.

## 4. Regression test requirement

Add a `cargo nextest` test that forces `use_color = true`, renders the affected cell(s) through the real `cli-table` path, and asserts output contains `\x1b[2m` and not `\x1b[38;5;8m`. Must fail on unfixed code, pass after the fix — demonstrate this as part of validation. Cover both the plain-dim case (e.g. `workflow/list.rs`) and the restructured `status_cell` case.

## 5. Escalation reason

Escalated on **"touches more than about two files"** — four files plus new test coverage, plus a small logic restructuring in `status_cell` (not pure mechanical replace). No new abstractions/API/migration involved — scope the plan accordingly.