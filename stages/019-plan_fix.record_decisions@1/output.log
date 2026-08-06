## Final decisions

**1. `status_cell` restructuring — Option A (single match returning `(Option<Color>, is_dim: bool)`)**
Reason: smallest diff that removes the `Ansi256(8)` sentinel while keeping the existing single-match shape; a tuple used immediately in one function doesn't warrant a new struct type (CLAUDE.md: no abstractions beyond what's needed).

**2. Mechanical sites (9 of 10) — Option A (fix inline, no shared helper)**
Reason: `.dimmed(use_color)` is already a single self-explanatory call; a helper would only add indirection, and unifying `model.rs`'s separate private `color_if` is explicitly out of scope per the brief.

**3. Regression test placement — Option A (`#[cfg(test)] mod tests` in each of the four files)**
Reason: matches `docs/internal/testing-strategy.md`'s classification of rendering internals as unit/crate-level tests, and follows the existing precedent in `runs/list.rs` (`truncate_goal` test); the affected functions are private/`pub(crate)` and unreachable from `tests/it` anyway.

The developer accepted all three recommendations as-is ("your recommendations"), so no overrides to record.

## Open decisions

None. All three decisions from `plan_fix.key_decisions` were answered, and no answer surfaces a new sub-decision (Option A on #1 doesn't introduce a new type needing its own naming/placement call; Option A on #3 doesn't need a shared fixture since each file gets its own local test module). The plan can be drafted.