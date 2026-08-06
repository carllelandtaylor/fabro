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
- Color output enabled. Confirmed by driving the actual built binary: `Styles::detect_stderr()` (used by `workflow list.rs`) calls `console::colors_enabled_stderr()`, which is gated on `env::var("CLICOLOR_FORCE")` (or a real stderr tty) — plain piping/no-tty disables it. Separately and *in addition*, `cli_table`'s own `ColorChoice::Auto` (set from the same `use_color` bool at each call site) is gated on `termcolor`'s `env_allows_color()`, which checks `TERM` (must not be unset/`"dumb"`) and `NO_COLOR` (must be unset) — this check is not tty-based at all, it's pure env var. Both gates must pass for table-body ANSI color to actually appear in the output stream. Reproduced by running the built CLI with `TERM=xterm-256color CLICOLOR_FORCE=1` under a `script`-allocated pty.
- Reporter did not specify a particular OS, shell, or terminal emulator — general "dark terminal theme" is the stated condition. Not needed for repro: the emitted SGR sequence is visible/verifiable directly in captured output without an actual dark-background display.

## Reproduction steps (minimized, reliably reproduces — confirmed 3/3 runs)

This is reduced to the smallest sequence that still triggers the bug: no server, no `fabro install`/`fabro server start`, no populated `goal` fields, and no real pty are required. Removing any one of the two commands below, or dropping either env var, either loses the ability to observe color output at all (this sandbox has no real tty) or fails to build the binary. Both are load-bearing; nothing else is.

**PRECONDITIONS AND DATA**
- Repo checked out at the current branch tip, working tree clean.
- Cwd = repo root (so `.fabro/workflows/*/workflow.toml` — 17 project workflow definitions already present in the repo — are discovered as "Project Workflows"). No fixture data needs to be created; the checked-in workflow files are sufficient, and none of them need a non-empty `goal` field for the bug to manifest (see step 3).
- No running Fabro server, no `settings.toml`, no account/credentials required — `fabro workflow list` never contacts a server.
- Environment variables `TERM=xterm-256color` and `CLICOLOR_FORCE=1` must be set on the command in step 2. Reason: this sandbox has no real tty, so two independent color-enable gates must be forced: `console::colors_enabled_stderr()` (gates the pre-styled title text, checks `CLICOLOR_FORCE` or a real tty) and `cli_table`'s `ColorChoice::Auto` (gates the table body, checks `termcolor::env_allows_color()` — requires `TERM` to be set and not `"dumb"`, and `NO_COLOR` unset). On a real terminal with a genuine dark-background tty, plain `fabro workflow list` reproduces without any env vars — the env vars are a sandbox-only substitute for "has a tty with color support," not part of the bug's trigger condition itself.

**ORDERED ACTIONS**
1. `cargo build -p fabro-cli` — produces `target/debug/fabro`. (One-time; not required on subsequent runs once the binary exists.)
2. `TERM=xterm-256color CLICOLOR_FORCE=1 ./target/debug/fabro workflow list 2>&1 | cat -v`

That is the complete minimal sequence — a single command after the build. No further input, flags, or interaction is needed.

**OBSERVED (WRONG) RESULT**
Every row under "Project Workflows" emits, for the DESCRIPTION cell:
```
^[[0m^[[0m ^[[0m^[[0m^[[0m^[[38;5;8m           ^[[0m ^[[0m
```
i.e. literal escape `\x1b[38;5;8m` — SGR "set foreground to 256-color palette index 8," an absolute dark-grey palette color, exactly `Color::Ansi256(8)` as reported. This color does not adapt to the terminal's actual background; on a dark-themed terminal it renders text that is nearly the same color as the background (per reporter) and is effectively unreadable.

Contrast check confirms this isn't a global color-suppression artifact: the adjacent NAME cell in the identical row renders `^[[36m` (SGR 36, plain ANSI cyan) — a normal, terminal-relative color — proving color reaches the output stream correctly and the DESCRIPTION column's `38;5;8` is the specific defect, not an environment fluke.

**EXPECTED RESULT**
The DESCRIPTION cell (and the other nine confirmed call sites — see Code locations) should use a terminal-relative "dim/faint" attribute (SGR 2, `\x1b[2m`) instead of an absolute palette color, so it renders as a dimmed variant of whatever foreground the user's terminal theme actually uses — legible on both light and dark backgrounds. The repo already has this primitive in scope (`fabro_util::terminal::Styles::dim`, and `cli_table::CellStruct::dimmed(bool)` — see Evidence), it's just not applied at these ten sites.

**RELIABILITY**
Reproduces deterministically, 3/3 consecutive runs, identical output each time (byte-identical DESCRIPTION-cell escape sequence). Not intermittent.

Local project workflows (`.fabro/workflows/*/workflow.toml`) all have empty `goal` fields (confirmed via `grep -rn "goal" .fabro/workflows/*/workflow.toml` — zero matches across all 17 files), so the DESCRIPTION *cell text* itself is blank in this repro — the escape sequence still wraps the (empty) cell content identically, which is sufficient to confirm the styling call site fires with the hardcoded color, but there is no local data to produce a directly human-legible "dark grey text on dark background" screenshot with visible characters. `fabro ps` (the `runs/list.rs` command, which is reachable with populated cells including STATUS) requires a running Fabro server (`fabro server start` / `fabro install`) not available in this environment (`Failed to start fabro server ... no settings.toml configured`) — not exercised live; its code path is identical (`color_if(use_color, Color::Ansi256(8))` / same status-color match) and was verified statically instead.

Unknown: reporter did not give the exact terminal emulator, exact background hex/theme name, or a captured transcript from their own machine — the steps above are an independently executed repro of the same underlying mechanism (hardcoded `Color::Ansi256(8)` reaching the output stream), not a replay of the reporter's own session.

## Evidence
- Raw captured output at `/tmp/wf_list_raw.txt` (earlier pty capture via `script`) and `/tmp/repro_confirm.txt` (final confirmation run, this stage — 2 lines: the DESCRIPTION header row and the `card-game` row) showing `\x1b[38;5;8m` emitted around the DESCRIPTION cell in `fabro workflow list` table rows, contrasted with `\x1b[36m` on the adjacent NAME cell in the same row. Exit code of the confirmation run: `0` (command succeeds; the bug is a rendering defect, not a crash).
- Confirmed via `cli-table` 0.5.0 crate source (`~/.cargo/registry/.../cli-table-0.5.0/src/style.rs`, `cell.rs`) that `CellStruct` (via the `Style` trait, already imported in all four affected files) exposes `fn dimmed(self, bool) -> Self`, which sets `termcolor::ColorSpec.dimmed`. Confirmed via `termcolor` 1.4.1 source (`src/lib.rs` ~line 1525, `Ansi::set_color`) that `spec.dimmed` emits literal `\x1B[2m` (SGR 2, "faint") — a terminal-relative attribute, not an absolute palette color — with no ANSI-in-cell-width concern since it's set via the `Style` trait's structured `ColorSpec`, not by pre-styling the string before `.cell()`. This resolves the reporter's open technical question: `cli-table` 0.5's `CellStruct` *does* support a dim/faint attribute directly, so no string pre-styling + width-measurement workaround is needed.

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
