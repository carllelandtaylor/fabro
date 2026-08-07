No design mock exists anywhere in the repo, and the `detect_spec` stage already confirmed this (`NO_MOCK`) and exited to the sanity-check path. I confirmed this independently — a repo-wide search for `design-mock.html` / `design-decisions.html` finds nothing.

**Screen inventory: none — out of scope for this task.**

This is a terminal CLI output bug (hardcoded `Color::Ansi256(8)` instead of `.dimmed()` in table rendering), not a UI change. Cross-checking against the branch diff (`git diff --name-only` from the detect stage):

```
.facto/brief.md
.facto/bug-brief.md
.facto/result.md
docs/plans/2026-08-06-fix-text-color-plan.md
lib/apps/fabro-cli/src/commands/runs/list.rs
lib/apps/fabro-cli/src/commands/workflow/list.rs
```

Every changed file is either a Rust CLI source file (`lib/apps/fabro-cli/src/commands/**`) or a planning/doc artifact. None of `apps/fabro-web/**` (the only part of this codebase with rendered screens/mocks) is touched, and the fix doesn't add or alter any browser-rendered surface — it changes ANSI escape sequences emitted to a terminal by `cli-table`. There is no frame, band, route, viewport, or app screen for this change to reproduce.

This design-review branch does not apply to the current task. The pipeline should proceed past screen enumeration without producing a screen list.