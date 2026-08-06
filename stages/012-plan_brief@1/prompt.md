Assemble the brief that the planning sub-workflow runs on, so it plans the FIX rather than a feature. State all of the following explicitly and completely, because the planner sees this brief and not the earlier conversation.

1. THE ROOT CAUSE — the one-line statement from the diagnosis, plus the trace that supports it and the files and functions involved.

2. THE LOGGED REPRODUCTION STEPS — preconditions and data, the ordered minimal actions, the observed wrong result and the expected result, verbatim. The plan's validation must be able to re-run exactly these.

3. THE NO-REGRESSION CONSTRAINT — do not remove, disable or weaken working functionality to make the bug disappear. Say this as a hard constraint on every step of the plan, not as a preference.

4. THE REGRESSION TEST REQUIREMENT — the plan must include a test that fails on the unfixed code and passes with the fix, wherever the project has a test suite.

5. THE ESCALATION REASON — which checklist item made this large or structural, so the plan is scoped to that reality.

Do not write the plan here; produce the brief.