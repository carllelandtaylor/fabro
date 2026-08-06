I'll verify current code state and check the testing strategy doc before producing the Phase 2 analysis, since this determines test placement for the fix.## Phase 2 — Existing Codebase Analysis

**Guidelines read:** `CLAUDE.md` (root), `docs/internal/testing-strategy.md`, `CONTRIBUTING.md`, `docs/internal/` index (no PRODUCT-REQUIREMENTS.md, TECHNICAL-DESIGN.md, or ADR/decisions directories exist in this repo — this is a bug-fix pipeline, not a feature build, so `.facto/brief.md`'s requirements stand in for acceptance criteria).

### Validation mechanisms (real commands)
- `cargo build -p fabro-cli` — build the affected crate
- `cargo nextest run -p fabro-cli` — run unit + `tests/it` integration tests
- `cargo +nightly-2026-04-14 fmt --check --all` / `--all` to fix
- `cargo +nightly-2026-04-14 clippy --workspace --all-targets -- -D warnings`
- `cargo insta pending-snapshots` / `cargo insta accept` for any snapshot churn

### Existing code confirmed (all four sites re-read directly, matches prior diagnosis exactly)
- `lib/apps/fabro-cli/src/commands/runs/list.rs:128,138,148,178-185` — `status_cell` uses `Color::Ansi256(8)` both as a color *and* as a sentinel (`color != Some(Color::Ansi256(8))`) to suppress bold — the one non-mechanical site.
- `lib/apps/fabro-cli/src/commands/model.rs:110-148` — has its own **private** `color_if` (separate from `lib/apps/fabro-cli/src/shared/utilities.rs:166`), two `Ansi256(8)` sites at 135, 138. Not in scope to unify — out of scope for this fix.
- `lib/apps/fabro-cli/src/commands/workflow/list.rs:113` — one site; `styles.dim` (the *other*, `console`-based dim mechanism) is already used two lines above for a non-table string, confirming the codebase already treats "dim" as the correct semantic for this exact kind of text.
- `lib/apps/fabro-cli/src/commands/run/checkpoints.rs:93` — one site, same pattern.
- `fabro_util::terminal::Styles::dim` (`terminal.rs:28`) is a **different, unrelated mechanism** (`console::Style::dim()`, used for pre-styled plain strings) — not applicable to `cli_table::CellStruct`, which needs `cli_table::Style::dimmed(bool)` instead. Confirmed `Style` (the `cli-table` trait) is already imported in all four files, so no import changes needed.

### Test pattern to follow
`docs/internal/testing-strategy.md` explicitly classifies **"rendering internals"** as unit/crate-level test territory, not `tests/it/cmd/*`. This fits the plan directly: `status_cell`, `model_row`, and the two inline row-builder closures are private/pub(crate) functions that build `CellStruct`/`Vec<CellStruct>` and can be rendered through the real `cli-table` path in a `#[cfg(test)]` mod local to each file (pattern already established in `runs/list.rs:209-242` for `truncate_goal`). Existing `tests/it/cmd/workflow_list.rs` snapshots run with color **disabled** (`TestContext` forces `NO_COLOR=1` by default per `tests/it/support/mod.rs:115`), so it can't currently distinguish `\x1b[2m` from `\x1b[38;5;8m` — confirms the unit-test layer, not the existing cmd-level snapshot, is where this regression must be caught.

### Verification Coverage

| Domain | Expertise | Criterion (from brief.md) | Verification |
|---|---|---|---|
| Rust terminal color/SGR handling (`cli-table` + `termcolor`, `Color::Ansi256` vs `.dimmed()`) | high | Replace `Ansi256(8)` with `.dimmed(use_color)` at all 10 sites; no `Color::Ansi256(8)` remains in `lib/apps/fabro-cli/src` | automated — `rg "Ansi256\(8\)"` returns empty + unit tests assert `\x1b[2m` present |
| Rust terminal color/SGR handling | high | `status_cell` restructured with explicit `is_dim` flag; status→visual mapping unchanged | automated — unit test per `RunStatus` variant asserting color/bold/dim triple |
| Rust terminal color/SGR handling | high | `use_color`/`NO_COLOR` gating preserved (no ANSI when color disabled) | automated — unit test with `use_color=false` asserts no `\x1b[` bytes |
| Rust terminal color/SGR handling | high | Non-dim colors (Cyan, Green, Red, Yellow, Magenta) untouched | automated — existing `workflow_list_shows_project_and_user_sections` snapshot (no-color path) stays green; new unit tests assert Cyan/status colors unchanged |
| Codebase-specific Rust test conventions (`docs/internal/testing-strategy.md`, insta) | high | New regression test fails pre-fix (`\x1b[38;5;8m`), passes post-fix (`\x1b[2m`) | automated — run new test against pre-fix code, confirm red, then confirm green after fix |
| Terminal visual readability on dark-background terminals | low (subjective human perception, not a coding domain) | Text is actually legible on a real dark-background terminal | manual-described — human runs `TERM=xterm-256color CLICOLOR_FORCE=1 ./target/debug/fabro workflow list` in a real terminal and eyeballs it |

### Risk call-out
The one **low-expertise / subjective** row — actual visual legibility on a dark terminal — cannot be automated. SGR 2 "faint" is a terminal-relative attribute, so all we can mechanically prove is that the correct escape byte (`\x1b[2m`) is emitted instead of the absolute-palette one; whether it *looks* right is inherently manual. This is a low-severity risk (SGR 2 is a standard, near-universal terminal capability) but should be flagged as a manual verification step in the plan rather than silently assumed.

No other domain in this fix requires deep/specialty expertise — this is a well-trodden Rust terminal-styling change with the reuse target (`cli_table::Style::dimmed`) and one non-mechanical restructuring (`status_cell`'s sentinel→flag) already fully traced to line numbers.