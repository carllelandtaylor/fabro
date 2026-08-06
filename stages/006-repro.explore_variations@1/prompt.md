Goal: BUG: CLI table text is unreadable on dark terminal themes.

SYMPTOM
Secondary text in `fabro` CLI tables is hardcoded to `Color::Ansi256(8)` ("bright
black" — a fixed dark grey from the palette). That assumes a light terminal
background; on a dark theme it renders nearly invisible against the background.
Reproduced by the user with `fabro workflow list`, where the DESCRIPTION column
is unreadable.

LOCATIONS — 10 occurrences in 4 files, all under
lib/apps/fabro-cli/src/commands/
  runs/list.rs         6  (lines 128, 138, 148, 178, 184, 185)
  model.rs             2  (lines 135, 138)
  workflow/list.rs     1  (line 113)
  run/checkpoints.rs   1  (line 93)

runs/list.rs is the worst case: line 178 uses Ansi256(8) as a *status* color and
line 184 suppresses bold for it, so a run status is conveyed entirely by a color
the user cannot see. This is not purely cosmetic.

SUSPECTED CAUSE
An absolute palette index is used where a terminal-relative attribute belongs.
The codebase already has the right primitive: `fabro_util::terminal::Styles::dim`
= `Style::new().dim()` at lib/foundation/fabro-util/src/terminal.rs:28. It emits
SGR 2 (faint) against the terminal's own foreground, so it adapts to the theme.
workflow/list.rs already uses `styles.dim` for the section path in
`print_section` — the table cells in the same file just don't.

OPEN QUESTION — resolve this before choosing the fix
Does cli-table 0.5's `CellStruct` expose a faint/dim attribute the way it exposes
`.bold(bool)`? Check the actual crate source in the cargo registry rather than
guessing. Three ways it can go:
  a) If CellStruct has a faint/dim attribute, use it. Cleanest.
  b) Otherwise pre-apply the dim style to the string before `.cell()`. Risk:
     cli-table's column width measurement may count the embedded ANSI escape
     codes and mis-align the table. Verify alignment if taking this path.
  c) As a last resort pick a mid-tone color readable on both light and dark
     backgrounds — but this still absolutizes what should be relative.

Note all styling here already goes through the `use_color` flag via
`color_if()`, so the no-color path is unaffected either way.

CONTRIBUTION PATH
Per CONTRIBUTING.md this qualifies as a "bug fix / small improvement" — send a
PR directly, no issue needed. The `origin` remote is the fork
carllelandtaylor/fabro; `upstream` is fabro-sh/fabro and its push URL is
DISABLED, so push branches to `origin` only. Branch from main and open the PR
against fabro-sh/fabro main, e.g.

  gh pr create --repo fabro-sh/fabro --base main --head carllelandtaylor:<branch>

REQUIRED CHECKS (CONTRIBUTING.md's list is stale relative to CI — use these)
  cargo nextest run --workspace
  cargo +nightly-2026-04-14 fmt --check --all
  cargo +nightly-2026-04-14 clippy --workspace --all-targets -- -D warnings
The pinned nightly matters: stable rustfmt will not match CI.

VERIFICATION
No existing test covers this. Add one asserting the faint attribute (or whatever
the chosen mechanism emits) so this cannot regress. Manually verify the rendered
output against both a light and a dark terminal profile.


## Completed stages
- **understand_locate**: failed
- **repro.understand_report**: failed
- **repro.bring_up_app**: failed
- **repro.attempt_repro**: failed

## Context
- failure_class: deterministic
- failure_signature: repro.attempt_repro|deterministic|acp turn failed


The last attempt did not surface the symptom. Choose a materially different variation to try next, rather than repeating what already failed. Review the attempts logged so far and pick a new axis to vary: a different precondition or starting state, different input values including empty, maximum-length, unicode and boundary values, a different ordering of actions, different seed or fixture data, a different account or permission level, a different environment or feature-flag setting, or a different entry point into the same code path. State the hypothesis this variation tests before running it. Stay within the bounded effort cap: roughly 8 to 10 distinct attempts in total unless the caller gave a different cap. If the cap is now exhausted, end this stage as FAILED so the workflow proceeds to a clean could-not-reproduce report; otherwise end it as succeeded so the next attempt runs.