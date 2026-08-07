## Review classification — cycle 2

**Outcome: 2 — IN-SCOPE findings include an important-severity item. Another full cycle follows the fixes.**

**Cycle mode: CONTINUING CYCLE** (cycle 2 of 5 max; well under the cap, so this is a normal continuation, not a capped final cycle.)

**In-scope finding count by severity:**
- Critical: 0
- Important: 1
  1. `runs/list.rs:173-180,310-317` — `status_cell`'s `use_color=false` test only exercises `RunStatus::Dead` (the `is_dim=true`, `color=None` path); neither the bold-suppression gate nor the color-gating branch is exercised with `use_color=false`. Mutation-verified: both gates can be deleted without any test failing.
- Minor: 8
  2. No test pins that non-dim colors (Cyan) survived the refactor in `workflow/list.rs`, `checkpoints.rs`, `model.rs`.
  3. Row-level `contains("\x1b[2m")` assertions in `checkpoints.rs`/`workflow/list.rs` can't tell which cell is dim.
  4. `test_support.rs` `render_row`/`render_cell`'s `use_color` bool parameter is dead (always `true`).
  5. `test_support.rs:1` redundant inner `#![cfg(test)]`.
  6. `has_style_escape` misses background-color/underline SGRs.
  7. `model.rs` `model_row(...)` built twice via magic `.nth()` indices.
  8. Inconsistent helper-before/after-caller ordering between `workflow/list.rs` and `checkpoints.rs`.
  9. Stale plan doc (`docs/plans/2026-08-06-fix-text-color-plan.md`) with dangling `.facto/` references and an inaccurate claim.

Out-of-scope findings (A–E: `truncate_str` UTF-8 panic risk, `short_id` panic risk, inconsistent error handling, duplicate `color_if`, negative-token rounding) are reported only — not part of this gate, never fixed by this loop.

**Next step:** proceed to another fix cycle addressing the 1 important finding (extend the no-color test to also cover a non-dim status, e.g. `Succeeded`, asserting both the bold gate and color gate are suppressed), optionally bundling minor #2/#3 since they're cheap and adjacent to the same test code.