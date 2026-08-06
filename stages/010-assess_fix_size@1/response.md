**Decision: ESCALATE**

Trigger: the checklist item "touches more than about two files" — the fix requires changes across four files (`lib/apps/fabro-cli/src/commands/runs/list.rs`, `model.rs`, `workflow/list.rs`, `run/checkpoints.rs`), plus a new regression test asserting on emitted ANSI/SGR bytes. It also involves a small logic restructuring in `runs/list.rs::status_cell` (replacing the `Ansi256(8)` sentinel comparison with an explicit `is_dim` flag), not a pure search-and-replace. No new abstraction/interface, schema/API/migration change is involved — only the file-count criterion triggered escalation.

Ending this stage as **SUCCEEDED** (escalated path).