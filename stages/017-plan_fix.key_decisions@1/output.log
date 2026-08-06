I'll read the actual code at the key sites to ground the decisions in real structure before presenting them.I read the four affected sites directly to ground the decisions in the real code. Here's what needs your input before drafting the plan:

**1. How to restructure `status_cell`'s color/bold/dim logic** (`runs/list.rs:171-186`)

Today, `Ansi256(8)` is overloaded: it's both the color for `Submitted|Pending|Dead` *and* the sentinel used to suppress bold (`color != Some(Color::Ansi256(8))`). Since `color.unwrap_or(Color::Ansi256(8))` is actually dead code today (every match arm already returns `Some(...)`), the real fix is to drop the color for those three statuses entirely and drive both bold-suppression and dimming off one explicit flag.

- **A. Single match returning `(Option<Color>, bool is_dim)`** — one match arm per status produces both the color and the dim flag together (e.g. `Submitted | Pending | Dead => (None, true)`), then `.bold(use_color && !is_dim).dimmed(use_color && is_dim).foreground_color(color_if(use_color, color))`. Minimal diff, keeps the existing single-match shape.
- **B. Small local struct (`struct StatusStyle { color: Option<Color>, dim: bool }`) returned by a helper fn** — more self-documenting field names, but adds a type for a single 15-line function.
- **C. Two separate matches** (one for color, one for is_dim) — avoids a tuple, but duplicates the status list and risks the two matches drifting out of sync on a future status addition.

**Recommendation: A.** It's the smallest change that removes the sentinel, and a tuple is fine for a value used immediately in the same function — a struct is unwarranted ceremony here (per CLAUDE.md: no abstractions beyond what's needed).

**2. Shared helper vs. inline fix for the 9 mechanical sites**

The 9 non-`status_cell` sites are all identical one-liners: `.foreground_color(color_if(use_color, Color::Ansi256(8)))` → `.dimmed(use_color)`. `model.rs` already has its own private `color_if` duplicate of `shared/utilities.rs`'s (out of scope to unify, per the brief).

- **A. Fix each site inline**, no new helper — `.dimmed(use_color)` is already a single self-explanatory call, nothing left to extract.
- **B. Add a small shared `dim_cell(text, use_color) -> CellStruct` helper** in each file (or `shared/utilities.rs`) to remove repetition.

**Recommendation: A.** `.dimmed(use_color)` is already as short as a helper call would be, and introducing one means touching `shared/utilities.rs` plus deciding whether to also collapse `model.rs`'s duplicate `color_if` — scope creep the brief explicitly ruled out.

**3. Regression test placement**

Only `runs/list.rs` currently has a `#[cfg(test)] mod tests` block (testing `truncate_goal`, a plain string fn — no precedent yet for asserting on rendered ANSI bytes). `model.rs`, `workflow/list.rs`, and `checkpoints.rs` have none.

- **A. Add a `#[cfg(test)] mod tests` to each of the four files**, mirroring `runs/list.rs`'s existing pattern — build the real `Vec<CellStruct>` → `.table()` → force `ColorChoice::Always` → `.display()?` → assert on the string. Keeps tests next to the code they cover.
- **B. Consolidate all four sites' regression coverage into one new test module/file** (e.g. under `tests/it`) — one place to see "did we kill Ansi256(8) everywhere," but `docs/internal/testing-strategy.md` already classifies rendering internals as unit/crate-level, not `tests/it`, and these are private/pub(crate) functions `tests/it` can't reach anyway.

**Recommendation: A.** Matches the testing-strategy guidance already surfaced in `plan_fix.analyze_codebase`, and keeps each test next to the private function it exercises rather than requiring new pub(crate) surface just for testability.

Let me know if any of these should go a different way — otherwise I'll draft the plan using recommendations A/A/A.