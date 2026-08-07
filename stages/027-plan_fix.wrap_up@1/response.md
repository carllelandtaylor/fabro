The plan is committed at `4193b3b`, containing **4 steps** (`runs/list.rs`, `workflow/list.rs`, `run/checkpoints.rs`, `model.rs`).

**Plan path:** `/repos/carllelandtaylor/fabro/docs/plans/2026-08-06-fix-text-color-plan.md`

Before execution, be aware:

- **One manual-only, low-expertise row**: actual visual legibility on a real dark-background terminal can't be automated (SGR 2 "faint" is terminal-relative). Automated tests can only prove `\x1b[2m` is emitted instead of `\x1b[38;5;8m` — someone needs to eyeball a real terminal.
- **Two blocked-no-tooling manual checks**: `fabro ps` (`runs/list.rs::status_cell`) and `fabro run checkpoints <run-id>` (`checkpoints.rs::print_timeline`) both require a live server, unavailable in this sandbox — covered only by direct unit tests, not a live run.
- No test-framework gap — `cargo nextest` is already in place; the plan just adds new `#[cfg(test)]` modules following existing conventions.