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
///
/// Each CSI sequence is parsed as `\x1b[<params>ml<terminator>`, where
/// `<params>` is digits/semicolons; only sequences terminated by `m` are SGR
/// codes. Non-SGR CSI sequences (e.g. `\x1b[2K` cursor/erase controls) are
/// skipped rather than folded into the next `m`-terminated sequence, which
/// would misattribute their bytes as part of an unrelated SGR code.
pub(crate) fn has_style_escape(rendered: &str) -> bool {
    let mut rest = rendered;
    while let Some(seq_start) = rest.find("\x1b[") {
        rest = &rest[seq_start + 2..];
        let Some(terminator_idx) = rest.find(|c: char| !(c.is_ascii_digit() || c == ';')) else {
            break;
        };
        let terminator = rest.as_bytes()[terminator_idx];
        let code = &rest[..terminator_idx];
        rest = &rest[terminator_idx + 1..];
        if terminator != b'm' {
            continue;
        }
        if !(code.is_empty() || code == "0") {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::has_style_escape;

    #[test]
    fn detects_dim_sgr() {
        assert!(has_style_escape("\x1b[2mtext\x1b[0m"));
    }

    #[test]
    fn ignores_bare_reset() {
        assert!(!has_style_escape("\x1b[0mtext\x1b[0m"));
    }

    #[test]
    fn ignores_non_sgr_csi_sequences() {
        assert!(!has_style_escape("\x1b[2Ktext"));
    }

    #[test]
    fn non_sgr_csi_does_not_mask_a_later_reset_as_styled() {
        // A naive scanner that hunts for the next 'm' after any "\x1b[" would
        // fold "2K\x1b[0" into one bogus "code", misreading the reset as a
        // real style. The fix must still see this as unstyled.
        assert!(!has_style_escape("\x1b[2K\x1b[0m"));
    }

    #[test]
    fn still_detects_sgr_after_a_non_sgr_csi_sequence() {
        assert!(has_style_escape("\x1b[2K\x1b[2m"));
    }
}
