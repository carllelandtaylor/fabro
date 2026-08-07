Step 1's changes (`runs/list.rs`) were already committed by the prior stage. Let me verify that the code matches step 1's spec exactly before moving on to step 2.Good, step 1 is complete (only a string literal in an assertion message remains, no actual `Color::Ansi256(8)` usage). Now implementing Step 2: fix `workflow/list.rs`.Now implementing Step 2. First the mechanical fix in `print_section`:Build succeeds. Now running the new tests.I'll wait for the test run to finish in the background.Good — only the assertion-message string remains, no actual `Color::Ansi256(8)` usage. Now waiting for the background test run.I'll wait for the background nextest run notification rather than polling further.I already have the nextest run going in the background (task bwkmtwd14) — I'll wait for its completion notification rather than starting a redundant monitor.

I've implemented **Step 2: Fix `workflow/list.rs`** from the plan.

**Mode:** Inline — matches the Phase 1 choice (small, single-file, fully-specified diff), still appropriate since the change was exactly as scoped.

**Files changed:**
- `lib/apps/fabro-cli/src/commands/workflow/list.rs`: replaced `.foreground_color(color_if(use_color, Color::Ansi256(8)))` on the `goal_str` cell in `print_section` with `.dimmed(use_color)`; added a new `#[cfg(test)] mod tests` block with `goal_cell_dims_instead_of_ansi256` (asserts `\x1b[2m` present, `\x1b[38;5;8m` absent) and `goal_cell_no_color_has_no_escape_bytes` (asserts no `\x1b[` bytes when `use_color = false`).

`cargo build -p fabro-cli` passed. `rg -n "Ansi256\(8\)"` on this file now only matches an assertion-message string, not actual code. Test run (`cargo nextest run -p fabro-cli -- workflow::list`) is running in the background; awaiting result before reporting final validation.

**Commit message for this step (not yet committed, per instructions):**
```
fix: dim workflow description column instead of hardcoding Ansi256(8)

Context:
`fabro workflow list`'s DESCRIPTION cell used the absolute palette
color `Ansi256(8)`, making it nearly invisible on dark-background
terminals, while the section header two lines above already uses the
project's `styles.dim` primitive for the same visual intent. Switching
to `cli_table::CellStruct::dimmed(bool)` applies the equivalent
terminal-relative SGR 2 treatment to the table cell.

Verification:
Automated: `cargo nextest run -p fabro-cli -- workflow::list`
Manual:
1. `cargo build -p fabro-cli`
2. `TERM=xterm-256color CLICOLOR_FORCE=1 ./target/debug/fabro workflow list 2>&1 | cat -v`
3. Expect the DESCRIPTION column to emit `\x1b[2m`, never `\x1b[38;5;8m`.
```

**Validation instructions (verbatim from plan, for the next stage):**
- [ ] `cargo build -p fabro-cli`
- [ ] `cargo nextest run -p fabro-cli -- workflow::list` — new tests pass
- [ ] `rg -n "Ansi256\(8\)" lib/apps/fabro-cli/src/commands/workflow/list.rs` — no matches
- [ ] `cargo +nightly-2026-04-14 fmt --check --all`
- [ ] `cargo +nightly-2026-04-14 clippy -p fabro-cli --all-targets -- -D warnings`
- [ ] Re-run the logged repro: `TERM=xterm-256color CLICOLOR_FORCE=1 ./target/debug/fabro workflow list 2>&1 | cat -v` shows `\x1b[2m` on the DESCRIPTION cell, not `\x1b[38;5;8m`