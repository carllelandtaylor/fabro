use std::path::Path;

use anyhow::Result;
use chrono::{DateTime, Utc};
use cli_table::format::{Border, Separator};
use cli_table::{Cell, CellStruct, Color, Style, Table};
use fabro_util::terminal::Styles;
use fabro_util::text::strip_goal_decoration;
use fabro_workflow::run_status::RunStatus;

use super::short_run_id;
use crate::args::RunsListArgs;
use crate::command_context::CommandContext;
use crate::commands::resolve_run_id;
use crate::server_runs::{ServerRunInfo, ServerRunLookup, filter_server_runs};
use crate::shared::{color_if, format_duration_ms, run_status_kind, tilde_path};

pub(crate) async fn list_command(
    args: &RunsListArgs,
    styles: &Styles,
    base_ctx: &CommandContext,
) -> Result<()> {
    let ctx = base_ctx.with_target(&args.server)?;
    let printer = ctx.printer();
    let client = ctx.server().await?;
    let parent_id = match args.parent.as_deref() {
        Some(selector) => Some(resolve_run_id(client.as_ref(), selector).await?),
        None => None,
    };
    let filtered_by_parent = parent_id.is_some();
    let lookup = match parent_id {
        Some(parent_id) => ServerRunLookup::from_client_by_parent(client, parent_id).await?,
        None => ServerRunLookup::from_client(client).await?,
    };
    let label_filters = parse_label_filters(&args.filter.label);
    let filtered = filter_server_runs(
        lookup.runs(),
        args.filter.before.as_deref(),
        args.filter.workflow.as_deref(),
        &label_filters,
        !args.all,
    );

    if ctx.json_output() {
        let json_rows: Vec<_> = filtered
            .iter()
            .map(|run| {
                serde_json::json!({
                    "run_id": run.run_id(),
                    "parent_id": run.parent_id(),
                    "workflow_name": run.workflow_name(),
                    "workflow_graph_name": run.workflow_graph_name(),
                    "workflow_slug": run.workflow_slug(),
                    "status": run.status(),
                    "start_time": run.start_time(),
                    "labels": run.labels(),
                    "wall_time_ms": run.wall_time_ms(),
                    "total_usd_micros": run.total_usd_micros(),
                    "source_directory": run.source_directory(),
                    "repo_origin_url": run.repo_origin_url(),
                    "goal": run.goal(),
                })
            })
            .collect();
        fabro_util::printout!(printer, "{}", serde_json::to_string_pretty(&json_rows)?);
        return Ok(());
    }

    if args.quiet {
        for run in &filtered {
            fabro_util::printout!(printer, "{}", run.run_id());
        }
        return Ok(());
    }

    if filtered.is_empty() {
        if args.all {
            fabro_util::printerr!(printer, "No runs found.");
        } else {
            fabro_util::printerr!(
                printer,
                "No running processes found. Use -a to show all runs (including archived)."
            );
        }
        return Ok(());
    }

    let mut display_runs = filtered;
    display_runs.reverse();
    let show_parent_column =
        !filtered_by_parent && display_runs.iter().any(|run| run.parent_id().is_some());

    let use_color = styles.use_color;
    let now = Utc::now();
    let mut title = vec!["RUN ID".cell().bold(use_color)];
    if show_parent_column {
        title.push("PARENT".cell().bold(use_color));
    }
    title.extend([
        "WORKFLOW".cell().bold(use_color),
        "STATUS".cell().bold(use_color),
        "DIRECTORY".cell().bold(use_color),
        "DURATION".cell().bold(use_color),
        "GOAL".cell().bold(use_color),
    ]);

    let rows: Vec<Vec<CellStruct>> = display_runs
        .iter()
        .map(|run| run_row(run, show_parent_column, now, use_color))
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
    fabro_util::printout!(printer, "{}", table.display()?);

    fabro_util::printerr!(printer, "\n{} run(s) listed.", display_runs.len());
    Ok(())
}

fn run_row(
    run: &ServerRunInfo,
    show_parent_column: bool,
    now: DateTime<Utc>,
    use_color: bool,
) -> Vec<CellStruct> {
    let duration_display = match run.wall_time_ms() {
        Some(ms) => format_duration_ms(ms),
        None => match run.start_time_dt() {
            Some(start) => {
                let elapsed = now.signed_duration_since(start);
                format_duration_ms(elapsed.num_milliseconds().max(0).cast_unsigned())
            }
            None => "-".to_string(),
        },
    };
    let dir_display = run
        .source_directory()
        .map_or_else(|| "-".to_string(), |p| tilde_path(Path::new(p)));
    let run_id = run.run_id().to_string();

    let mut row = vec![short_run_id(&run_id).cell().dimmed(use_color)];
    if show_parent_column {
        let parent_display = run.parent_id().map_or_else(
            || "-".to_string(),
            |parent_id| short_run_id(&parent_id.to_string()).to_string(),
        );
        row.push(parent_display.cell().dimmed(use_color));
    }
    row.extend([
        run.workflow_display_name().cell(),
        status_cell(run.status(), use_color),
        dir_display.cell(),
        duration_display.cell(),
        truncate_goal(&run.goal(), 50).cell().dimmed(use_color),
    ]);
    row
}

fn status_cell(status: RunStatus, use_color: bool) -> CellStruct {
    let text = run_status_kind(status);
    let (color, is_dim) = match status {
        RunStatus::Succeeded { .. } => (Some(Color::Green), false),
        RunStatus::Failed { .. } => (Some(Color::Red), false),
        RunStatus::Running | RunStatus::Starting | RunStatus::Runnable => {
            (Some(Color::Cyan), false)
        }
        RunStatus::Submitted | RunStatus::Pending { .. } | RunStatus::Dead => (None, true),
        RunStatus::Blocked { .. } | RunStatus::Removing => (Some(Color::Yellow), false),
        RunStatus::Paused { .. } => (Some(Color::Magenta), false),
    };
    let cell = text
        .cell()
        .bold(use_color && !is_dim)
        .dimmed(use_color && is_dim);
    match color {
        Some(c) => cell.foreground_color(color_if(use_color, c)),
        None => cell,
    }
}

fn parse_label_filters(label_args: &[String]) -> Vec<(String, String)> {
    label_args
        .iter()
        .filter_map(|s| s.split_once('='))
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

fn truncate_goal(goal: &str, max_len: usize) -> String {
    truncate_str(strip_goal_decoration(goal), max_len)
}

fn truncate_str(s: &str, max_len: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max_len {
        return s.to_string();
    }
    let truncated: String = s.chars().take(max_len - 3).collect();
    format!("{truncated}...")
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use fabro_types::status::{BlockedReason, FailureReason, PendingReason, SuccessReason};
    use fabro_types::test_support::test_principal;
    use fabro_types::{
        AskFabro, Run, RunId, RunLifecycle, RunLinks, RunOrigin, RunSize, RunTimestamps,
        WorkflowRef,
    };

    use super::*;
    use crate::commands::test_support::{has_style_escape, render_cell, render_row};
    use crate::server_runs::ServerRunInfo;

    fn test_run(run_id: RunId, parent_id: Option<RunId>, goal: &str) -> ServerRunInfo {
        ServerRunInfo::from_run(Run {
            id: run_id,
            parent_id,
            children_count: 0,
            title: "test run".to_string(),
            goal: goal.to_string(),
            workflow: WorkflowRef {
                slug:       None,
                name:       Some("test-workflow".to_string()),
                graph_name: None,
                node_count: 0,
                edge_count: 0,
            },
            automation: None,
            repository: None,
            created_by: test_principal(),
            origin: RunOrigin::default(),
            labels: HashMap::new(),
            lifecycle: RunLifecycle {
                status:          RunStatus::Running,
                approval:        None,
                pending_control: None,
                queue_position:  None,
                error:           None,
                archived:        false,
                archived_at:     None,
            },
            sandbox: None,
            models: Vec::new(),
            source_directory: None,
            timestamps: RunTimestamps {
                created_at:    run_id.created_at(),
                started_at:    None,
                last_event_at: None,
                completed_at:  None,
            },
            timing: None,
            billing: None,
            size: RunSize::default(),
            ask_fabro: AskFabro::default(),
            diff: None,
            pull_request: None,
            current_question: None,
            superseded_by: None,
            retried_from: None,
            links: RunLinks { web: None },
        })
    }

    #[test]
    fn truncate_goal_strips_markdown_headings() {
        assert_eq!(truncate_goal("## Fix bug", 50), "Fix bug");
        assert_eq!(truncate_goal("# Title", 50), "Title");
        assert_eq!(truncate_goal("### Deep heading", 50), "Deep heading");
    }

    #[test]
    fn truncate_goal_strips_plan_prefix() {
        assert_eq!(truncate_goal("Plan: do stuff", 50), "do stuff");
    }

    #[test]
    fn truncate_goal_strips_heading_and_plan_prefix() {
        assert_eq!(truncate_goal("## Plan: migrate DB", 50), "migrate DB");
    }

    #[test]
    fn truncate_goal_plain_text_unchanged() {
        assert_eq!(truncate_goal("Fix the login bug", 50), "Fix the login bug");
    }

    #[test]
    fn truncate_goal_still_truncates_after_stripping() {
        assert_eq!(
            truncate_goal("## A long goal description", 10),
            "A long ..."
        );
    }

    #[test]
    fn status_cell_dims_submitted_pending_dead_instead_of_ansi256() {
        for status in [
            RunStatus::Submitted,
            RunStatus::Pending {
                reason: PendingReason::ApprovalRequired,
            },
            RunStatus::Dead,
        ] {
            let rendered = render_cell(status_cell(status, true));
            assert!(
                rendered.contains("\x1b[2m"),
                "expected dim SGR for {status:?}, got: {rendered:?}"
            );
            assert!(
                !rendered.contains("\x1b[38;5;8m"),
                "expected no Ansi256(8) SGR for {status:?}, got: {rendered:?}"
            );
            assert!(
                !rendered.contains("\x1b[1m"),
                "expected bold to be suppressed for dim status {status:?}, got: {rendered:?}"
            );
        }
    }

    #[test]
    fn status_cell_keeps_bold_color_for_non_dim_statuses() {
        let cases = [
            (
                RunStatus::Succeeded {
                    reason: SuccessReason::Completed,
                },
                "\x1b[32m",
            ),
            (
                RunStatus::Failed {
                    reason: FailureReason::WorkflowError,
                },
                "\x1b[31m",
            ),
            (RunStatus::Running, "\x1b[36m"),
            (RunStatus::Starting, "\x1b[36m"),
            (RunStatus::Runnable, "\x1b[36m"),
            (
                RunStatus::Blocked {
                    blocked_reason: BlockedReason::HumanInputRequired,
                },
                "\x1b[33m",
            ),
            (RunStatus::Removing, "\x1b[33m"),
            (RunStatus::Paused { prior_block: None }, "\x1b[35m"),
        ];
        for (status, expected_fg) in cases {
            let rendered = render_cell(status_cell(status, true));
            assert!(
                rendered.contains("\x1b[1m"),
                "expected bold SGR for {status:?}, got: {rendered:?}"
            );
            assert!(
                rendered.contains(expected_fg),
                "expected {expected_fg:?} SGR for {status:?}, got: {rendered:?}"
            );
            assert!(
                !rendered.contains("\x1b[2m"),
                "expected no dim SGR for {status:?}, got: {rendered:?}"
            );
        }
    }

    #[test]
    fn status_cell_emits_no_escapes_when_color_disabled() {
        for status in [RunStatus::Dead, RunStatus::Succeeded {
            reason: SuccessReason::Completed,
        }] {
            let rendered = render_cell(status_cell(status, false));
            assert!(
                !has_style_escape(&rendered),
                "expected no style SGR when status_cell is built with use_color=false for {status:?}, got: {rendered:?}"
            );
        }
    }

    #[test]
    fn run_row_dims_run_id_parent_and_goal_instead_of_ansi256() {
        let parent_id = RunId::new();
        let run_id = RunId::new();
        let run = test_run(run_id, Some(parent_id), "fix the login bug");
        let now = Utc::now();

        let row = run_row(&run, true, now, true);
        let [
            run_id_cell,
            parent_cell,
            _workflow_cell,
            _status_cell,
            _dir_cell,
            _duration_cell,
            goal_cell,
        ]: [_; 7] = row
            .try_into()
            .unwrap_or_else(|_| panic!("expected 7 columns when show_parent_column is true"));

        let run_id_rendered = render_cell(run_id_cell);
        assert!(
            run_id_rendered.contains("\x1b[2m"),
            "expected RUN ID cell to carry dim SGR, got: {run_id_rendered:?}"
        );
        assert!(
            !run_id_rendered.contains("\x1b[38;5;8m"),
            "expected no Ansi256(8) SGR on RUN ID cell, got: {run_id_rendered:?}"
        );

        let parent_rendered = render_cell(parent_cell);
        assert!(
            parent_rendered.contains("\x1b[2m"),
            "expected PARENT cell to carry dim SGR, got: {parent_rendered:?}"
        );
        assert!(
            !parent_rendered.contains("\x1b[38;5;8m"),
            "expected no Ansi256(8) SGR on PARENT cell, got: {parent_rendered:?}"
        );

        let goal_rendered = render_cell(goal_cell);
        assert!(
            goal_rendered.contains("\x1b[2m"),
            "expected GOAL cell to carry dim SGR, got: {goal_rendered:?}"
        );
        assert!(
            !goal_rendered.contains("\x1b[38;5;8m"),
            "expected no Ansi256(8) SGR on GOAL cell, got: {goal_rendered:?}"
        );
    }

    #[test]
    fn run_row_emits_no_escapes_when_color_disabled() {
        let run = test_run(RunId::new(), Some(RunId::new()), "fix the login bug");
        let now = Utc::now();

        let rendered = render_row(run_row(&run, true, now, false));
        assert!(
            !has_style_escape(&rendered),
            "expected no style SGR when run_row is built with use_color=false, got: {rendered:?}"
        );
    }
}
