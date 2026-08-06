Understand the bug and locate the code it lives in. The input is either a GitHub Issue number or a free-text description of the symptom: BUG: CLI table text is unreadable on dark terminal themes.

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
.

READ THE BUG. If you were given an issue number, read it with `gh issue view <n>` and again with `gh issue view <n> --comments`; otherwise use the free-text description you were given. Capture EXPECTED BEHAVIOR versus ACTUAL BEHAVIOR, and the AFFECTED AREA — which screen, command, endpoint or module is involved.

LOCATE THE CODE. Search the codebase by feature name, route, error string, command or UI label, and trace the relevant code until you understand where the behavior actually lives. Do not form a theory of the cause yet — that is a later phase — but do know which files and functions are in play.

OPTIONAL STATUS WRITE, best-effort, warn and continue. If the repo has an active Issue tracker, set the Issue's Project Status to in-progress, but only when it is currently sitting at the tracker's backlog value. Use `facto-helper.sh tracker.exists` to detect the tracker and `facto-helper.sh current-issue` to get the issue number; read `project.owner`, `project.number`, `project.name`, `status_field`, `status_values.backlog`, `status_values.in_progress` and `repo` with `facto-helper.sh tracker.field <name>`; read the current status from `gh issue view <n> --repo <slug> --json projectItems`; and if it equals the backlog name, resolve the project id, status field id, option id and item id with `gh project view`, `gh project field-list` and `gh project item-list`, then apply it with `gh project item-edit`. A failure here is a warning, never an error — print the warning and carry on.

CORE GUARDRAILS that govern this whole run and every phase after it. Fix the ROOT CAUSE, not the symptom — never swallow an error or special-case the one input from the report. Do NOT remove, disable or weaken working functionality to make the symptom go away; no regressions. Add a regression test that fails before the fix and passes after, wherever the project has a test suite. Work AUTONOMOUSLY — escalation in this workflow means heavier tooling for a bigger change, never a request for developer review, and the only genuinely blocking case is being unable to reproduce the bug.

Report the expected-versus-actual summary, the affected area, the files and functions you traced, and the outcome of the optional Status write.