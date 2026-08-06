These are empty checkpoint commits — no diffs recorded in git. I need to actually reproduce the bug now and document it. Let me look at the relevant source files first.Good, there's a project config already. Let's check whether a fabro binary is already built, then run the exact command to reproduce.Binary already built from an earlier stage. Let's run the reproduction command and capture raw output with escape codes visible.I'll add a `[run] goal = "..."` to one workflow.toml temporarily to get a non-empty DESCRIPTION cell for reproduction, then revert it.Reproduction confirmed reliably on a second run. Now reverting the temporary fixture edit used for reproduction.Working tree is clean again. Reproduction is confirmed and reliable.

## Repro report: unreadable DESCRIPTION column in `fabro workflow list` on dark themes

**PRECONDITIONS AND DATA**
- Repo checked out at current `main`-derived branch; `fabro-cli` built (`cargo build -p fabro-cli`, or `cargo build --workspace`) producing `target/debug/fabro`.
- A `.fabro/project.toml` project root (present at repo root here).
- At least one workflow with a non-empty `[run] goal` in its `workflow.toml`, e.g. `.fabro/workflows/hello/workflow.toml`:
  ```toml
  [run]
  goal = "Say hello to the user as a repro description string"
  ```
- Color output enabled: a real terminal with `TERM` ≠ `dumb` and a dark background theme (or, for headless verification, `TERM=xterm-256color script -qec "..." out.txt` to force a pty, since `console::colors_enabled_stderr()` requires stderr to be a real tty with a non-dumb `TERM`).

**ORDERED ACTIONS**
1. `cd` to the repo root.
2. Add the `[run] goal = "..."` line to `.fabro/workflows/hello/workflow.toml` as above.
3. Run `fabro workflow list` in a terminal with a dark color scheme (or headlessly: `TERM=xterm-256color script -qec "./target/debug/fabro workflow list" /tmp/out.txt` then `cat -v /tmp/out.txt`).
4. Look at the DESCRIPTION column for the `hello` row.

**OBSERVED (WRONG) RESULT**
The DESCRIPTION cell is emitted as:
```
^[[38;5;8mSay hello to the user as a repro description string^[[0m
```
i.e. `ESC[38;5;8m` — SGR "set foreground to absolute ANSI256 palette index 8" (a fixed dark grey), sourced from `lib/apps/fabro-cli/src/commands/workflow/list.rs:113`. On a dark-background theme this renders nearly invisible. By contrast, other secondary text in the very same command's output (`(~/.fabro/workflows)` at line 85) uses `ESC[2m` (SGR faint, terminal-relative `styles.dim`) and stays readable on both light and dark themes — confirming the DESCRIPTION column is the outlier.

**EXPECTED RESULT**
DESCRIPTION text should use the terminal-relative faint/dim attribute (`fabro_util::terminal::Styles::dim`, SGR 2), consistent with the rest of the file and codebase, so it adapts to the user's theme instead of an absolute palette color.

**EVIDENCE**
Raw `cat -v` capture of the `hello` row under a forced-color pty:
```
^[[0m^[[0m ^[[0m^[[0m^[[0m^[[36mhello          ^[[0m ^[[0m^[[0m ^[[0m^[[0m^[[0m^[[38;5;8mSay hello to the user as a repro description string^[[0m ^[[0m
```

**RELIABILITY**
Reproduced twice in a row (two independent `script` captures), fully deterministic — every non-empty DESCRIPTION cell gets `Ansi256(8)` on every invocation when color is enabled, not intermittent.

The working tree was restored to clean (`git checkout -- .fabro/workflows/hello/workflow.toml`) after capturing evidence — no fixture changes are left behind. Next step is the actual fix per the goal: resolve the open question (does `cli-table`'s `CellStruct` expose a dim/faint attribute) and apply it consistently across the 10 occurrences in the 4 files.