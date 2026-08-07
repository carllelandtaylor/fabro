## Review classification

**Outcome: 2 — IN-SCOPE findings include important-severity items. Another full cycle follows the fixes.**

**Cycle mode: CONTINUING CYCLE** (cycle 1 of 5 max; nowhere near the cap, so this is a normal continuation, not a capped final cycle.)

**In-scope finding count by severity:**
- Critical: 0
- Important: 2
  1. `workflow/list.rs:163-183` / `run/checkpoints.rs:136-156` — new tests build their own cell literal instead of exercising the actual fixed production code (`print_section`/`print_timeline`); mutation-verified to pass even when the real fix is reverted.
  2. `runs/list.rs:316-323`, `model.rs:680-689`, `workflow/list.rs:176-183`, `run/checkpoints.rs:149-156` — all four `use_color=false` tests render with `ColorChoice::Never`, which strips styling unconditionally regardless of whether `use_color` gating actually works; mutation-verified to pass even when gating is broken.
- Minor: 5
  3. `runs/list.rs:254-273` — dim-status test doesn't assert bold suppression.
  4. `model.rs:665-678` — asserts `\x1b[2m` appears somewhere, not that both PROVIDER and ALIASES cells are individually dimmed.
  5. `.facto/*.md` scratch artifacts committed to the branch (not a repo convention).
  6. `render_cell`/`render_row` test helper duplicated near-verbatim across all four files.
  7. `Starting`, `Runnable`, `Removing` status variants untested in `status_cell` test cases.

Out-of-scope findings (A–D: `truncate_str` UTF-8 panic risk, inconsistent error handling, `short_id` panic risk, duplicate `color_if`) are reported only — not part of this gate and never fixed by this loop.

**Next step:** proceed to another fix cycle addressing the 2 important findings (tests must call real production code and use `ColorChoice::Always` with SGR-presence assertions to actually detect broken `use_color` gating), ideally bundling the minor fixes (#3, #4) while touching the same test code.