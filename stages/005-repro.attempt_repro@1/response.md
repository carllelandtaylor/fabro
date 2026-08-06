The `repro.bring_up_app` stage is already complete — it's committed as `6d0c237` and the working tree is clean with no uncommitted changes.

Checking `.facto/bug-brief.md`, the reproduction was already performed and recorded in full:

- Built `fabro-cli` and ran `TERM=xterm-256color CLICOLOR_FORCE=1 ./target/debug/fabro workflow list` under a pty
- Observed the exact symptom: the DESCRIPTION cell emits literal `\x1b[38;5;8m` (absolute 256-color palette index 8), contrasted against the adjacent NAME cell's `\x1b[36m` — confirming the hardcoded `Color::Ansi256(8)` reaches the output stream exactly as the reporter described, and that this isn't a global color-suppression artifact
- Resolved the reporter's open technical question by reading the `cli-table` 0.5.0 and `termcolor` 1.4.1 crate sources: `CellStruct` already exposes `.dimmed(bool)` (via the `Style` trait, already imported in all four affected files), which emits SGR 2 "faint" — a terminal-relative attribute — with no ANSI-in-column-width concern
- Noted `fabro ps` (the `runs/list.rs` STATUS-color case) couldn't be driven live (needs a running server), so it was verified statically only

The brief's Root cause section is correctly left `Unknown` for the diagnosis stage, and all other established facts (preconditions, reproduction steps, evidence, code locations) are filled in. Nothing further is needed for this stage — it's already succeeded and persisted.