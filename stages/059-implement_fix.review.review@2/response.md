## Review results — cycle 2 of 5

Subagent (opus) read `git diff 0abf2297c00a90013a93ae01c6c139b7b85b1b1f..HEAD` and every changed file in full, verified cycle-1's fixes via mutation testing (reverting lines to confirm tests actually catch regressions), then reviewed the diff fresh.

**Bottom line:** The production fix remains correct, complete, and convention-compliant — 977/977 tests pass, fmt/clippy clean, `rg Ansi256` matches only assertion strings. Cycle 1's fix for "important finding #2" (`ColorChoice::Never` masking broken `use_color` gating) was correctly applied to `workflow/list.rs`, `checkpoints.rs`, and `model.rs`, but **not carried through to `runs/list.rs::status_cell`** — the one function that was actually restructured (the `(Option<Color>, is_dim)` tuple). Mutation-verified: both `.bold(use_color && !is_dim)`'s `use_color` gate and the `color_if` gating on the `Some(c)` branch can be deleted without any test failing.

### IN-SCOPE

1. **[important]** `runs/list.rs:173-180,310-317` — `status_cell`'s `use_color=false` test only exercises `RunStatus::Dead` (the `is_dim=true`, `color=None` path), so neither the bold-suppression gate nor the color-gating branch is tested with `use_color=false`. Fix: loop the no-color test over a non-dim status too (e.g. `Succeeded`), asserting no style escapes for either.
2. **[minor]** `workflow/list.rs`, `checkpoints.rs`, `model.rs` — no test pins that non-dim colors (Cyan) survived the refactor; mutation confirms deleting the Cyan `.foreground_color()` calls leaves all tests green.
3. **[minor]** `checkpoints.rs:136-144`, `workflow/list.rs:163-171` — row-level `contains("\x1b[2m")` assertions can't tell which cell is dim (unlike `model.rs`, which was fixed to check per-cell); moving `.dimmed()` to the wrong cell still passes.
4. **[minor]** `test_support.rs:5-21` — `render_row`/`render_cell`'s `use_color` bool parameter is dead (always `true`) and reads contradictorily at call sites like `render_row(model_row(&model, false), true)`; should hardcode `ColorChoice::Always`.
5. **[minor]** `test_support.rs:1` — redundant inner `#![cfg(test)]`, inconsistent with every other `test_support.rs` in the workspace.
6. **[minor]** `test_support.rs:27-29` — `has_style_escape` misses background-color/underline SGRs; a leaked background color under `use_color=false` would pass silently.
7. **[minor]** `model.rs:657-672` — `model_row(...)` built twice via magic `.nth()` indices with no link to column names.
8. **[minor]** `workflow/list.rs:123` vs `checkpoints.rs:55` — inconsistent helper-before/after-caller ordering between the two otherwise-identical extractions.
9. **[minor]** `docs/plans/2026-08-06-fix-text-color-plan.md` — stale `status: active`, dangling references to deleted `.facto/` files, and a factually wrong claim that status is conveyed "only by color."

### OUT-OF-SCOPE (report-only)

- **A. [important, if in scope]** `workflow/list.rs::truncate_str` byte-slices UTF-8 (panic risk on multi-byte chars) — pre-existing; `runs/list.rs::truncate_str` already does this correctly with `.chars()`.
- **B. [minor]** `checkpoints.rs::short_id` byte-slices an assumed-ASCII run id.
- **C. [minor]** Inconsistent error handling across `print_section`/`print_timeline`/`print_models_table` on render failure (panic vs. silent swallow).
- **D. [minor]** `model.rs` still has its private duplicate `color_if` (now only one call site).
- **E. [minor]** `format_context_window` rounds wrong for negative `tokens` (unreachable in practice).

One important-severity in-scope finding remains, so another cycle is warranted after fixing it — recommend bundling minor #2 and #3 into the same pass since they're cheap and adjacent.