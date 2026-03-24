use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap};

use crate::app::{App, Dialog};

pub fn render_dialog(buf: &mut Buffer, area: Rect, dialog: &Dialog, app: &App) {
    let width = (area.width * 60 / 100).min(60).max(30);
    let height = match dialog {
        Dialog::NewSession { .. } => 7,
        Dialog::SearchSession { .. } => {
            let n = app.sessions.len().min(8);
            (5 + n as u16).max(7)
        }
        Dialog::ConfirmKill { .. } => 5,
        Dialog::ConfirmQuit => 5,
        Dialog::Error(_) => 7,
    };

    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    let dialog_area = Rect::new(x, y, width, height);

    Clear.render(dialog_area, buf);

    match dialog {
        Dialog::NewSession { name_input } => {
            render_new_session_dialog(buf, dialog_area, name_input);
        }
        Dialog::SearchSession { query, selected } => {
            render_search_dialog(buf, dialog_area, query, *selected, app);
        }
        Dialog::ConfirmKill { session_idx } => {
            render_confirm_kill_dialog(buf, dialog_area, *session_idx, app);
        }
        Dialog::ConfirmQuit => {
            render_confirm_quit_dialog(buf, dialog_area);
        }
        Dialog::Error(msg) => {
            render_error_dialog(buf, dialog_area, msg);
        }
    }
}

fn render_new_session_dialog(buf: &mut Buffer, area: Rect, name: &str) {
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

    let hint = if name.is_empty() {
        " (e.g. LSP-14939, fix-auth, etc.)"
    } else {
        ""
    };

    let lines = vec![
        Line::from(vec![
            Span::styled(
                "Session name:",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(hint, Style::default().fg(Color::Rgb(80, 80, 80))),
        ]),
        Line::from(Span::styled(
            format!(" > {}_", name),
            Style::default().fg(Color::Yellow),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Enter: create | Esc: cancel",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let paragraph = Paragraph::new(lines);
    paragraph.render(inner, buf);
}

fn render_confirm_kill_dialog(buf: &mut Buffer, area: Rect, session_idx: usize, app: &App) {
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

    let session_name = app
        .sessions
        .get(session_idx)
        .map(|s| s.name.as_str())
        .unwrap_or("unknown");

    let lines = vec![
        Line::from(format!("Kill \"{}\"?", session_name)),
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
        Line::from("Active sessions will be killed."),
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

    lines.push(Line::from(vec![
        Span::styled(" > ", Style::default().fg(Color::Yellow)),
        Span::styled(
            format!("{}_", query),
            Style::default().fg(Color::Yellow),
        ),
    ]));
    lines.push(Line::from(""));

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
            let prefix = if is_selected { " > " } else { "   " };
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
        "Up/Down: select | Enter: focus | Esc: cancel",
        Style::default().fg(Color::DarkGray),
    )));

    let paragraph = Paragraph::new(lines);
    paragraph.render(inner, buf);
}
