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
Unknown — not verified at this stage (locating and confirming code is a job of later stages). The reporter's claimed locations are recorded verbatim below under Reported hypothesis for the diagnosis stage to verify.

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
