#![allow(dead_code)]

mod app;
mod config;
mod keys;
mod session;
mod tiling;
mod ui;

use anyhow::Result;
use app::App;
use clap::Parser;
use config::Config;
use crossterm::event::{self, EnableBracketedPaste, DisableBracketedPaste, Event, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(
    name = "claude-dtm",
    about = "Claude-DTM: Dynamic Tiling Manager for Claude Code sessions"
)]
struct Cli {
    /// Path to the git repository
    #[arg(long, short)]
    repo: Option<String>,

    /// Base branch to fork worktrees from
    #[arg(long, short = 'b')]
    base_branch: Option<String>,

    /// Command to run in each session
    #[arg(long, short = 'c')]
    command: Option<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut config = Config::load().unwrap_or_default();

    if let Some(ref base) = cli.base_branch {
        config.default_base_branch = base.clone();
    }
    if let Some(ref cmd) = cli.command {
        config.default_command = cmd.clone();
    }

    let repo_path = config.resolve_repo_path(cli.repo.as_deref());

    // Verify it's a git repo
    if !repo_path.join(".git").exists() && !repo_path.join(".git").is_file() {
        let output = std::process::Command::new("git")
            .args(["rev-parse", "--git-dir"])
            .current_dir(&repo_path)
            .output();

        match output {
            Ok(o) if o.status.success() => {}
            _ => {
                eprintln!(
                    "Error: {} is not a git repository.\n\
                     Use --repo to specify a git repository path.",
                    repo_path.display()
                );
                std::process::exit(1);
            }
        }
    }

    let mut app = App::new(repo_path, config);
    let _ = app.load_sessions();

    // Setup terminal
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableBracketedPaste)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let result = run_event_loop(&mut terminal, &mut app);

    // Restore terminal
    terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), DisableBracketedPaste, LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(ref e) = result {
        eprintln!("Error: {:?}", e);
    }

    result
}

fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<()> {
    let tick_rate = Duration::from_millis(50);

    loop {
        let mut screen_cache: Vec<Vec<ratatui::text::Line<'static>>> = Vec::new();
        let term_size = terminal.size()?;

        let header_height = 2u16;
        let legend_height = 3u16;
        let statusbar_height = 1u16;
        let chrome_height = header_height + legend_height + statusbar_height;
        let pane_area_height = term_size.height.saturating_sub(chrome_height);
        let pane_area = ratatui::layout::Rect::new(0, header_height, term_size.width, pane_area_height);

        let rects = tiling::tile(
            pane_area,
            app.sessions.len(),
            app.focused,
            app.layout_mode,
            app.master_count,
            app.master_ratio,
        );

        for (i, session) in app.sessions.iter_mut().enumerate() {
            let rect = rects.get(i).copied().unwrap_or_default();
            let inner_rows = rect.height.saturating_sub(2);
            let inner_cols = rect.width.saturating_sub(2);

            if inner_rows > 0 && inner_cols > 0 {
                let _ = session.resize(inner_rows, inner_cols);
            }

            let parser = session.parser.lock().unwrap();
            let lines = ui::pane::render_terminal_screen(&parser, inner_rows, inner_cols);
            screen_cache.push(lines);
        }

        terminal.draw(|frame| {
            ui::render(frame, app, &screen_cache);
        })?;

        app.update_statuses();

        // Check if background task finished
        {
            let mut bg = app.bg_result.lock().unwrap();
            if let Some(msg) = bg.take() {
                app.show_dialog = Some(app::Dialog::Error(msg));
                app.input_mode = keys::InputMode::Dialog;
            }
        }

        if event::poll(tick_rate)? {
            match event::read()? {
                Event::Key(key) => {
                    if key.kind == KeyEventKind::Press {
                        if let Some(action) = keys::map_key(key, app.input_mode) {
                            app.handle_action(action)?;
                        }
                    }
                }
                Event::Paste(text) => {
                    match app.input_mode {
                        keys::InputMode::PaneFocused => {
                            if let Some(session) = app.sessions.get_mut(app.focused) {
                                let mut data = Vec::new();
                                data.extend_from_slice(b"\x1b[200~");
                                data.extend_from_slice(text.as_bytes());
                                data.extend_from_slice(b"\x1b[201~");
                                let _ = session.write_input(&data);
                            }
                        }
                        keys::InputMode::Dialog => {
                            // Insert pasted text into the active dialog input
                            for c in text.chars() {
                                if !c.is_control() {
                                    app.handle_action(keys::Action::DialogInput(c))?;
                                }
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
