use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap};

use crate::app::{App, Dialog};

pub fn render_dialog(buf: &mut Buffer, area: Rect, dialog: &Dialog, app: &App) {
    // Calculate centered dialog rect
    let width = (area.width * 60 / 100).min(60).max(30);
    let height = match dialog {
        Dialog::NewSession { .. } => 14,
        Dialog::SearchSession { .. } => {
            // Dynamic height: 5 base + up to 8 result rows
            let n = app.sessions.len().min(8);
            (5 + n as u16).max(7)
        }
        Dialog::ConfirmKill { .. } => 7,
        Dialog::ConfirmQuit => 5,
        Dialog::Error(_) => 7,
    };

    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    let dialog_area = Rect::new(x, y, width, height);

    // Clear the background
    Clear.render(dialog_area, buf);

    match dialog {
        Dialog::NewSession {
            name_input,
            branch_input,
            base_branch_input,
            field_focus,
        } => {
            render_new_session_dialog(
                buf,
                dialog_area,
                name_input,
                branch_input,
                base_branch_input,
                *field_focus,
            );
        }
        Dialog::SearchSession { query, selected } => {
            render_search_dialog(buf, dialog_area, query, *selected, app);
        }
        Dialog::ConfirmKill {
            session_idx,
            delete_branch,
        } => {
            render_confirm_kill_dialog(buf, dialog_area, *session_idx, *delete_branch);
        }
        Dialog::ConfirmQuit => {
            render_confirm_quit_dialog(buf, dialog_area);
        }
        Dialog::Error(msg) => {
            render_error_dialog(buf, dialog_area, msg);
        }
    }
}

fn render_new_session_dialog(
    buf: &mut Buffer,
    area: Rect,
    name: &str,
    branch: &str,
    base_branch: &str,
    field_focus: usize,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(Line::from(Span::styled(
            " New Session ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));

    let inner = block.inner(area);
    block.render(area, buf);

    let field_label = |idx: usize| -> Style {
        if field_focus == idx {
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        }
    };

    let field_value = |idx: usize| -> Style {
        if field_focus == idx {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        }
    };

    let name_hint = if name.is_empty() && field_focus == 0 {
        " (shown on pane title)"
    } else {
        ""
    };

    let lines = vec![
        Line::from(vec![
            Span::styled("Session name:", field_label(0)),
            Span::styled(name_hint, Style::default().fg(Color::Rgb(80, 80, 80))),
        ]),
        Line::from(Span::styled(format!(" > {}_", name), field_value(0))),
        Line::from(""),
        Line::from(Span::styled("Branch name:", field_label(1))),
        Line::from(Span::styled(format!(" > {}_", branch), field_value(1))),
        Line::from(""),
        Line::from(Span::styled("Base branch:", field_label(2))),
        Line::from(Span::styled(format!(" > {}_", base_branch), field_value(2))),
        Line::from(""),
        Line::from(Span::styled(
            "Tab: switch field | Enter: create | Esc: cancel",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let paragraph = Paragraph::new(lines);
    paragraph.render(inner, buf);
}

fn render_confirm_kill_dialog(
    buf: &mut Buffer,
    area: Rect,
    _session_idx: usize,
    delete_branch: bool,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red))
        .title(Line::from(Span::styled(
            " Kill Session? ",
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
        )));

    let inner = block.inner(area);
    block.render(area, buf);

    let branch_toggle = if delete_branch {
        "[x] Delete branch"
    } else {
        "[ ] Delete branch (press 'b' to toggle)"
    };

    let lines = vec![
        Line::from("Worktree will be removed."),
        Line::from(""),
        Line::from(Span::styled(branch_toggle, Style::default().fg(Color::Yellow))),
        Line::from(""),
        Line::from(Span::styled(
            "Enter: confirm | Esc: cancel",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let paragraph = Paragraph::new(lines);
    paragraph.render(inner, buf);
}

fn render_confirm_quit_dialog(buf: &mut Buffer, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red))
        .title(Line::from(Span::styled(
            " Quit? ",
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
        )));

    let inner = block.inner(area);
    block.render(area, buf);

    let lines = vec![
        Line::from("Active sessions will be paused."),
        Line::from(""),
        Line::from(Span::styled(
            "Enter: confirm | Esc: cancel",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let paragraph = Paragraph::new(lines);
    paragraph.render(inner, buf);
}

fn render_error_dialog(buf: &mut Buffer, area: Rect, msg: &str) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red))
        .title(Line::from(Span::styled(
            " Error ",
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
        )));

    let inner = block.inner(area);
    block.render(area, buf);

    let lines = vec![
        Line::from(Span::styled(
            msg.to_string(),
            Style::default().fg(Color::Red),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Press Enter or Esc to dismiss",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    paragraph.render(inner, buf);
}

fn render_search_dialog(
    buf: &mut Buffer,
    area: Rect,
    query: &str,
    selected: usize,
    app: &App,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(Line::from(Span::styled(
            " Search Sessions ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));

    let inner = block.inner(area);
    block.render(area, buf);

    let mut lines: Vec<Line> = Vec::new();

    // Search input field
    lines.push(Line::from(vec![
        Span::styled(" 🔍 ", Style::default().fg(Color::Yellow)),
        Span::styled(
            format!("{}_", query),
            Style::default().fg(Color::Yellow),
        ),
    ]));
    lines.push(Line::from(""));

    // Filter sessions
    let q = query.to_lowercase();
    let matches: Vec<(usize, &crate::session::Session)> = app
        .sessions
        .iter()
        .enumerate()
        .filter(|(_, s)| {
            if query.is_empty() {
                true
            } else {
                s.name.to_lowercase().contains(&q)
                    || s.branch.to_lowercase().contains(&q)
            }
        })
        .collect();

    if matches.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No matching sessions",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for (match_idx, (pane_idx, session)) in matches.iter().enumerate() {
            let is_selected = match_idx == selected;
            let prefix = if is_selected { " ▸ " } else { "   " };
            let status_color = crate::ui::pane::status_color(session.status);

            let style = if is_selected {
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            lines.push(Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(
                    format!("[{}] ", pane_idx + 1),
                    Style::default().fg(status_color),
                ),
                Span::styled(&session.name, style),
                Span::styled(
                    format!("  {}", session.status.label()),
                    Style::default().fg(status_color),
                ),
            ]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "↑/↓: select | Enter: focus | Esc: cancel",
        Style::default().fg(Color::DarkGray),
    )));

    let paragraph = Paragraph::new(lines);
    paragraph.render(inner, buf);
}
