I'll verify the reporter's claimed code locations by reading the actual files, since the brief currently leaves "Code locations" as Unknown.All ten claimed locations check out exactly. Let me also verify the `cli-table` crate version and API surface to resolve the reporter's open question about dim attribute support.All confirmed. Now updating the bug brief with verified code locations.Verified all ten of the reporter's claimed occurrences and updated `.facto/bug-brief.md` in place.

**Affected area:** `fabro` CLI table rendering in `lib/apps/fabro-cli/src/commands/` — `runs/list.rs`, `model.rs`, `workflow/list.rs`, `run/checkpoints.rs` — plus the shared styling primitive `fabro_util::terminal::Styles` (`lib/foundation/fabro-util/src/terminal.rs`).

**Traced and confirmed exact** (via `rg -n "Ansi256\(8\)" lib/apps/fabro-cli/src`, which returns precisely these ten hits, no more):
- `runs/list.rs`: `list_command` (lines 128, 138, 148 — RUN ID/PARENT/GOAL cells) and `status_cell` (lines 178, 184, 185 — status color, bold-suppression, and default color, the most severe case since status becomes unreadable-only)
- `model.rs`: `model_row` (lines 135, 138 — PROVIDER/ALIASES cells); also noted this file has its own private `color_if` duplicating the shared one in `shared/utilities.rs:166`
- `workflow/list.rs`: `print_section` (line 113 — DESCRIPTION cell); confirmed `styles.dim` is already used two lines earlier in the same function for the section header, just not applied to the table cell
- `run/checkpoints.rs`: `print_timeline` (line 93 — Details cell)
- `Styles::dim` at `terminal.rs:28`, and confirmed three of the four files already carry a `&Styles` parameter into scope (`runs/list.rs` does too, but only reads `.use_color` from it)

**Still `Unknown` in the brief:** exact reproduction transcript/terminal emulator (reporter never gave one), Evidence (no screenshots/logs provided), Root cause (diagnosis stage's job), and the reporter's open technical question about whether `cli-table` 0.5's `CellStruct` supports a dim attribute — I couldn't inspect the crate source locally (no cargo registry cache present), so that's deferred to diagnosis/fix.