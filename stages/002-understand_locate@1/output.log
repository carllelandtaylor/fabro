facto-helper.sh isn't present anywhere in this environment, so the optional tracker status write can't be performed — I'll note this as a warning and move on to code verification, per the guardrails (best-effort, non-blocking).

Now let me verify the 10 occurrences and check the cli-table crate source for a dim/faint attribute.All 10 occurrences confirmed matching the report exactly. Now let's inspect the cli-table crate source to answer the open question about a faint/dim attribute on `CellStruct`.## Investigation Summary

**EXPECTED BEHAVIOR:** Secondary/muted text in `fabro` CLI tables (IDs, descriptions, goals, "Dead"/"Pending" status) should remain legibly dim against the terminal's own foreground/background, adapting to both light and dark themes.

**ACTUAL BEHAVIOR:** Secondary text is colored with the absolute palette index `Color::Ansi256(8)` ("bright black"), a fixed dark grey tuned for light backgrounds. On dark themes it's nearly invisible against the background. In `runs/list.rs::status_cell` this is worse than cosmetic: `Ansi256(8)` is also used as the color for `RunStatus::Submitted | Pending | Dead`, and bold is explicitly suppressed for that color (line 184), so those three run statuses are conveyed *only* through a color the user can't see.

**AFFECTED AREA:** `fabro` CLI table rendering — `fabro run list`, `fabro model list`, `fabro workflow list`, `fabro run checkpoints` (or equivalent checkpoint-listing command).

**FILES AND FUNCTIONS TRACED** (all under `lib/apps/fabro-cli/src/commands/`), all 10 occurrences confirmed via grep at the exact lines reported:
- `runs/list.rs` — the run-listing table builder (run-id/parent-id/goal columns at 128/138/148) and `status_cell()` (173–185), where `Ansi256(8)` doubles as both a color and a bold-suppression sentinel for `Submitted`/`Pending`/`Dead` statuses.
- `model.rs` — `model_row()` (122–145): provider and aliases columns at 135/138.
- `workflow/list.rs` — the workflow table row builder inside `print_section` (98–116): DESCRIPTION column at 113. Notably, the same function already dims the section path correctly via `styles.dim.apply_to(...)` at line 85/88 — the table cells are the inconsistent part.
- `run/checkpoints.rs` — the checkpoint-listing row builder (70–96): detail-string column at 93.

All ten sites route through a local `color_if(use_color, color) -> Option<Color>` helper (defined twice, once in `shared/utilities.rs:166` and duplicated in `model.rs:110`), so the no-color path is confirmed unaffected regardless of fix approach.

**OPEN QUESTION — resolved:** Checked the actual `cli-table 0.5.0` source in the cargo registry (`~/.cargo/registry/.../cli-table-0.5.0/src/style.rs` and `cell.rs`). `CellStruct` implements the crate's `Style` trait, which includes `fn dimmed(self, dimmed: bool) -> Self` alongside `.bold(bool)` — it sets `ColorSpec::set_dimmed` on the underlying `termcolor::ColorSpec`, terminal-relative (SGR faint), exactly analogous to `fabro_util::terminal::Styles::dim`. This is **option (a)** from the report: no need to pre-apply ANSI codes into strings (option b) or pick a fixed mid-tone color (option c). The fix is mechanical — replace each `.foreground_color(color_if(use_color, Color::Ansi256(8)))` with `.dimmed(use_color)`, and in `status_cell()` replace the `Ansi256(8)` sentinel/bold-suppression logic with the same `.dimmed()` call so the status text itself carries the dim treatment instead of an invisible color.

**OPTIONAL STATUS WRITE:** Skipped with a warning — `facto-helper.sh` was not found anywhere in this environment (no issue tracker integration present), so the in-progress status update could not be attempted. This is non-blocking per the guardrails.

No code changes were made in this pass — this was the read/locate/triage phase. Ready to proceed to the fix (apply `.dimmed(use_color)` across the four files, add a regression test asserting the dim attribute is emitted, run the required fmt/clippy/nextest checks, and open the PR) on your go-ahead.