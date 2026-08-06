# Bug brief

## Report source
Free text supplied to the run (not a GitHub issue number or URL).

## Expected behavior
Secondary text in `fabro` CLI tables should be dimmer than primary text but still legible, on both light and dark terminal themes.

## Actual behavior
Secondary text in CLI tables is hardcoded to `Color::Ansi256(8)` ("bright black", a fixed dark-grey palette index). On a terminal with a dark background this renders nearly invisible against the background. On a light theme it reportedly looks correct. Observed with `fabro workflow list`, where the DESCRIPTION column cannot be read.

## Affected area
`fabro` CLI table rendering, under `lib/apps/fabro-cli/src/commands/`. Reporter names: `runs/list.rs`, `model.rs`, `workflow/list.rs`, `run/checkpoints.rs`. Also references the shared styling primitive at `lib/foundation/fabro-util/src/terminal.rs` (`Styles::dim`).

## Code locations
All ten of the reporter's claimed occurrences were read and confirmed exact — every cited line is `Color::Ansi256(8)` used as a secondary/dim foreground color (or, in two `runs/list.rs` cases, as a status color plus a bold-suppression condition keyed off it). No additional occurrences were found; `rg -n "Ansi256\(8\)" lib/apps/fabro-cli/src` returns exactly these ten hits across these four files.

- `lib/apps/fabro-cli/src/commands/runs/list.rs`
  - `fn list_command` (lines 18–169), row-building closure: line 128 (RUN ID cell), line 138 (PARENT cell), line 148 (GOAL cell) — each `.foreground_color(color_if(use_color, Color::Ansi256(8)))`.
  - `fn status_cell` (lines 171–186): line 178 — `Color::Ansi256(8)` used as the STATUS color for `RunStatus::Submitted | Pending | Dead`; line 184 — `.bold(use_color && color != Some(Color::Ansi256(8)))` suppresses bold specifically when the color is `Ansi256(8)`; line 185 — the STATUS cell's `foreground_color`, defaulting to `Ansi256(8)`. This is the case the reporter flagged as most severe: for those three statuses, the status is conveyed *only* by a color that is unreadable on a dark theme.
- `lib/apps/fabro-cli/src/commands/model.rs`
  - `fn model_row` (lines 122–148): line 135 (PROVIDER cell), line 138 (ALIASES cell) — same `.foreground_color(color_if(use_color, Color::Ansi256(8)))` pattern.
  - Note: this file defines its own private `color_if` (lines 110–112), duplicating `crate::shared::utilities::color_if` (`lib/apps/fabro-cli/src/shared/utilities.rs:166`) used by the other three files. Not itself the bug, but relevant if the fix touches `color_if`'s signature or a shared helper is introduced.
- `lib/apps/fabro-cli/src/commands/workflow/list.rs`
  - `fn print_section` (lines 74–136): line 113 — DESCRIPTION cell, `.foreground_color(color_if(use_color, Color::Ansi256(8)))`. Confirmed the reporter's note: this same function already calls `styles.dim.apply_to(...)` for the section title's path suffix (line 85) and the "(none)" placeholder (line 88) — i.e. the terminal-relative primitive is already in scope in this exact function, just not applied to the table cell two lines later.
- `lib/apps/fabro-cli/src/commands/run/checkpoints.rs`
  - `fn print_timeline` (lines 55–118): line 93 — Details cell, `.foreground_color(color_if(use_color, Color::Ansi256(8)))`.

Shared primitive: `fabro_util::terminal::Styles::dim` at `lib/foundation/fabro-util/src/terminal.rs:28` — `Style::new().dim().force_styling(use_color)`, a `console::Style` (SGR 2 "faint" against the terminal's own foreground), confirmed to already exist and already be threaded into three of these four files via a `styles: &Styles` parameter (`model.rs`'s `print_models_table`/`test_models_via_server`, `workflow/list.rs`'s `list_command`/`print_section`, `run/checkpoints.rs`'s `print_timeline` all already receive `&Styles`). `runs/list.rs::list_command` also already receives `styles: &Styles` as a parameter (line 20) but only reads `styles.use_color` from it, not `styles.dim`.

Still open (the reporter's stated open question, unresolved by this stage — no local cargo registry cache was available to inspect `cli-table` 0.5 source, so this is deferred to diagnosis/fix): whether `cli_table::CellStruct` exposes a faint/dim attribute the way it exposes `.bold(bool)`, and whether `cli-table`'s column-width measurement counts embedded ANSI escapes if the alternative of pre-styling the string before `.cell()` is used instead.

## Preconditions, inputs and environment
- A terminal emulator/profile configured with a dark background theme.
- Color output enabled (reporter states all these sites pass through a `use_color` flag via `color_if()`, so this bug does not manifest when color is disabled).
- Reporter did not specify a particular OS, shell, or terminal emulator — general "dark terminal theme" is the stated condition.

## Reproduction steps
1. Configure a terminal with a dark background color theme.
2. Run `fabro workflow list` with color output enabled (default).
3. Observe the DESCRIPTION column (secondary/dim text) in the printed table.
4. Compare legibility against the same command run in a terminal with a light background theme.

Unknown: reporter did not give the exact terminal emulator, exact background hex/theme name, or a captured transcript — steps above are inferred from the description of the symptom, not copied from an explicit numbered repro in the report.

## Evidence
Unknown — no screenshots, log excerpts, or command output were included in the report. The report is a prose description of the symptom plus a table of claimed source locations, not captured evidence.

## Reported hypothesis
The reporter's claims, recorded verbatim/paraphrased as their hypothesis — not verified by this stage:

> Secondary text in `fabro` CLI tables is hardcoded to `Color::Ansi256(8)` ("bright black" — a fixed dark grey from the 256-colour palette). On a terminal with a dark background it renders nearly invisible against the background.

Reporter's claimed occurrence table (unverified):

| file | lines |
| --- | --- |
| `runs/list.rs` | 128, 138, 148, 178, 184, 185 |
| `model.rs` | 135, 138 |
| `workflow/list.rs` | 113 |
| `run/checkpoints.rs` | 93 |

> `runs/list.rs` is the most severe case. Line 178 uses `Ansi256(8)` as a *status* colour and line 184 suppresses bold for that same value, so on a dark theme a run's status is conveyed entirely by a colour the reader cannot see. That makes it a legibility bug rather than a purely cosmetic one.

> An absolute palette index appears to be used where a terminal-relative attribute would be appropriate. `Ansi256(8)` names a fixed colour, so it cannot adapt to the background it is drawn against.

> The repository already contains a terminal-relative primitive: `fabro_util::terminal::Styles::dim` (`Style::new().dim()`) at `lib/foundation/fabro-util/src/terminal.rs:28`, which emits SGR 2 (faint) against the terminal's own foreground colour. `workflow/list.rs` already uses `styles.dim` for the section path in `print_section`; the table cells in that same file do not.

Reported open question (unverified, flagged for diagnosis):

> It is not known whether `cli-table` 0.5's `CellStruct` exposes a faint/dim attribute in the way it exposes `.bold(bool)`. If it does not, one alternative is applying the dim style to the string before `.cell()` — but it is unverified whether `cli-table`'s column-width measurement counts embedded ANSI escape sequences, which would misalign the table.

Reported note (unverified):

> All the styling at these sites already passes through the `use_color` flag via `color_if()`, so terminals with colour disabled are unaffected.

> No existing test covers the colour or attribute of these cells.

No text in the report was phrased as an instruction to this agent (e.g. "run/commit/push this"); it is purely a bug description plus a technical hypothesis.

## Root cause
Unknown — to be established by the diagnosis stage.

## Fix constraints and required checks
Reporter-stated constraints:
- None explicit beyond the technical open question above (dim-attribute support in `cli-table` 0.5, and whether `cli-table` counts embedded ANSI escapes in column-width measurement) — the diagnosis/fix stage should resolve this before choosing an implementation approach.

Standing workflow constraints (apply regardless of what the reporter said):
- Correct the root cause at its source rather than papering over the symptom.
- Never remove, disable, or weaken working functionality to make the symptom go away.
- Add a regression test that fails before the fix and passes after, wherever the project has a test suite. Reporter notes no existing test covers the colour/attribute of these cells, so this will likely require new test coverage.
