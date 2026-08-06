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

Run ID: 01KZCAK4457K0NSJW6ZJ7FK37Z
Pipeline progress: 8 of 284 stages completed

## Stage: understand_locate
- Status: failed
- Handler: agent

## Stage: repro.understand_report
- Status: failed
- Handler: agent

## Stage: repro.bring_up_app
- Status: failed
- Handler: agent

## Stage: repro.attempt_repro
- Status: failed
- Handler: agent

## Stage: repro.explore_variations
- Status: failed
- Handler: agent

## Stage: repro.report_result
- Status: failed
- Handler: agent

## Stage: check_repro
- Status: failed
- Handler: agent

## Stage: cnr_report
- Status: failed
- Handler: agent

## Current context
| Key | Value |
|-----|-------|
| failure_class | deterministic |
| failure_signature | cnr_report|deterministic|acp turn failed |


Report what this run actually did, using only what happened in the stages above. Do not invent a PR, a commit, a URL or a verification you did not perform.

If the bug could not be reproduced, report that result and stop there — what was reported, everything that was tried, and why no fix was attempted.

If the fix attempts were exhausted without a clean verification, say so plainly: the attempts made, what each one tried, and the state the branch is left in. Do not present an unverified fix as done.

Otherwise report, in this order: BUG — what was reported. ROOT CAUSE — the one-line statement. PATH TAKEN — direct or escalated, and which checklist item drove it. VERIFICATION — the outcome against the logged reproduction steps, that the bug is gone and nothing regressed, and the regression test that was added, or that the project has no test suite. CI RESULT — green, or stuck with the specific failing checks and why. PR URL.