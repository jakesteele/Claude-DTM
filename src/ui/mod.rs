pub mod dialog;
pub mod pane;
pub mod statusbar;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use crate::app::App;
use crate::keys::InputMode;
use crate::tiling;

pub fn render(frame: &mut ratatui::Frame, app: &App, screen_cache: &[Vec<Line<'static>>]) {
    let area = frame.area();

    // Layout: header (2 lines) + panes area + legend (3 lines) + statusbar (1 line)
    let header_height = 2u16;
    let legend_height = 3u16;
    let statusbar_height = 1u16;
    let chrome_height = header_height + legend_height + statusbar_height;

    // Render header
    let header_area = Rect::new(area.x, area.y, area.width, header_height);
    render_header(frame.buffer_mut(), header_area);

    // Pane area
    let pane_area_height = area.height.saturating_sub(chrome_height);
    let pane_area = Rect::new(area.x, area.y + header_height, area.width, pane_area_height);

    // Legend area
    let legend_y = area.y + header_height + pane_area_height;
    let legend_area = Rect::new(area.x, legend_y, area.width, legend_height);
    render_legend(frame.buffer_mut(), legend_area, app.input_mode);

    // Statusbar at bottom
    let statusbar_area = Rect::new(
        area.x,
        area.y + area.height - statusbar_height,
        area.width,
        statusbar_height,
    );
    statusbar::render_statusbar(frame.buffer_mut(), statusbar_area, app);

    // Tile panes
    let n = app.sessions.len();
    if n == 0 {
        render_empty_state(frame.buffer_mut(), pane_area);
    } else {
        let rects = tiling::tile(
            pane_area,
            n,
            app.focused,
            app.layout_mode,
            app.master_count,
            app.master_ratio,
        );

        for (i, (session, rect)) in app.sessions.iter().zip(rects.iter()).enumerate() {
            if rect.width == 0 || rect.height == 0 {
                continue;
            }

            let is_focused = i == app.focused;
            let is_entered = is_focused && app.input_mode == InputMode::PaneFocused;

            let lines = if i < screen_cache.len() {
                &screen_cache[i]
            } else {
                continue;
            };

            pane::render_pane(
                frame.buffer_mut(),
                *rect,
                lines,
                &session.name,
                session.status,
                is_focused,
                i,
                is_entered,
            );
        }
    }

    // Render dialog on top if present (always, even with 0 sessions)
    if let Some(ref dialog) = app.show_dialog {
        dialog::render_dialog(frame.buffer_mut(), area, dialog, app);
    }
}

fn render_header(buf: &mut Buffer, area: Rect) {
    if area.height == 0 {
        return;
    }

    // Line 1: Logo / title bar
    // Futuristic gradient-like title
    let logo_spans = vec![
        Span::styled(
            " ◆ DWM",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "-CLAUDE",
            Style::default()
                .fg(Color::LightCyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " ◆ ",
            Style::default().fg(Color::Cyan),
        ),
        Span::styled(
            "Tiling Session Manager",
            Style::default().fg(Color::DarkGray),
        ),
    ];

    let line1 = Line::from(logo_spans);
    buf.set_line(area.x, area.y, &line1, area.width);

    // Line 2: separator
    if area.height > 1 {
        let sep = "─".repeat(area.width as usize);
        let line2 = Line::from(Span::styled(
            sep,
            Style::default().fg(Color::Rgb(40, 40, 60)),
        ));
        buf.set_line(area.x, area.y + 1, &line2, area.width);
    }
}

fn render_legend(buf: &mut Buffer, area: Rect, mode: InputMode) {
    if area.height == 0 {
        return;
    }

    // Separator line
    let sep = "─".repeat(area.width as usize);
    let sep_line = Line::from(Span::styled(
        sep,
        Style::default().fg(Color::Rgb(40, 40, 60)),
    ));
    buf.set_line(area.x, area.y, &sep_line, area.width);

    match mode {
        InputMode::Normal => {
            let row1 = Line::from(vec![
                Span::styled(" NAV ", Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(" j", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled("/", Style::default().fg(Color::DarkGray)),
                Span::styled("k", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(":focus ", Style::default().fg(Color::DarkGray)),
                Span::styled("J", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled("/", Style::default().fg(Color::DarkGray)),
                Span::styled("K", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(":swap ", Style::default().fg(Color::DarkGray)),
                Span::styled("⏎", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(":zoom ", Style::default().fg(Color::DarkGray)),
                Span::styled("h", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled("/", Style::default().fg(Color::DarkGray)),
                Span::styled("l", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(":resize ", Style::default().fg(Color::DarkGray)),
                Span::styled("1-9", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(":goto", Style::default().fg(Color::DarkGray)),
            ]);

            let row2 = Line::from(vec![
                Span::styled(" CMD ", Style::default().fg(Color::Black).bg(Color::Magenta).add_modifier(Modifier::BOLD)),
                Span::styled(" n", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled(":new ", Style::default().fg(Color::DarkGray)),
                Span::styled("f", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled(":enter ", Style::default().fg(Color::DarkGray)),
                Span::styled("s", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(":search ", Style::default().fg(Color::DarkGray)),
                Span::styled("z", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(":promote ", Style::default().fg(Color::DarkGray)),
                Span::styled("q", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::styled(":kill ", Style::default().fg(Color::DarkGray)),
                Span::styled("p", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
                Span::styled(":pause ", Style::default().fg(Color::DarkGray)),
                Span::styled("r", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
                Span::styled(":resume ", Style::default().fg(Color::DarkGray)),
                Span::styled(" LAYOUT ", Style::default().fg(Color::Black).bg(Color::Rgb(80, 80, 120)).add_modifier(Modifier::BOLD)),
                Span::styled(" t", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(":tile ", Style::default().fg(Color::DarkGray)),
                Span::styled("m", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(":mono ", Style::default().fg(Color::DarkGray)),
                Span::styled("g", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(":grid ", Style::default().fg(Color::DarkGray)),
                Span::styled("Q", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::styled(":quit", Style::default().fg(Color::DarkGray)),
            ]);

            if area.height > 1 {
                buf.set_line(area.x, area.y + 1, &row1, area.width);
            }
            if area.height > 2 {
                buf.set_line(area.x, area.y + 2, &row2, area.width);
            }
        }
        InputMode::PaneFocused => {
            let row = Line::from(vec![
                Span::styled(" INPUT MODE ", Style::default().fg(Color::Black).bg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::styled("  All keystrokes go to the pane.  ", Style::default().fg(Color::DarkGray)),
                Span::styled("Esc", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(": exit input mode", Style::default().fg(Color::DarkGray)),
            ]);
            if area.height > 1 {
                buf.set_line(area.x, area.y + 1, &row, area.width);
            }
        }
        InputMode::Dialog => {
            let row = Line::from(vec![
                Span::styled(" DIALOG ", Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled("  ", Style::default().fg(Color::DarkGray)),
                Span::styled("Enter", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(": confirm  ", Style::default().fg(Color::DarkGray)),
                Span::styled("Esc", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(": cancel", Style::default().fg(Color::DarkGray)),
            ]);
            if area.height > 1 {
                buf.set_line(area.x, area.y + 1, &row, area.width);
            }
        }
    }
}

fn render_empty_state(buf: &mut Buffer, area: Rect) {
    use ratatui::widgets::{Block, Borders, Paragraph, Widget};
    use ratatui::layout::Alignment;

    if area.height < 8 || area.width < 40 {
        return;
    }

    let box_w = 40u16;
    let box_h = 5u16;
    let x = area.x + (area.width.saturating_sub(box_w)) / 2;
    let y = area.y + (area.height.saturating_sub(box_h)) / 2;
    let box_area = Rect::new(x, y, box_w, box_h);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(60, 60, 100)));

    let inner = block.inner(box_area);
    block.render(box_area, buf);

    // Center text inside the block
    let line1 = Line::from(Span::styled(
        "No active sessions",
        Style::default().fg(Color::DarkGray),
    ));
    let line2 = Line::from(vec![
        Span::styled("Press ", Style::default().fg(Color::Rgb(60, 60, 100))),
        Span::styled("n", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::styled(" to create a new session", Style::default().fg(Color::DarkGray)),
    ]);

    let paragraph = Paragraph::new(vec![line1, line2])
        .alignment(Alignment::Center);
    paragraph.render(inner, buf);
}
