use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Widget};

use crate::session::status::SessionStatus;

/// Convert a vt100 color to a ratatui Color
fn vt100_color_to_ratatui(color: vt100::Color) -> Color {
    match color {
        vt100::Color::Default => Color::Reset,
        vt100::Color::Idx(0) => Color::Black,
        vt100::Color::Idx(1) => Color::Red,
        vt100::Color::Idx(2) => Color::Green,
        vt100::Color::Idx(3) => Color::Yellow,
        vt100::Color::Idx(4) => Color::Blue,
        vt100::Color::Idx(5) => Color::Magenta,
        vt100::Color::Idx(6) => Color::Cyan,
        vt100::Color::Idx(7) => Color::White,
        vt100::Color::Idx(8) => Color::DarkGray,
        vt100::Color::Idx(9) => Color::LightRed,
        vt100::Color::Idx(10) => Color::LightGreen,
        vt100::Color::Idx(11) => Color::LightYellow,
        vt100::Color::Idx(12) => Color::LightBlue,
        vt100::Color::Idx(13) => Color::LightMagenta,
        vt100::Color::Idx(14) => Color::LightCyan,
        vt100::Color::Idx(15) => Color::White,
        vt100::Color::Idx(n) => Color::Indexed(n),
        vt100::Color::Rgb(r, g, b) => Color::Rgb(r, g, b),
    }
}

/// Render a vt100 Parser's screen into ratatui Lines
pub fn render_terminal_screen(
    parser: &vt100::Parser,
    rows: u16,
    cols: u16,
) -> Vec<Line<'static>> {
    let screen = parser.screen();
    let mut lines = Vec::with_capacity(rows as usize);
    let (screen_rows, _screen_cols) = screen.size();

    for row_idx in 0..rows {
        if row_idx >= screen_rows {
            lines.push(Line::from(""));
            continue;
        }

        let mut spans = Vec::new();
        let mut current_text = String::new();
        let mut current_style = Style::default();
        let mut first = true;

        for col_idx in 0..cols {
            if col_idx >= _screen_cols {
                break;
            }

            let cell = screen.cell(row_idx, col_idx);
            match cell {
                Some(cell) => {
                    let fg = vt100_color_to_ratatui(cell.fgcolor());
                    let bg = vt100_color_to_ratatui(cell.bgcolor());
                    let mut style = Style::default();
                    if fg != Color::Reset {
                        style = style.fg(fg);
                    }
                    if bg != Color::Reset {
                        style = style.bg(bg);
                    }
                    if cell.bold() {
                        style = style.add_modifier(Modifier::BOLD);
                    }
                    if cell.italic() {
                        style = style.add_modifier(Modifier::ITALIC);
                    }
                    if cell.underline() {
                        style = style.add_modifier(Modifier::UNDERLINED);
                    }
                    if cell.inverse() {
                        style = style.add_modifier(Modifier::REVERSED);
                    }

                    if first || style != current_style {
                        if !first {
                            spans.push(Span::styled(
                                std::mem::take(&mut current_text),
                                current_style,
                            ));
                        }
                        current_style = style;
                        first = false;
                    }

                    let contents = cell.contents();
                    if contents.is_empty() {
                        current_text.push(' ');
                    } else {
                        current_text.push_str(&contents);
                    }
                }
                None => {
                    if first || current_style != Style::default() {
                        if !first {
                            spans.push(Span::styled(
                                std::mem::take(&mut current_text),
                                current_style,
                            ));
                        }
                        current_style = Style::default();
                        first = false;
                    }
                    current_text.push(' ');
                }
            }
        }

        if !current_text.is_empty() {
            spans.push(Span::styled(current_text, current_style));
        }

        lines.push(Line::from(spans));
    }

    lines
}

pub fn status_color(status: SessionStatus) -> Color {
    match status {
        SessionStatus::Running => Color::Yellow,
        SessionStatus::Waiting => Color::Green,
        SessionStatus::Done => Color::DarkGray,
        SessionStatus::Paused => Color::Blue,
    }
}

pub fn render_pane(
    buf: &mut Buffer,
    area: Rect,
    lines: &[Line<'static>],
    branch: &str,
    status: SessionStatus,
    is_focused: bool,
    pane_number: usize,
    is_entered: bool,
) {
    if area.width < 3 || area.height < 3 {
        return;
    }

    let border_color = if is_entered {
        Color::LightRed
    } else if is_focused {
        Color::White
    } else {
        status_color(status)
    };

    let border_style = Style::default().fg(border_color);
    let title_style = if is_focused {
        Style::default()
            .fg(border_color)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(border_color)
    };

    let status_badge = if is_entered {
        " INPUT "
    } else {
        status.label()
    };

    let title = format!(" [{}] {} ({}) ", pane_number + 1, branch, status_badge);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(Line::from(Span::styled(title, title_style)));

    let inner = block.inner(area);
    block.render(area, buf);

    // Render terminal content inside the pane
    for (i, line) in lines.iter().enumerate() {
        if i as u16 >= inner.height {
            break;
        }
        let _line_area = Rect::new(inner.x, inner.y + i as u16, inner.width, 1);
        buf.set_line(inner.x, inner.y + i as u16, line, inner.width);
    }
}
