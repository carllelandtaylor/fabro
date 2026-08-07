use cli_table::{CellStruct, Table};

pub(crate) fn render_row(row: Vec<CellStruct>) -> String {
    vec![row]
        .table()
        .color_choice(cli_table::ColorChoice::Always)
        .display()
        .unwrap()
        .to_string()
}

pub(crate) fn render_cell(cell: CellStruct) -> String {
    render_row(vec![cell])
}

/// `cli-table` emits an unconditional `\x1b[0m` reset around every cell/border
/// under `ColorChoice::Always`, even when no style was applied. So "no color"
/// assertions must check for the presence of a non-reset SGR code, not just
/// any escape byte.
pub(crate) fn has_style_escape(rendered: &str) -> bool {
    let mut rest = rendered;
    while let Some(seq_start) = rest.find("\x1b[") {
        rest = &rest[seq_start + 2..];
        let Some(end) = rest.find('m') else {
            break;
        };
        let code = &rest[..end];
        if !(code.is_empty() || code == "0") {
            return true;
        }
        rest = &rest[end + 1..];
    }
    false
}
