Trace from the reproduced symptom to the ACTUAL cause in the code — not the surface location where it manifests. Follow the data and control flow backwards from the point where the wrong behavior becomes observable until you reach the code that is genuinely wrong, and confirm your theory against the code rather than asserting it. Add instrumentation or a scratch script if that is what it takes to see the real state at the failure point.

Produce exactly two things, because they drive every later phase:

1. A one-line ROOT-CAUSE STATEMENT naming the specific code and the specific wrongness — for example 'the retry wrapper resets the backoff counter on every call because it is constructed inside the loop body'.

2. A short PROPOSED FIX APPROACH: what should change, in which files, and why that fixes the cause rather than hiding the symptom. Include how the fix avoids removing or weakening any working behavior, and what regression test would fail before it and pass after.

If the trace does not reach a cause you can state this concretely, keep digging rather than guessing — a vague root cause produces a symptom patch.