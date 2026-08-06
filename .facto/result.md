# Reproduction report

## Bug
Secondary/dim text in `fabro` CLI tables (`fabro workflow list`, `fabro ps`, `fabro model list`, `fabro run checkpoints`) is styled with a hardcoded absolute palette color, `Color::Ansi256(8)` ("bright black"), instead of a terminal-relative dim/faint attribute. On a terminal with a dark background this renders the text nearly invisible against the background. In `runs/list.rs`, the same hardcoded color is also used as the STATUS color for `Submitted`/`Pending`/`Dead` runs, and bold is explicitly suppressed for that color — so for those statuses, run state is conveyed only through a color that can be unreadable.

Ten confirmed call sites, all using `.foreground_color(color_if(use_color, Color::Ansi256(8)))` (or, in two `runs/list.rs` cases, the equivalent status-color match):
- `lib/apps/fabro-cli/src/commands/runs/list.rs:128,138,148,178,184,185`
- `lib/apps/fabro-cli/src/commands/model.rs:135,138`
- `lib/apps/fabro-cli/src/commands/workflow/list.rs:113`
- `lib/apps/fabro-cli/src/commands/run/checkpoints.rs:93`

## Environment / preconditions
- Repo checked out at current branch tip, working tree clean, cwd = repo root.
- No running Fabro server, no `settings.toml`, no credentials required — `fabro workflow list` does not contact a server.
- `.fabro/workflows/*/workflow.toml` (17 project workflow definitions) already present in the repo; none needed modification to trigger the bug.
- Color output must be enabled through two independent, non-tty gates that both had to be forced in this sandboxed (no real tty) environment:
  - `CLICOLOR_FORCE=1` — forces `console::colors_enabled_stderr()` (gates the pre-styled title text).
  - `TERM=xterm-256color` — required so `termcolor::env_allows_color()` (which `cli_table`'s `ColorChoice::Auto` uses) doesn't treat the terminal as unsupported/`"dumb"`.
  - On a real terminal with a genuine dark-background tty, plain `fabro workflow list` reproduces with no env vars needed — the vars above are a sandbox-only substitute for "has a color-capable tty," not part of the bug's actual trigger condition.

## Reproduction steps
1. From the repo root, build the CLI: `cargo build -p fabro-cli` (produces `target/debug/fabro`; one-time, not needed on subsequent runs).
2. Run: `TERM=xterm-256color CLICOLOR_FORCE=1 ./target/debug/fabro workflow list 2>&1 | cat -v`

No other input, flags, fixture data, or interaction is required.

## Observed result
Every row under "Project Workflows" emits, for the DESCRIPTION cell, the literal escape sequence `\x1b[38;5;8m` — SGR "set foreground to 256-color palette index 8," i.e. exactly `Color::Ansi256(8)` as reported:
```
^[[0m^[[0m ^[[0m^[[0m^[[0m^[[38;5;8m           ^[[0m ^[[0m
```
This is an absolute dark-grey palette color that does not adapt to the terminal's background theme; on a dark background it renders as effectively unreadable, per the report.

Contrast check confirms this is not a global color-suppression artifact: the adjacent NAME cell in the same row renders `^[[36m` (plain ANSI cyan, terminal-relative) — proving color reaches the output stream correctly and that the DESCRIPTION column's `38;5;8` is the specific defect.

## Expected result
The DESCRIPTION cell (and the other nine confirmed call sites) should use a terminal-relative dim/faint attribute (SGR 2, `\x1b[2m`) instead of an absolute palette color, so secondary text renders as a dimmed variant of whatever foreground the user's terminal theme actually uses — legible on both light and dark backgrounds.

Resolved the reporter's open technical question by reading crate source: `cli_table::CellStruct` (via the already-imported `Style` trait) exposes `fn dimmed(self, bool) -> Self`, which sets `termcolor::ColorSpec.dimmed` and emits literal `\x1B[2m` — confirmed in `termcolor` 1.4.1 source. This is a terminal-relative attribute, set through the structured `ColorSpec` API rather than by pre-styling the string before `.cell()`, so there is no ANSI-in-column-width measurement concern. `cli-table` 0.5 does support the needed attribute directly.

## Evidence
- `/tmp/wf_list_raw.txt` — initial pty capture of `fabro workflow list` showing the raw escape sequences.
- `/tmp/repro_confirm.txt` — final confirmation run (2 lines: DESCRIPTION header row and the `card-game` row) showing `\x1b[38;5;8m` around the DESCRIPTION cell, contrasted with `\x1b[36m` on the adjacent NAME cell. Exit code `0` (rendering defect, not a crash).
- `cli-table` 0.5.0 source (`cell.rs`, `style.rs`) and `termcolor` 1.4.1 source (`src/lib.rs` ~line 1525, `Ansi::set_color`) read directly from the local cargo registry cache, confirming `CellStruct::dimmed(bool)` emits SGR 2.
- Static verification (not run live — requires a Fabro server not available in this environment) that `runs/list.rs`'s `fn status_cell` (the most severe case: STATUS conveyed only by `Ansi256(8)` for `Submitted`/`Pending`/`Dead`) uses the identical hardcoded-color pattern; code read and confirmed at lines 178, 184, 185.

## Reproduced?
**Yes.** Reproduces deterministically, 3/3 consecutive runs, byte-identical DESCRIPTION-cell escape sequence each time — not intermittent.

Note: local project workflow files all have empty `goal` fields, so the DESCRIPTION cell text itself is blank in this repro; the hardcoded-color escape sequence still wraps the (empty) cell content identically each run, which is sufficient to confirm the defective call site fires. `fabro ps` (`runs/list.rs`, the most severe STATUS-color case) could not be driven live in this environment (no running Fabro server available) and was verified statically by reading the code instead — its code path (`color_if(use_color, Color::Ansi256(8))` / identical status-color match) is the same mechanism confirmed live in `workflow/list.rs`.
