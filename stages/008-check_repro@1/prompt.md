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


FIRST, read both `.facto/bug-brief.md` and `.facto/result.md`. The brief is the shared record every stage of this workflow reads and updates; `result.md` is the report the reproduction sub-workflow wrote. That sub-workflow ran on its own thread, so this stage cannot see its conversation and those two files are your only account of what it did. Treat them as fact and never substitute a guess; if either is missing or does not cover what you need, say so explicitly rather than inventing the detail.

Read the reproduction report produced by the previous stage and decide whether the bug was actually reproduced.

Save the confirmed repro verbatim into the brief's `Preconditions, inputs and environment`, `Reproduction steps` and `Evidence` sections, so later phases can re-run it exactly: the PRECONDITIONS and data, the ORDERED minimal steps, the OBSERVED (wrong) result, the EXPECTED result, and the evidence. Every verification in this workflow runs against exactly these steps, so restate them in full rather than summarizing them away.

End this stage as SUCCEEDED only if the report says the bug was reproduced and someone was observed to see the wrong behavior. End this stage as FAILED if the report is a could-not-reproduce result. Never upgrade a could-not-reproduce into a reproduction, and never invent steps the report does not contain.