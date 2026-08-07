Confirmed: no `design-mock.html` or `design-decisions.html` exists anywhere in the repo, and the working tree is clean.

This stage — "fill in the frames and bands" of a design-mock board — doesn't apply here. This is a terminal CLI bug fix (hardcoded `Color::Ansi256(8)` → `.dimmed()` in `cli-table` cell rendering across four `fabro-cli` files). It has no rendered UI screen, no `apps/fabro-web` change, no flows, no frames, no bands. Four consecutive prior stages in this same design-review branch (`detect_spec`, `a_enumerate`, `a_mock_brief`, `mock.working_copy`, `mock.fill_tokens`) already independently reached this conclusion:

- `detect_spec` found `NO_MOCK` and routed to the sanity-check path
- `a_enumerate` found no screens to inventory — the diff touches only `lib/apps/fabro-cli/src/commands/**` and doc/plan files
- `a_mock_brief` and `mock.working_copy`/`mock.fill_tokens` all declined for the same reason, and additionally noted `<dest-dir>` is never resolved to a real path in this branch

I won't fabricate a `design-mock.html`, invent frames/bands/flows for a change that has none, or guess a destination directory. This looks like a pipeline routing gap — `path_gate` isn't fully short-circuiting the rest of the design-review branch for CLI-only changes — worth fixing at the pipeline definition level so this doesn't re-prompt at every remaining sub-stage.

What actually still needs work, per `implement_fix.test_plan` (the last stage that touched real code):

1. `render_cell` test helpers in `runs/list.rs` and `workflow/list.rs` hardcode `ColorChoice::Always` instead of branching on `use_color` — breaks the `*_no_color_has_no_escape_bytes` tests.
2. Clippy `absolute_paths` errors in `runs/list.rs`'s new test module (missing imports for `PendingReason`/`SuccessReason`/etc.).
3. Plan Steps 3 (`run/checkpoints.rs`) and 4 (`model.rs`) are unimplemented — `Ansi256(8)` still present at both sites.

Let me know if you'd like me to pivot to fixing those now.