#![cfg(test)]

use cli_table::{CellStruct, Table};

pub(crate) fn render_row(row: Vec<CellStruct>, use_color: bool) -> String {
    let color_choice = if use_color {
        cli_table::ColorChoice::Always
    } else {
        cli_table::ColorChoice::Never
    };
    vec![row]
        .table()
        .color_choice(color_choice)
        .display()
        .unwrap()
        .to_string()
}

pub(crate) fn render_cell(cell: CellStruct, use_color: bool) -> String {
    render_row(vec![cell], use_color)
}

/// `cli-table` emits an unconditional `\x1b[0m` reset around every cell/border
/// under `ColorChoice::Always`, even when no style was applied. So "no color"
/// assertions must check for the absence of specific style SGRs (bold, dim,
/// foreground color) rather than the absence of any escape byte.
pub(crate) fn has_style_escape(rendered: &str) -> bool {
    rendered.contains("\x1b[1m") || rendered.contains("\x1b[2m") || rendered.contains("\x1b[3")
}
