`facto-helper.sh` and the `gh` CLI are both unavailable in this sandbox, so the optional tracker status write could not be attempted. This is a non-blocking warning per the instructions — proceeding with the report.

## Summary

**EXPECTED vs ACTUAL**
- Expected: secondary/muted text in CLI tables (IDs, aliases, descriptions, timestamps, and one status value) should be readable on both light and dark terminal backgrounds, adapting to the user's theme.
- Actual: this text is styled with `Color::Ansi256(8)`, an absolute "bright black" palette entry that assumes a light background. On dark themes it's nearly invisible. Worst case: in `runs/list.rs`, `Ansi256(8)` is also used as the *status color* for `Submitted`/`Pending`/`Dead` run states (and bold is suppressed for that color), so those statuses are conveyed almost entirely through a color the user can't see — a functional readability regression, not just cosmetic.

**AFFECTED AREA** — `fabro` CLI table rendering, specifically:
- `fabro runs list` (and its status column)
- `fabro model list` / `fabro model test`
- `fabro workflow list` (DESCRIPTION column — the reproduction case)
- `fabro run checkpoints` (timeline "Details" column)

**FILES / FUNCTIONS TRACED** (all 10 occurrences verified against the report, `grep` confirms exact line matches):
- `lib/apps/fabro-cli/src/commands/runs/list.rs:128,138,148` (row cell dimming for run ID / parent ID / goal), `:171-186` `status_cell()` (status color + the `.bold(... != Some(Color::Ansi256(8)))` suppression)
- `lib/apps/fabro-cli/src/commands/model.rs:122-148` `model_row()` (provider/aliases columns)
- `lib/apps/fabro-cli/src/commands/workflow/list.rs:74-136` `print_section()` (DESCRIPTION column at line 113 — same file already uses `styles.dim` correctly for the section subtitle at line 85)
- `lib/apps/fabro-cli/src/commands/run/checkpoints.rs:55-94` `print_timeline()` (Details column at line 93)
- Shared color-gating: `crate::shared::color_if()` (used by all sites; unaffected by the fix since it only gates whether color/attributes render at all under `NO_COLOR`)
- The correct primitive: `fabro_util::terminal::Styles::dim` at `lib/foundation/fabro-util/src/terminal.rs:28` (`Style::new().dim()`, terminal-relative)

**OPEN QUESTION RESOLVED** — checked the actual `cli-table` 0.5.0 source (via docs.rs source view, matching the pinned `Cargo.lock` version/checksum, not just doc comments): `CellStruct` implements the crate's `Style` trait, which includes `fn dimmed(self, dimmed: bool) -> Self`. This sets a `dimmed: bool` field that `StyleStruct::color_spec()` forwards straight into `termcolor::ColorSpec::set_dimmed(...)` — the same terminal-relative faint/SGR-2 attribute `console::Style::dim()` emits, just via `termcolor` instead of `console`. This confirms **path (a)**: use `.dimmed(use_color)` on the cell builder chain instead of pre-styling strings (avoids path (b)'s ANSI-in-cell width-measurement risk) or picking a fixed mid-tone color (path (c), which would still be absolute). For the status-color case in `runs/list.rs`, the fix should replace the `Ansi256(8)` branch with `None` color plus `.dimmed(use_color)`, removing the now-unneeded `!= Some(Color::Ansi256(8))` bold-suppression special case entirely.

**OPTIONAL STATUS WRITE** — Skipped as a warning: neither `facto-helper.sh` nor `gh` is present in this sandbox, so tracker detection, current-issue lookup, and status field/value resolution could not be performed. No GitHub Issue tracker interaction occurred; this did not block investigation.

This turn covered understanding the bug and locating/confirming the exact code, plus resolving the crate-capability open question needed to choose the fix. No source files were modified yet — say the word and I'll implement the `.dimmed()` fix across the four files, adjust the `runs/list.rs` status-color branch, add a regression test asserting the dim/faint SGR sequence, and run the required `nextest`/`fmt`/`clippy` checks.