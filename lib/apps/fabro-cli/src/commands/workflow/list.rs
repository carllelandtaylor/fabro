use anyhow::{Result, bail};
use cli_table::format::{Border, Separator};
use cli_table::{Cell, CellStruct, Color, Style, Table};
use fabro_config::project::{
    WorkflowInfo, WorkflowSource, discover_project_config, list_workflows_detailed,
};
use fabro_util::printer::Printer;
use fabro_util::terminal::Styles;

use crate::args::WorkflowListArgs;
use crate::command_context::CommandContext;
use crate::shared::{color_if, print_json_pretty, relative_path};

const GOAL_MAX_LEN: usize = 60;

pub(super) fn list_command(_args: &WorkflowListArgs, base_ctx: &CommandContext) -> Result<()> {
    let printer = base_ctx.printer();
    let styles = Styles::detect_stderr();
    let cwd = std::env::current_dir()?;

    let Some(config_path) = discover_project_config(&cwd)? else {
        bail!(
            "No .fabro/project.toml found in {cwd} or any parent directory",
            cwd = cwd.display()
        );
    };

    let fabro_root = config_path
        .parent()
        .expect("project config should have a parent directory");
    let project_wf_dir = fabro_root.join("workflows");
    let user_wf_dir = Some(fabro_util::Home::from_env().workflows_dir());

    let workflows = list_workflows_detailed(Some(&project_wf_dir), user_wf_dir.as_deref());

    if base_ctx.json_output() {
        print_json_pretty(&workflows)?;
        return Ok(());
    }

    let project: Vec<_> = workflows
        .iter()
        .filter(|w| w.source == WorkflowSource::Project)
        .collect();
    let user: Vec<_> = workflows
        .iter()
        .filter(|w| w.source == WorkflowSource::User)
        .collect();

    fabro_util::printerr!(
        printer,
        "{} workflow(s) found\n",
        styles.bold.apply_to(workflows.len())
    );

    let user_path = user_wf_dir
        .as_deref()
        .map_or_else(|| "~/.fabro/workflows".to_string(), relative_path);
    print_section("User Workflows", &user_path, &user, &styles, printer);

    fabro_util::printerr!(printer, "");

    print_section(
        "Project Workflows",
        &relative_path(&project_wf_dir),
        &project,
        &styles,
        printer,
    );

    Ok(())
}

fn print_section(
    title: &str,
    path: &str,
    workflows: &[&WorkflowInfo],
    styles: &Styles,
    printer: Printer,
) {
    fabro_util::printerr!(
        printer,
        "{} {}",
        styles.bold.apply_to(title),
        styles.dim.apply_to(format!("({path})")),
    );
    if workflows.is_empty() {
        fabro_util::printerr!(printer, "  {}", styles.dim.apply_to("(none)"));
        return;
    }

    let use_color = styles.use_color;
    let title_row: Vec<CellStruct> = vec![
        "NAME".cell().bold(use_color),
        "DESCRIPTION".cell().bold(use_color),
    ];

    let rows: Vec<Vec<CellStruct>> = workflows
        .iter()
        .map(|w| workflow_row(w, use_color))
        .collect();

    let color_choice = if use_color {
        cli_table::ColorChoice::Auto
    } else {
        cli_table::ColorChoice::Never
    };
    let table = rows
        .table()
        .title(title_row)
        .color_choice(color_choice)
        .border(Border::builder().build())
        .separator(Separator::builder().build());
    fabro_util::printerr!(
        printer,
        "{}",
        table
            .display()
            .expect("rendering the workflow table should succeed")
    );
}

fn workflow_row(w: &WorkflowInfo, use_color: bool) -> Vec<CellStruct> {
    let goal_str = w
        .goal
        .as_deref()
        .map(|g| truncate_str(g, GOAL_MAX_LEN))
        .unwrap_or_default();
    vec![
        w.name
            .clone()
            .cell()
            .foreground_color(color_if(use_color, Color::Cyan)),
        goal_str.cell().dimmed(use_color),
    ]
}

fn truncate_str(s: &str, max: usize) -> String {
    let first_line = s.lines().next().unwrap_or(s);
    if first_line.len() <= max {
        first_line.to_string()
    } else {
        format!("{}...", &first_line[..max - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::test_support::{has_style_escape, render_row};

    fn test_workflow(goal: &str) -> WorkflowInfo {
        WorkflowInfo {
            name:   "wf".to_string(),
            goal:   Some(goal.to_string()),
            source: WorkflowSource::Project,
        }
    }

    #[test]
    fn name_cell_has_cyan_color() {
        let workflow = test_workflow("goal text");
        let name_cell = render_row(vec![
            workflow_row(&workflow, true).into_iter().next().unwrap(),
        ]);
        assert!(
            name_cell.contains("\x1b[36m"),
            "expected NAME cell to carry Cyan SGR, got: {name_cell:?}"
        );
    }

    #[test]
    fn goal_cell_dims_instead_of_ansi256() {
        let workflow = test_workflow("goal text");
        let description_cell = render_row(vec![
            workflow_row(&workflow, true).into_iter().nth(1).unwrap(),
        ]);
        assert!(
            description_cell.contains("\x1b[2m"),
            "expected DESCRIPTION cell to carry dim SGR, got: {description_cell:?}"
        );
        assert!(
            !description_cell.contains("\x1b[38;5;8m"),
            "expected no Ansi256(8) SGR on DESCRIPTION cell, got: {description_cell:?}"
        );
    }

    #[test]
    fn goal_cell_no_color_has_no_escape_bytes() {
        let workflow = test_workflow("goal text");
        let rendered = render_row(workflow_row(&workflow, false));
        assert!(
            !has_style_escape(&rendered),
            "expected no style SGR when workflow_row is built with use_color=false, got: {rendered:?}"
        );
    }
}
