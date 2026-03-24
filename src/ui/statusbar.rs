use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

use crate::app::App;
use crate::keys::InputMode;
use crate::tiling::LayoutMode;

pub fn render_statusbar(buf: &mut Buffer, area: Rect, app: &App) {
    if area.height == 0 {
        return;
    }

    let mut spans = Vec::new();

    // Layout indicator
    let layout_name = app.layout_mode.name();
    spans.push(Span::styled(
        format!(" {} ", layout_name),
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ));

    spans.push(Span::raw(" "));

    // Session count
    let total = app.sessions.len();
    let active = app
        .sessions
        .iter()
        .filter(|s| {
            s.status == crate::session::status::SessionStatus::Running
                || s.status == crate::session::status::SessionStatus::Waiting
        })
        .count();

    spans.push(Span::styled(
        format!(" {}/{} sessions ", active, total),
        Style::default().fg(Color::White).bg(Color::DarkGray),
    ));

    spans.push(Span::raw(" "));

    // Monocle indicator
    if app.layout_mode == LayoutMode::Monocle && total > 0 {
        spans.push(Span::styled(
            format!(" [{}/{}] ", app.focused + 1, total),
            Style::default().fg(Color::Yellow),
        ));
        spans.push(Span::raw(" "));
    }

    // Focused session info
    if let Some(session) = app.sessions.get(app.focused) {
        let color = crate::ui::pane::status_color(session.status);
        spans.push(Span::styled(
            format!(" {} ", session.branch),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw(" "));
    }

    // Input mode
    let mode_span = match app.input_mode {
        InputMode::Normal => Span::styled(
            " NORMAL ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        InputMode::PaneFocused => Span::styled(
            " INPUT ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Red)
                .add_modifier(Modifier::BOLD),
        ),
        InputMode::Dialog => Span::styled(
            " DIALOG ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
    };
    spans.push(mode_span);

    // Key hints (right-aligned)
    let hints = match app.input_mode {
        InputMode::Normal => "n:new j/k:nav f:enter q:kill Q:quit",
        InputMode::PaneFocused => "Esc:exit pane",
        InputMode::Dialog => "Enter:confirm Esc:cancel",
    };

    // Calculate remaining space for right-aligned hints
    let left_len: usize = spans.iter().map(|s| s.content.len()).sum();
    let hints_len = hints.len();
    let padding = if area.width as usize > left_len + hints_len + 1 {
        area.width as usize - left_len - hints_len - 1
    } else {
        1
    };

    spans.push(Span::raw(" ".repeat(padding)));
    spans.push(Span::styled(
        hints.to_string(),
        Style::default().fg(Color::DarkGray),
    ));

    let line = Line::from(spans);
    buf.set_line(area.x, area.y, &line, area.width);
}
