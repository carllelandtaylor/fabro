No additional planning docs exist — the brief at `.facto/brief.md` is the sole input, as it states. No Issue context was found (no tracker configured), so nothing to add from that source.

Here's what needs to be built:

- **Root cause**: 10 call sites across 4 `fabro-cli` files hardcode secondary/status table cell styling to `Color::Ansi256(8)` (an absolute 256-color palette index) instead of using `cli_table::CellStruct::dimmed(bool)`, the terminal-relative SGR-2 "faint" mechanism the crate already supports — causing near-invisible text on dark-background terminals.
- **Files to fix**: `runs/list.rs` (4 sites: `list_command` ×3, `status_cell` ×3 counting the sentinel), `model.rs` (2 sites), `workflow/list.rs` (1 site), `run/checkpoints.rs` (1 site).
- **Mechanical part**: replace `.foreground_color(color_if(use_color, Color::Ansi256(8)))` → `.dimmed(use_color)` at 9 of the 10 sites.
- **Non-mechanical part**: `runs/list.rs::status_cell` uses `Ansi256(8)` both as the color for `Submitted|Pending|Dead` statuses *and* as a sentinel to suppress bold (`color != Some(Color::Ansi256(8))`) — this needs restructuring to an explicit `is_dim` flag rather than a straight swap.
- **Hard constraint**: preserve existing `use_color`/`color_if` gating (NO_COLOR/non-tty behavior), leave all non-dim colors (Cyan, per-status green/red/yellow) untouched, and preserve which status maps to which visual treatment — no deleting/narrowing behavior to make the bug disappear.
- **New test required**: assert real rendered output contains `\x1b[2m` and not `\x1b[38;5;8m`, covering both a plain-dim site (e.g. `workflow/list.rs`) and the restructured `status_cell` case. Must be shown failing pre-fix, passing post-fix.
- **Validation**: re-run the exact logged repro (`cargo build -p fabro-cli` then `TERM=xterm-256color CLICOLOR_FORCE=1 ./target/debug/fabro workflow list 2>&1 | cat -v`) and confirm `\x1b[38;5;8m` is gone, replaced by `\x1b[2m`.
- **Scope**: no new abstractions, no API/schema changes — a coordinated multi-file mechanical fix plus one small logic restructuring plus a new test.

The requirements are clear, internally consistent, and fully traced (exact line numbers, confirmed crate API, confirmed repro output). Nothing to flag — moving on to the next planning phase.