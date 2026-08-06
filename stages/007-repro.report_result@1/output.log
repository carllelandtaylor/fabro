The binary is already built (from prior stages). Let me run `fabro workflow list` and capture the raw ANSI escape sequence as evidence.All 10 occurrences confirmed in the current source. Now I have solid live-captured evidence plus verified source locations. Here's the reproduction report:

---

## Bug
`fabro` CLI tables render secondary/status text using a hardcoded absolute palette color (`Color::Ansi256(8)`, "bright black") instead of a terminal-relative attribute. `Color::Ansi256(8)` maps to a fixed dark grey defined by the terminal's own 256-color palette entry 8, which is calibrated for light backgrounds — on dark-themed terminals it renders nearly the same shade as the background and is effectively invisible. The codebase already has the correct terminal-relative primitive (`fabro_util::terminal::Styles::dim`, which emits SGR 2 "faint" against the terminal's actual foreground color) and uses it correctly elsewhere in the very same file, but the table cells bypass it.

## Environment / preconditions
- Repo `carllelandtaylor/fabro` at commit `0abf229` (branch `fabro/run/01KZCASWWVN35FN33S9YXJ2R2C`), `fabro-cli` built via `cargo build -p fabro-cli --bin fabro` (debug profile, `target/debug/fabro`).
- Requires a real/pty terminal with color enabled (`console::colors_enabled_stderr()` / `colors_enabled()` must be true) — a `TERM` other than `dumb`, no `NO_COLOR` set, stderr/stdout attached to a tty (a plain pipe with no pty, e.g. running under `script` with the default inherited `TERM=dumb`, disables color and hides the bug).
- Any dark-background terminal color scheme (default color 8 rendered close to the background luminance).

## Reproduction steps
1. `cargo build -p fabro-cli --bin fabro`
2. Set up a minimal project so `workflow list` has a description to render (the shipped repo's own `.fabro/workflows/*` entries have no `[run].goal`, so descriptions are blank there — use any project dir with a workflow that sets `[run].goal`, or the sample built here):
   ```
   TMP=$(mktemp -d)
   mkdir -p "$TMP/.fabro/workflows/demo"
   printf '_version = 1\n' > "$TMP/.fabro/project.toml"
   cat > "$TMP/.fabro/workflows/demo/workflow.toml" <<'EOF'
   _version = 1
   [run]
   goal = "Secondary text readability demo"
   [workflow]
   graph = "workflow.fabro"
   EOF
   touch "$TMP/.fabro/workflows/demo/workflow.fabro"
   ```
3. `cd "$TMP" && TERM=xterm-256color script -qec "$OLDPWD/target/debug/fabro workflow list" /tmp/repro_out.txt`
4. `cat -v /tmp/repro_out.txt`

## Observed result
The DESCRIPTION cell is emitted with SGR `38;5;8` (absolute ANSI-256 palette index 8), not SGR `2` (relative "faint"):
```
^[[1mProject Workflows^[[0m ^[[2m(.fabro/workflows)^[[0m
^[[0m^[[0m ^[[0m^[[0m^[[0m^[[36mdemo^[[0m ^[[0m^[[0m ^[[0m^[[0m^[[0m^[[38;5;8mSecondary text readability demo^[[0m ^[[0m
```
Note the contrast within this single output: the section-path text `(.fabro/workflows)` on the line above correctly uses `^[[2m` (dim/faint, terminal-relative — `Styles::dim` from `fabro_util::terminal.rs:28`), while the DESCRIPTION column on the next line uses `^[[38;5;8m` (a fixed absolute grey). The former adapts to whatever the terminal defines as its own foreground; the latter is pinned to palette slot 8, which is dark grey by convention and disappears against a dark background.

Source inspection confirms this is systemic, not a one-off — `color_if(use_color, Color::Ansi256(8))` appears at all 10 locations described in the bug report, current line numbers verified against `0abf229`:
- `lib/apps/fabro-cli/src/commands/runs/list.rs`: lines 128, 138, 148, 178, 184, 185 — including `status_cell()` (128–130 region: `status_cell` at ~172–186), where line 178 sets `Color::Ansi256(8)` as an actual **status** color (not just secondary text) for `Submitted`/`Pending`/`Dead` runs, and line 184 (`.bold(use_color && color != Some(Color::Ansi256(8))))`) explicitly suppresses bold for that same color — so those run statuses are conveyed *only* by a color invisible on dark themes.
- `lib/apps/fabro-cli/src/commands/model.rs`: lines 135, 138 (provider and aliases columns in `model_row`)
- `lib/apps/fabro-cli/src/commands/workflow/list.rs`: line 113 (DESCRIPTION column, reproduced live above)
- `lib/apps/fabro-cli/src/commands/run/checkpoints.rs`: line 93 (detail column)

## Expected result
Secondary/status table text should use a terminal-relative attribute (SGR 2, faint/dim, via `Styles::dim` / `Style::new().dim()`) so it renders against whatever foreground/background the user's terminal theme defines, remaining legible on both light and dark themes — consistent with how `workflow/list.rs`'s own `print_section` header already renders the `(path)` suffix.

## Evidence
- Raw captured output (`cat -v`) from step 4 above, showing `^[[38;5;8m` on the affected cell vs. `^[[2m` on the adjacent correctly-styled text in the same command's output.
- Source excerpts confirming the pattern at all 4 files / 10 call sites (`color_if(use_color, Color::Ansi256(8))` in `runs/list.rs`, `model.rs`, `workflow/list.rs`, `run/checkpoints.rs`), and the correct-usage counterexample `Style::new().dim()` at `lib/foundation/fabro-util/src/terminal.rs:28`, already consumed as `styles.dim` in `workflow/list.rs`'s `print_section`.

## Reproduced?
**Yes**, every time (deterministic — not intermittent) given a color-enabled terminal session. The escape-sequence choice is unconditional in the source (no randomness or timing dependency); it was reproduced live via `fabro workflow list` and is confirmed present at all 10 reported locations by direct source inspection at the current commit.