I'll check the current repository state to confirm the actual build/test status, since the stage log shows a background validation was kicked off but never reported back.Working tree is clean and no real `Ansi256(8)` usage remains (only assertion-message strings). Now let me confirm build/test/clippy status is actually green.Build is clean. Now the test suite and clippy.All 977 tests pass. Now clippy.Clippy is clean. Let me run the full workspace test suite and fmt check, plus re-verify the logged repro, to give a complete picture.Confirmed: `\x1b[2m` (SGR faint) now appears where `\x1b[38;5;8m` used to be, and no `\x1b[38;5;8m` appears anywhere in the output. Full workspace validation to close this out.Full workspace test suite is green: 7796 passed, no failures. Everything checks out — the fix is complete, tested, and validated. Now let me open the PR.Branch is already pushed. Now creating the PR.PR created: **https://github.com/carllelandtaylor/fabro/pull/1**. Now let's check CI status and drive it to green.No CI workflows are configured in this repository (`.github/workflows` has none registered), so there's nothing to drive to green — the PR has no checks to wait on.## Final report

**Bug:** `fabro` CLI tables hardcoded secondary/status cell text to `Color::Ansi256(8)` (absolute 256-color palette index) instead of a terminal-relative dim attribute, making it nearly invisible on dark-background terminals.

### Fix (implemented, tested, committed)
Replaced all 10 sites across four files with `cli_table::CellStruct::dimmed(bool)` (SGR 2 "faint"):
- `runs/list.rs` — 3 mechanical sites in `list_command`, plus `status_cell` restructured to return an explicit `(Option<Color>, is_dim)` pair instead of overloading `Ansi256(8)` as a bold-suppression sentinel
- `workflow/list.rs` — DESCRIPTION cell in `print_section`
- `run/checkpoints.rs` — Details cell in `print_timeline`
- `model.rs` — PROVIDER/ALIASES cells in `model_row`

Two real bugs surfaced and were fixed during implementation, beyond the planned mechanical swap:
- Test helpers (`render_cell` in `runs/list.rs` and `workflow/list.rs`) hardcoded `ColorChoice::Always`, forcing reset codes regardless of the `dimmed()` argument — fixed to branch `Always`/`Never` based on the color state under test.
- A `clippy::absolute_paths` violation in new test code (missing imports for `PendingReason`/`SuccessReason`/etc.) — fixed.

### Validation (all re-run and confirmed green just now)
- `cargo build --workspace` — clean
- `cargo nextest run -p fabro-cli` — 977 passed
- `cargo nextest run --workspace` — 7796 passed, no regressions
- `cargo +nightly-2026-04-14 fmt --check --all` — clean
- `cargo +nightly-2026-04-14 clippy -p fabro-cli --all-targets -- -D warnings` — clean
- `rg -n "Ansi256\(8\)" lib/apps/fabro-cli/src` — no real usages remain, only assertion-message strings
- Re-ran the exact logged repro (`TERM=xterm-256color CLICOLOR_FORCE=1 ./target/debug/fabro workflow list`): DESCRIPTION cell now emits `\x1b[2m`; `\x1b[38;5;8m` no longer appears anywhere in output

### Design review — sanity-check path (no mock exists)
No `design-mock.html` exists anywhere in the repo. This is a terminal ANSI-escape-sequence fix confined to `lib/apps/fabro-cli/src/commands/**`; no `apps/fabro-web` file changed, so there is no rendered UI screen. **No screens were inspected and no issues were found**, because there was nothing to launch or screenshot — the whole `design_review.mock.*` branch (10 consecutive sub-stages) correctly reached this same conclusion each time rather than fabricating evidence. No manual-fallback checklist was produced, since the "app couldn't be launched" condition doesn't apply here (there's no app screen for this change to begin with). Cleanup confirmed nothing was left running: no mock server, no browser process, no listening ports.

### PR and CI
- Opened: **https://github.com/carllelandtaylor/fabro/pull/1**
- CI: this repo is a fork (`carllelandtaylor/fabro` ← `fabro-sh/fabro`) with **zero registered GitHub Actions workflows** on the remote (`gh api .../actions/workflows` returns `total_count: 0`) despite workflow files existing in `.github/workflows/` — Actions has never been enabled for this fork. There are no checks to drive green; `gh pr checks` reports none. All equivalent validation was run locally instead (see above), all green.