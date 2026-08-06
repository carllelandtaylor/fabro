Goal: Fix a reported bug end-to-end and autonomously: reproduce it, diagnose the real root cause, fix it with adaptive escalation, verify against the logged repro steps, open a pull request, and drive CI to green

## Completed stages
- **understand_locate**: succeeded
  - Files: .facto/bug-brief.md
- **repro.understand_report**: succeeded
  - Files: .facto/bug-brief.md
- **repro.bring_up_app**: succeeded
  - Files: .facto/bug-brief.md
- **repro.attempt_repro**: succeeded
- **repro.confirm_distill**: succeeded
- **repro.report_result**: succeeded
  - Files: .facto/result.md
- **check_repro**: succeeded
- **diagnose**: succeeded
  - Files: .facto/bug-brief.md


Choose between the direct path and the escalated path for the fix approach you just proposed. Escalation is purely about heavier tooling for a bigger change — it is never a request for developer review, and either path runs autonomously to completion.

ESCALATE if ANY of these holds: the fix touches more than about two files, or touches shared or core logic that many things depend on; it introduces a new abstraction or interface; it requires a schema, API or migration change; or it is otherwise large or multi-step.

Otherwise the fix is small and localized, and takes the DIRECT path.

End this stage as SUCCEEDED if the checklist says escalate. End this stage as FAILED if the fix is small and localized and takes the direct path. State which checklist item triggered escalation, or state that none did, so the choice is auditable.