Goal: Fix a reported bug end-to-end and autonomously: reproduce it, diagnose the real root cause, fix it with adaptive escalation, verify against the logged repro steps, open a pull request, and drive CI to green
Run ID: 01KZCNWCW1GV00S1Q28JJN472Y
Pipeline progress: 15 of 284 stages completed

## Stage: understand_locate
- Status: succeeded
- Handler: agent
- Files touched: .facto/bug-brief.md
- Response:
  > Now writing the brief from the report content only, without touching the rest of the codebase.`.facto/bug-brief.md` is written. Summary:
  > 
  > **Filled in:** Report source, Expected behavior, Actual behavior, Affected area, Reported hypothesis (verbatim reporter claims, including the claimed file/line table, the `cli-table` open question, and the "no existing test" note).
  > 
  > **Left `Unknown`:** Code locations (verifying them is a later stage's job, per instructions — the reporter's claimed table is preserved under Reported hypothesis instead), Preconditions/environment (partially filled, but exact terminal/theme unspecified), Reproduction steps (inferred general steps only, no exact captured transcript in the report), Evidence (none was provided — no screenshots/logs/output), Root cause (diagnosis stage's job).
  > 
  > **Fix constraints:** recorded the reporter's open technical question plus the standing workflow constraints (fix root cause, don't weaken functionality, add a regression test).
  > 
  > **Optional status write:** skipped with a warning — `facto-helper.sh` is not present anywhere on this machine, and the report is free text with no associated issue number, so there's no tracker item to update.

## Stage: repro.understand_report
- Status: succeeded
- Handler: agent
- Files touched: .facto/bug-brief.md
- Response:
  > I'll verify the reporter's claimed code locations by reading the actual files, since the brief currently leaves "Code locations" as Unknown.All ten claimed locations check out exactly. Let me also verify the `cli-table` crate version and API surface to resolve the reporter's open question about dim attribute support.All confirmed. Now updating the bug brief with verified code locations.Verified all ten of the reporter's claimed occurrences and updated `.facto/bug-brief.md` in place.
  > 
  > **Affected area:** `fabro` CLI table rendering in `lib/apps/fabro-cli/src/commands/` — `runs/list.rs`, `model.rs`, `workflow/list.rs`, `run/checkpoints.rs` — plus the shared styling primitive `fabro_util::terminal::Styles` (`lib/foundation/fabro-util/src/terminal.rs`).
  > 
  > **Traced and confirmed exact** (via `rg -n "Ansi256\(8\)" lib/apps/fabro-cli/src`, which returns precisely these ten hits, no more):
  > - `runs/list.rs`: `list_command` (lines 128, 138, 148 — RUN ID/PARENT/GOAL cells) and `status_cell` (lines 178, 184, 185 — status color, bold-suppression, and default color, the most severe case since status becomes unreadable-only)
  > - `model.rs`: `model_row` (lines 135, 138 — PROVIDER/ALIASES cells); also noted this file has its own private `color_if` duplicating the shared one in `shared/utilities.rs:166`
  > - `workflow/list.rs`: `print_section` (line 113 — DESCRIPTION cell); confirmed `styles.dim` is already used two lines earlier in the same function for the section header, just not applied to the table cell
  > - `run/checkpoints.rs`: `print_timeline` (line 93 — Details cell)
  > - `Styles::dim` at `terminal.rs:28`, and confirmed three of the four files already carry a `&Styles` parameter into scope (`runs/list.rs` does too, but only reads `.use_color` from it)
  > 
  > **Still `Unknown` in the brief:** exact reproduction transcript/terminal emulator (reporter never gave one), Evidence (no screenshots/logs provided), Root cause (diagnosis stage's job), and the reporter's open technical question about whether `cli-table` 0.5's `CellStruct` supports a dim attribute — I couldn't inspect the crate source locally (no cargo registry cache present), so that's deferred to diagnosis/fix.

## Stage: repro.bring_up_app
- Status: succeeded
- Handler: agent
- Files touched: .facto/bug-brief.md
- Response:
  > Now let's build the CLI and reproduce the bug with `fabro workflow list`.I'll wait for the background build to finish before driving the CLI.I'll just wait for the background build notification instead of polling.The `cargo build -p fabro-cli` command is running in the background; I'll wait for it to finish before driving the CLI.While the build compiles in the background, let me inspect the relevant source and the `cli-table` crate to resolve the reporter's open technical question.Excellent — `cli-table`'s `Style` trait exposes `.dimmed(bool)`, using `termcolor`'s `ColorSpec::set_dimmed`, which emits SGR 2 (faint) rather than an absolute color. This resolves the reporter's open question cleanly: no foreground color needed, no ANSI-in-column-width concern. Let me confirm `termcolor` actually emits SGR 2 for dimmed.Now let's check the `color_if` helper and `Styles::dim` definitions to plan the fix precisely.Good — `Style` trait is implemented for `CellStruct` and exposes `.dimmed(bool)`. Now let's check current imports in the affected files to know what needs adding.All four files already import `cli_table::Style`, so `.dimmed(bool)` is available with no new imports. Let me check the background build.Good, all the sites are clear now. Let's check on the build.Still compiling many dependencies from a fresh build. I'll wait for the background build task notification rather than continuing to poll manually.The `fabro-cli` build is compiling a large dependency tree; I'll wait for the background task notification rather than polling further.Build is still in progress. I'll stop polling and wait for the automatic notification when it finishes.The build finished successfully. Now let's reproduce the bug by running `fabro workflow list` and inspecting the raw ANSI output.`CLICOLOR_FORCE=1` will force color regardless of tty. Let's reproduce with that.`TERM=dumb` in this environment — that's why cli-table's `ColorChoice::Auto` suppressed the table body colors. Let's force `TERM` too.`fabro ps` is the hidden command that maps to `runs/list.rs`. Let's try it.I have solid reproduction evidence. Let me update the bug brief with these findings before finishing this stage.The bug brief is updated with the reproduction findings. Summary of what I confirmed:
  > 
  > **Driver chosen:** CLI, run directly (this is a terminal color-rendering bug, no web/GUI involved). No project-specific "run" skill exists for launching the app, so I built `fabro-cli` (`cargo build -p fabro-cli`) and ran the resulting binary directly.
  > 
  > **What I confirmed:**
  > - Reproduced the exact mechanism: `fabro workflow list` emits literal `\x1b[38;5;8m` (SGR "set foreground to 256-color palette index 8") around the DESCRIPTION cell — exactly `Color::Ansi256(8)` as reported — contrasted against the adjacent NAME cell's plain `\x1b[36m` cyan, confirming color styling reaches output correctly and this specific escape is the bug, not a global suppression artifact.
  > - Discovered and documented the two independent color-enablement gates in play (`console::colors_enabled_stderr()` for the pre-styled title text, and `cli_table`'s env-based — not tty-based — `ColorChoice::Auto`), which was needed to get color output at all in this sandboxed/no-real-tty environment.
  > - Resolved the reporter's open technical question by reading the `cli-table` 0.5.0 and `termcolor` 1.4.1 crate sources: `CellStruct` already exposes `.dimmed(bool)` (all four affected files already import the `Style` trait), which emits SGR 2 (`\x1b[2m`, terminal-relative "faint") via structured `ColorSpec` — no ANSI-in-cell-width concern, since nothing needs to be pre-styled into the string before `.cell()`.
  > - `fabro ps` (the `runs/list.rs` command) requires a running server not available here, so it was verified statically only, not driven live.
  > 
  > This gives the fix stage a clear, low-risk path: replace each `.foreground_color(color_if(use_color, Color::Ansi256(8)))` with `.dimmed(use_color)`.

## Stage: repro.attempt_repro
- Status: succeeded
- Handler: agent
- Response:
  > The `repro.bring_up_app` stage is already complete — it's committed as `6d0c237` and the working tree is clean with no uncommitted changes.
  > 
  > Checking `.facto/bug-brief.md`, the reproduction was already performed and recorded in full:
  > 
  > - Built `fabro-cli` and ran `TERM=xterm-256color CLICOLOR_FORCE=1 ./target/debug/fabro workflow list` under a pty
  > - Observed the exact symptom: the DESCRIPTION cell emits literal `\x1b[38;5;8m` (absolute 256-color palette index 8), contrasted against the adjacent NAME cell's `\x1b[36m` — confirming the hardcoded `Color::Ansi256(8)` reaches the output stream exactly as the reporter described, and that this isn't a global color-suppression artifact
  > - Resolved the reporter's open technical question by reading the `cli-table` 0.5.0 and `termcolor` 1.4.1 crate sources: `CellStruct` already exposes `.dimmed(bool)` (via the `Style` trait, already imported in all four affected files), which emits SGR 2 "faint" — a terminal-relative attribute — with no ANSI-in-column-width concern
  > - Noted `fabro ps` (the `runs/list.rs` STATUS-color case) couldn't be driven live (needs a running server), so it was verified statically only
  > 
  > The brief's Root cause section is correctly left `Unknown` for the diagnosis stage, and all other established facts (preconditions, reproduction steps, evidence, code locations) are filled in. Nothing further is needed for this stage — it's already succeeded and persisted.

## Stage: repro.confirm_distill
- Status: succeeded
- Handler: agent
- Response:
  > I'll read the current bug brief to see what's already documented before refining it into a minimal repeatable repro sequence.Reproduced reliably a third time. Now I'll update the bug brief with the minimized, precisely-formatted repro sequence and confirmation evidence.Both goals are met: the bug reliably reproduces (confirmed 3/3 identical runs) and the minimal repeatable sequence is logged in `.facto/bug-brief.md` — one build command plus a single `fabro workflow list` invocation, with explicit preconditions, ordered actions, observed-vs-expected output, and captured evidence (`/tmp/wf_list_raw.txt`, `/tmp/repro_confirm.txt`).The change to `.facto/bug-brief.md` is staged. Consistent with the pattern in the git log (each stage's commit is created automatically by the pipeline, named after the stage), I've left it staged rather than committing manually — the pipeline will commit as `repro.attempt_repro (succeeded)`.

## Stage: repro.report_result
- Status: succeeded
- Handler: agent
- Files touched: .facto/result.md
- Response:
  > No template file found locally — I'll build the report directly from the instructions' section list. Now writing `.facto/result.md`.The bug brief was already updated and committed in the prior `repro.confirm_distill` stage — it's fully current with the minimized repro, evidence, and reliability notes, so no further edits are needed there. `.facto/result.md` is now written with the reproduction report.
  > 
  > ## Reproduction report
  > 
  > **Bug:** `fabro` CLI tables hardcode secondary/status text to `Color::Ansi256(8)`, an absolute dark-grey palette color, instead of a terminal-relative dim attribute — nearly invisible on dark-background terminals. 10 confirmed sites across `runs/list.rs`, `model.rs`, `workflow/list.rs`, `run/checkpoints.rs`.
  > 
  > **Reproduced?** Yes, 3/3 deterministic runs.
  > 1. `cargo build -p fabro-cli`
  > 2. `TERM=xterm-256color CLICOLOR_FORCE=1 ./target/debug/fabro workflow list 2>&1 | cat -v`
  > 
  > Observed literal `\x1b[38;5;8m` around the DESCRIPTION cell (vs. plain `\x1b[36m` on the adjacent NAME cell, proving color output works and this specific site is the defect). Also resolved the reporter's open question: `cli_table::CellStruct::dimmed(bool)` exists and emits proper SGR 2 faint, so the fix path is clear — swap `.foreground_color(color_if(use_color, Color::Ansi256(8)))` for `.dimmed(use_color)` at all ten sites.

## Stage: check_repro
- Status: succeeded
- Handler: agent
- Response:
  > Both files are consistent: the reproduction report confirms the bug was reproduced deterministically (3/3 runs, byte-identical output) — `fabro workflow list` emits `Color::Ansi256(8)` (`\x1b[38;5;8m`) for secondary/dim text instead of a terminal-relative dim attribute, contrasted against the adjacent NAME cell's normal `\x1b[36m` to rule out a global color-suppression artifact.
  > 
  > The bug-brief's `Preconditions, inputs and environment`, `Reproduction steps (minimized...)`, and `Evidence` sections already contain this exact confirmed repro verbatim — preconditions (repo root, no server needed, `TERM=xterm-256color CLICOLOR_FORCE=1` to force color output in a tty-less sandbox), the two ordered actions (`cargo build -p fabro-cli`, then `./target/debug/fabro workflow list`), the observed `\x1b[38;5;8m` escape vs. expected `\x1b[2m` (SGR faint), and the evidence files (`/tmp/wf_list_raw.txt`, `/tmp/repro_confirm.txt`) plus the `cli-table`/`termcolor` source confirmation that `CellStruct::dimmed(bool)` is the correct fix mechanism. Nothing in these sections is a summary or invention — it matches the report's content and the report explicitly states "Reproduced? Yes."
  > 
  > This stage ends **SUCCEEDED**: the report reproduced the bug and observed the wrong behavior (hardcoded absolute-palette color instead of terminal-relative dim), and the brief already carries the confirmed repro verbatim for later phases to re-run exactly.

## Stage: diagnose
- Status: succeeded
- Handler: agent
- Files touched: .facto/bug-brief.md
- Response:
  > Confirmed: `CellStruct::dimmed(bool)` exists and applies `set_dimmed` to the `ColorSpec`, emitting SGR 2. This confirms the fix path. Now writing the root cause into the bug brief.Good, this confirms my proposed fix approach is accurate. The root cause and fix approach are now written into the bug brief.
  > 
  > ## Summary
  > 
  > **Root cause:** Ten call sites across `runs/list.rs`, `model.rs`, `workflow/list.rs`, and `run/checkpoints.rs` style secondary/status table cells with `.foreground_color(color_if(use_color, Color::Ansi256(8)))` — an absolute palette color — instead of `cli_table::CellStruct::dimmed(bool)` (confirmed to exist in the `cli-table` 0.5.0 source and to emit terminal-relative SGR 2 "faint"), which is the mechanism already used elsewhere in these same files via `Styles::dim`. The absolute color can't adapt to a dark terminal background, making it nearly invisible; in `runs/list.rs::status_cell`, `Ansi256(8)` is additionally overloaded as a sentinel value for bold-suppression logic.
  > 
  > **Proposed fix:** Replace all ten sites with `.dimmed(use_color)`, and in `status_cell` (lines 171–186) introduce an explicit `is_dim` flag instead of comparing against the `Ansi256(8)` sentinel, so no fixed-color logic remains. A new test forcing `use_color = true` should assert the rendered output contains `\x1b[2m` and not `\x1b[38;5;8m` — failing before the fix, passing after.
  > 
  > Both are now recorded in `.facto/bug-brief.md` under **Root cause**, ready for the fix stage.

## Stage: assess_fix_size
- Status: succeeded
- Handler: agent
- Response:
  > **Decision: ESCALATE**
  > 
  > Trigger: the checklist item "touches more than about two files" — the fix requires changes across four files (`lib/apps/fabro-cli/src/commands/runs/list.rs`, `model.rs`, `workflow/list.rs`, `run/checkpoints.rs`), plus a new regression test asserting on emitted ANSI/SGR bytes. It also involves a small logic restructuring in `runs/list.rs::status_cell` (replacing the `Ansi256(8)` sentinel comparison with an explicit `is_dim` flag), not a pure search-and-replace. No new abstraction/interface, schema/API/migration change is involved — only the file-count criterion triggered escalation.
  > 
  > Ending this stage as **SUCCEEDED** (escalated path).

## Stage: escalated_gate
- Status: succeeded
- Handler: conditional
- Notes: Conditional node evaluated: escalated_gate

## Stage: plan_brief
- Status: succeeded
- Handler: agent
- Files touched: .facto/brief.md
- Response:
  > Now I have all the detail needed. Let me write the brief for the planning stage.The brief is written to `.facto/brief.md`. Here it is:
  > 
  > # Fix planning brief
  > 
  > This brief is the sole input to the planning stage. Plan a FIX for the root cause below — not a feature, not a refactor beyond what the fix requires.
  > 
  > ## 1. Root cause
  > 
  > **One-line statement:** Ten call sites across four `fabro-cli` command files style secondary/status table cells with `.foreground_color(color_if(use_color, Color::Ansi256(8)))` — an absolute, non-adaptive 256-color palette index — instead of `cli_table::CellStruct::dimmed(bool)`, the terminal-relative dim/faint (SGR 2) mechanism the crate already provides for exactly this purpose; on dark-background terminals `Ansi256(8)` renders as near-invisible text.
  > 
  > **Supporting trace:**
  > - `cli_table` 0.5.0's `CellStruct` (via the `Style` trait, already imported in all four affected files) exposes `fn dimmed(self, bool) -> Self`, confirmed at `~/.cargo/registry/.../cli-table-0.5.0/src/style.rs:18,79-82`. It sets `ColorSpec::set_dimmed`, which `termcolor` 1.4.1's `Ansi::set_color` (confirmed `src/lib.rs` ~line 1525) turns into literal `\x1b[2m` (SGR 2, "faint" — relative to the terminal's own foreground), not an absolute palette color.
  > - The project already has an equivalent terminal-relative primitive in scope in three of the four files: `fabro_util::terminal::Styles::dim` at `lib/foundation/fabro-util/src/terminal.rs:28`. `workflow/list.rs::print_section` already calls `styles.dim.apply_to(...)` two lines above the buggy table cell in the same function — the correct primitive is already in scope there, just not applied to the cell.
  > - Reproduction confirmed color reaches the output stream correctly (the adjacent NAME cell renders plain `\x1b[36m` cyan in the same row), isolating the defect specifically to the `Ansi256(8)` call sites.
  > 
  > **Files and functions involved (all ten confirmed occurrences):**
  > - `lib/apps/fabro-cli/src/commands/runs/list.rs` — `fn list_command`: lines 128, 138, 148; `fn status_cell` (lines 171–186): lines 178, 184 (sentinel comparison), 185 — most severe, status conveyed only by unreadable color.
  > - `lib/apps/fabro-cli/src/commands/model.rs` — `fn model_row`: lines 135, 138.
  > - `lib/apps/fabro-cli/src/commands/workflow/list.rs` — `fn print_section`: line 113.
  > - `lib/apps/fabro-cli/src/commands/run/checkpoints.rs` — `fn print_timeline`: line 93.
  > 
  > Shared primitive available for reuse: `fabro_util::terminal::Styles::dim` at `lib/foundation/fabro-util/src/terminal.rs:28`.
  > 
  > **Proposed mechanism:** Replace `.foreground_color(color_if(use_color, Color::Ansi256(8)))` with `.dimmed(use_color)` at all ten sites. `runs/list.rs::status_cell` requires restructuring (introduce an explicit `is_dim` flag) since `Ansi256(8)` also doubles as a bold-suppression sentinel there — not a pure search-and-replace. After the fix, no `Color::Ansi256(8)` should remain anywhere in `lib/apps/fabro-cli/src`.
  > 
  > ## 2. Logged reproduction steps (must be re-runnable exactly as recorded)
  > 
  > **Preconditions:** Repo at current branch tip, clean tree, cwd = repo root, no server/settings/credentials needed. `TERM=xterm-256color CLICOLOR_FORCE=1` must be set (sandbox has no real tty; a real dark-background terminal reproduces with no env vars).
  > 
  > **Ordered actions:**
  > 1. `cargo build -p fabro-cli`
  > 2. `TERM=xterm-256color CLICOLOR_FORCE=1 ./target/debug/fabro workflow list 2>&1 | cat -v`
  > 
  > **Observed (wrong):** DESCRIPTION cell emits `\x1b[38;5;8m`; adjacent NAME cell emits normal `\x1b[36m`, proving the defect is isolated to the Ansi256(8) sites.
  > 
  > **Expected:** DESCRIPTION cell (and the other nine sites) emit `\x1b[2m` instead.
  > 
  > **Reliability:** 3/3 deterministic, byte-identical. `fabro ps` (`runs/list.rs`) has an identical code path but requires a server unavailable here — verified statically, must still be covered by fix + test.
  > 
  > The plan's validation must re-run these two commands and confirm `\x1b[38;5;8m` → `\x1b[2m`.
  > 
  > ## 3. No-regression constraint (hard, applies to every step)
  > 
  > - `use_color`/`color_if` gating for `NO_COLOR`/non-tty must be preserved — `.dimmed(use_color)` must mirror the existing gating, not bypass it.
  > - All non-`Ansi256(8)` colors (Cyan, per-status colors) untouched.
  > - `status_cell`'s status-distinguishing behavior preserved — only the dim/gray mechanism changes.
  > - No step may delete a color/status case or narrow existing behavior to make the bug disappear.
  > 
  > ## 4. Regression test requirement
  > 
  > Add a `cargo nextest` test that forces `use_color = true`, renders the affected cell(s) through the real `cli-table` path, and asserts output contains `\x1b[2m` and not `\x1b[38;5;8m`. Must fail on unfixed code, pass after the fix — demonstrate this as part of validation. Cover both the plain-dim case (e.g. `workflow/list.rs`) and the restructured `status_cell` case.
  > 
  > ## 5. Escalation reason
  > 
  > Escalated on **"touches more than about two files"** — four files plus new test coverage, plus a small logic restructuring in `status_cell` (not pure mechanical replace). No new abstractions/API/migration involved — scope the plan accordingly.

## Stage: plan_fix.issue_context
- Status: succeeded
- Handler: command
- Script: `set +e; if facto-helper.sh tracker.exists 2>/dev/null; then ISSUE_NUMBER="$(facto-helper.sh current-issue 2>/dev/null)"; if [ -n "$ISSUE_NUMBER" ]; then REPO_SLUG="$(facto-helper.sh tracker.field repo)"; gh issue view "$ISSUE_NUMBER" --repo "$REPO_SLUG" --json title,body,comments; else echo 'no active issue resolved'; fi; else echo 'no facto tracker configured'; fi; exit 0`
- Output:
  ```
  no facto tracker configured
  ```

## Stage: plan_fix.absorb
- Status: succeeded
- Handler: agent
- Response:
  > No additional planning docs exist — the brief at `.facto/brief.md` is the sole input, as it states. No Issue context was found (no tracker configured), so nothing to add from that source.
  > 
  > Here's what needs to be built:
  > 
  > - **Root cause**: 10 call sites across 4 `fabro-cli` files hardcode secondary/status table cell styling to `Color::Ansi256(8)` (an absolute 256-color palette index) instead of using `cli_table::CellStruct::dimmed(bool)`, the terminal-relative SGR-2 "faint" mechanism the crate already supports — causing near-invisible text on dark-background terminals.
  > - **Files to fix**: `runs/list.rs` (4 sites: `list_command` ×3, `status_cell` ×3 counting the sentinel), `model.rs` (2 sites), `workflow/list.rs` (1 site), `run/checkpoints.rs` (1 site).
  > - **Mechanical part**: replace `.foreground_color(color_if(use_color, Color::Ansi256(8)))` → `.dimmed(use_color)` at 9 of the 10 sites.
  > - **Non-mechanical part**: `runs/list.rs::status_cell` uses `Ansi256(8)` both as the color for `Submitted|Pending|Dead` statuses *and* as a sentinel to suppress bold (`color != Some(Color::Ansi256(8))`) — this needs restructuring to an explicit `is_dim` flag rather than a straight swap.
  > - **Hard constraint**: preserve existing `use_color`/`color_if` gating (NO_COLOR/non-tty behavior), leave all non-dim colors (Cyan, per-status green/red/yellow) untouched, and preserve which status maps to which visual treatment — no deleting/narrowing behavior to make the bug disappear.
  > - **New test required**: assert real rendered output contains `\x1b[2m` and not `\x1b[38;5;8m`, covering both a plain-dim site (e.g. `workflow/list.rs`) and the restructured `status_cell` case. Must be shown failing pre-fix, passing post-fix.
  > - **Validation**: re-run the exact logged repro (`cargo build -p fabro-cli` then `TERM=xterm-256color CLICOLOR_FORCE=1 ./target/debug/fabro workflow list 2>&1 | cat -v`) and confirm `\x1b[38;5;8m` is gone, replaced by `\x1b[2m`.
  > - **Scope**: no new abstractions, no API/schema changes — a coordinated multi-file mechanical fix plus one small logic restructuring plus a new test.
  > 
  > The requirements are clear, internally consistent, and fully traced (exact line numbers, confirmed crate API, confirmed repro output). Nothing to flag — moving on to the next planning phase.

## Stage: plan_fix.clarify_gate
- Status: succeeded
- Handler: human

## Stage: plan_fix.analyze_codebase
- Status: succeeded
- Handler: agent
- Response:
  > I'll verify current code state and check the testing strategy doc before producing the Phase 2 analysis, since this determines test placement for the fix.## Phase 2 — Existing Codebase Analysis
  > 
  > **Guidelines read:** `CLAUDE.md` (root), `docs/internal/testing-strategy.md`, `CONTRIBUTING.md`, `docs/internal/` index (no PRODUCT-REQUIREMENTS.md, TECHNICAL-DESIGN.md, or ADR/decisions directories exist in this repo — this is a bug-fix pipeline, not a feature build, so `.facto/brief.md`'s requirements stand in for acceptance criteria).
  > 
  > ### Validation mechanisms (real commands)
  > - `cargo build -p fabro-cli` — build the affected crate
  > - `cargo nextest run -p fabro-cli` — run unit + `tests/it` integration tests
  > - `cargo +nightly-2026-04-14 fmt --check --all` / `--all` to fix
  > - `cargo +nightly-2026-04-14 clippy --workspace --all-targets -- -D warnings`
  > - `cargo insta pending-snapshots` / `cargo insta accept` for any snapshot churn
  > 
  > ### Existing code confirmed (all four sites re-read directly, matches prior diagnosis exactly)
  > - `lib/apps/fabro-cli/src/commands/runs/list.rs:128,138,148,178-185` — `status_cell` uses `Color::Ansi256(8)` both as a color *and* as a sentinel (`color != Some(Color::Ansi256(8))`) to suppress bold — the one non-mechanical site.
  > - `lib/apps/fabro-cli/src/commands/model.rs:110-148` — has its own **private** `color_if` (separate from `lib/apps/fabro-cli/src/shared/utilities.rs:166`), two `Ansi256(8)` sites at 135, 138. Not in scope to unify — out of scope for this fix.
  > - `lib/apps/fabro-cli/src/commands/workflow/list.rs:113` — one site; `styles.dim` (the *other*, `console`-based dim mechanism) is already used two lines above for a non-table string, confirming the codebase already treats "dim" as the correct semantic for this exact kind of text.
  > - `lib/apps/fabro-cli/src/commands/run/checkpoints.rs:93` — one site, same pattern.
  > - `fabro_util::terminal::Styles::dim` (`terminal.rs:28`) is a **different, unrelated mechanism** (`console::Style::dim()`, used for pre-styled plain strings) — not applicable to `cli_table::CellStruct`, which needs `cli_table::Style::dimmed(bool)` instead. Confirmed `Style` (the `cli-table` trait) is already imported in all four files, so no import changes needed.
  > 
  > ### Test pattern to follow
  > `docs/internal/testing-strategy.md` explicitly classifies **"rendering internals"** as unit/crate-level test territory, not `tests/it/cmd/*`. This fits the plan directly: `status_cell`, `model_row`, and the two inline row-builder closures are private/pub(crate) functions that build `CellStruct`/`Vec<CellStruct>` and can be rendered through the real `cli-table` path in a `#[cfg(test)]` mod local to each file (pattern already established in `runs/list.rs:209-242` for `truncate_goal`). Existing `tests/it/cmd/workflow_list.rs` snapshots run with color **disabled** (`TestContext` forces `NO_COLOR=1` by default per `tests/it/support/mod.rs:115`), so it can't currently distinguish `\x1b[2m` from `\x1b[38;5;8m` — confirms the unit-test layer, not the existing cmd-level snapshot, is where this regression must be caught.
  > 
  > ### Verification Coverage
  > 
  > | Domain | Expertise | Criterion (from brief.md) | Verification |
  > |---|---|---|---|
  > | Rust terminal color/SGR handling (`cli-table` + `termcolor`, `Color::Ansi256` vs `.dimmed()`) | high | Replace `Ansi256(8)` with `.dimmed(use_color)` at all 10 sites; no `Color::Ansi256(8)` remains in `lib/apps/fabro-cli/src` | automated — `rg "Ansi256\(8\)"` returns empty + unit tests assert `\x1b[2m` present |
  > | Rust terminal color/SGR handling | high | `status_cell` restructured with explicit `is_dim` flag; status→visual mapping unchanged | automated — unit test per `RunStatus` variant asserting color/bold/dim triple |
  > | Rust terminal color/SGR handling | high | `use_color`/`NO_COLOR` gating preserved (no ANSI when color disabled) | automated — unit test with `use_color=false` asserts no `\x1b[` bytes |
  > | Rust terminal color/SGR handling | high | Non-dim colors (Cyan, Green, Red, Yellow, Magenta) untouched | automated — existing `workflow_list_shows_project_and_user_sections` snapshot (no-color path) stays green; new unit tests assert Cyan/status colors unchanged |
  > | Codebase-specific Rust test conventions (`docs/internal/testing-strategy.md`, insta) | high | New regression test fails pre-fix (`\x1b[38;5;8m`), passes post-fix (`\x1b[2m`) | automated — run new test against pre-fix code, confirm red, then confirm green after fix |
  > | Terminal visual readability on dark-background terminals | low (subjective human perception, not a coding domain) | Text is actually legible on a real dark-background terminal | manual-described — human runs `TERM=xterm-256color CLICOLOR_FORCE=1 ./target/debug/fabro workflow list` in a real terminal and eyeballs it |
  > 
  > ### Risk call-out
  > The one **low-expertise / subjective** row — actual visual legibility on a dark terminal — cannot be automated. SGR 2 "faint" is a terminal-relative attribute, so all we can mechanically prove is that the correct escape byte (`\x1b[2m`) is emitted instead of the absolute-palette one; whether it *looks* right is inherently manual. This is a low-severity risk (SGR 2 is a standard, near-universal terminal capability) but should be flagged as a manual verification step in the plan rather than silently assumed.
  > 
  > No other domain in this fix requires deep/specialty expertise — this is a well-trodden Rust terminal-styling change with the reuse target (`cli_table::Style::dimmed`) and one non-mechanical restructuring (`status_cell`'s sentinel→flag) already fully traced to line numbers.

## Current context
| Key | Value |
|-----|-------|
| human.gate.label | [P] Proceed, that's right |
| human.gate.plan_fix.clarify_gate.answer | P |
| human.gate.plan_fix.clarify_gate.label | [P] Proceed, that's right |
| human.gate.plan_fix.clarify_gate.question | Is that a correct read of what needs to be built? Pick 'proceed' if it's right or close enough — I'll only stop if something is genuinely unclear. |
| human.gate.selected | P |


Phase 3 — Key Decisions. Identify the technical decisions that need making before you can draft the plan. A decision belongs here only if there are multiple reasonable approaches, the choice meaningfully affects the implementation, and the developer should have input. Skip anything that is obvious from the codebase patterns or the requirements.

For each decision: state the decision, present 2-3 options with brief pros and cons tied to the requirements, and make a recommendation — what you would pick and why.

Present all the decisions in a single message as a numbered list, and invite the developer to reply once covering everything. Do not use the AskUserQuestion tool; use natural conversation. Exception: if a decision genuinely depends on the answer to an earlier one, hold it back and ask it as a follow-up after that answer lands.

Common decisions to surface when relevant: data model changes (new tables or fields, schema design), API design (endpoints, request and response shapes), component architecture (new components versus extending existing ones), state management approach, meaningful third-party library choices, and migration strategy if existing behaviour is changing.

Reference actual files, functions and patterns from the codebase — not hypothetical ones. Respect stated constraints such as 'keep it simple' or 'no new dependencies'. Do not over-engineer: a three-step feature does not need a twelve-decision interview.

If you are back here after a first round, ask only the follow-up decisions that depended on the answers already given.