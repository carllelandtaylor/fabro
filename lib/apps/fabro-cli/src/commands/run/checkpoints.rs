use anyhow::Result;
use cli_table::format::{Border, Separator};
use cli_table::{Cell, CellStruct, Color, Style, Table};
use fabro_api::types::TimelineEntryResponse;
use fabro_types::RunId;
use fabro_util::printer::Printer;
use fabro_util::terminal::Styles;
use git2::Repository;
use serde::Serialize;

use crate::server_client::Client;
use crate::shared::color_if;
use crate::shared::repo::ensure_matching_repo_origin;

#[derive(Serialize)]
pub(crate) struct TimelineEntryJson {
    ordinal:        usize,
    node_name:      String,
    visit:          usize,
    run_commit_sha: Option<String>,
}

pub(crate) async fn ensure_origin_if_local(
    client: &Client,
    run_id: &RunId,
    verb: &str,
) -> Result<()> {
    if Repository::discover(".").is_err() {
        return Ok(());
    }

    let state = client.get_run_state(run_id).await?;
    ensure_matching_repo_origin(state.spec.repo_origin_url(), verb)?;
    Ok(())
}

pub(crate) fn timeline_entries_json(entries: &[TimelineEntryResponse]) -> Vec<TimelineEntryJson> {
    entries
        .iter()
        .map(|entry| TimelineEntryJson {
            ordinal:        usize::try_from(entry.ordinal.get())
                .expect("timeline ordinal should fit in usize"),
            node_name:      entry.node_name.clone(),
            visit:          usize::try_from(entry.visit.get())
                .expect("timeline visit should fit in usize"),
            run_commit_sha: entry.run_commit_sha.clone(),
        })
        .collect()
}

pub(crate) fn short_id(run_id: &str) -> &str {
    &run_id[..8.min(run_id.len())]
}

fn timeline_row(entry: &TimelineEntryJson, use_color: bool) -> Vec<CellStruct> {
    let ordinal_str = format!("@{}", entry.ordinal);
    let mut details = Vec::new();
    if entry.visit > 1 {
        details.push(format!("visit {}, loop", entry.visit));
    }
    if entry.run_commit_sha.is_none() {
        details.push("no run commit".to_string());
    }

    let detail_str = if details.is_empty() {
        String::new()
    } else {
        format!("({})", details.join(", "))
    };

    vec![
        ordinal_str
            .cell()
            .foreground_color(color_if(use_color, Color::Cyan)),
        entry.node_name.clone().cell(),
        detail_str.cell().dimmed(use_color),
    ]
}

pub(crate) fn print_timeline(entries: &[TimelineEntryJson], styles: &Styles, printer: Printer) {
    if entries.is_empty() {
        fabro_util::printerr!(printer, "No checkpoints found.");
        return;
    }

    let use_color = styles.use_color;
    let title = vec![
        "@".cell().bold(use_color),
        "Node".cell().bold(use_color),
        "Details".cell().bold(use_color),
    ];

    let rows: Vec<Vec<CellStruct>> = entries
        .iter()
        .map(|entry| timeline_row(entry, use_color))
        .collect();

    let color_choice = if use_color {
        cli_table::ColorChoice::Auto
    } else {
        cli_table::ColorChoice::Never
    };
    let table = rows
        .table()
        .title(title)
        .color_choice(color_choice)
        .border(Border::builder().build())
        .separator(Separator::builder().build());
    #[allow(
        clippy::print_stderr,
        reason = "The checkpoint timeline table is operator feedback, not command output."
    )]
    if let Ok(display) = table.display() {
        for line in display.to_string().lines() {
            eprintln!("{}", line.trim_end());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::test_support::{has_style_escape, render_row};

    fn test_entry() -> TimelineEntryJson {
        TimelineEntryJson {
            ordinal:        2,
            node_name:      "some_node".to_string(),
            visit:          2,
            run_commit_sha: None,
        }
    }

    #[test]
    fn timeline_row_ordinal_cyan_and_detail_dims_instead_of_ansi256() {
        // Columns: @, Node, Details.
        let row = timeline_row(&test_entry(), true);
        let [ordinal_cell, _node_cell, detail_cell]: [_; 3] = row
            .try_into()
            .unwrap_or_else(|_| panic!("expected 3 columns"));

        let ordinal_cell = render_row(vec![ordinal_cell]);
        assert!(
            ordinal_cell.contains("\x1b[36m"),
            "expected ordinal cell to carry Cyan SGR, got: {ordinal_cell:?}"
        );

        let detail_cell = render_row(vec![detail_cell]);
        assert!(
            detail_cell.contains("\x1b[2m"),
            "expected DETAILS cell to carry dim SGR, got: {detail_cell:?}"
        );
        assert!(
            !detail_cell.contains("\x1b[38;5;8m"),
            "expected no Ansi256(8) SGR on DETAILS cell, got: {detail_cell:?}"
        );
    }

    #[test]
    fn detail_cell_no_color_has_no_escape_bytes() {
        let rendered = render_row(timeline_row(&test_entry(), false));
        assert!(
            !has_style_escape(&rendered),
            "expected no style SGR when timeline_row is built with use_color=false, got: {rendered:?}"
        );
    }
}
