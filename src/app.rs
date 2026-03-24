use anyhow::Result;
use std::path::PathBuf;
use std::time::Instant;

use crate::config::Config;
use crate::keys::{Action, InputMode};
use crate::session::status::SessionStatus;
use crate::session::{Session, SessionInfo, SessionsFile};
use crate::tiling::LayoutMode;

#[derive(Debug, Clone)]
pub enum Dialog {
    NewSession {
        name_input: String,
    },
    SearchSession {
        query: String,
        /// Index of the highlighted match in the filtered results
        selected: usize,
    },
    ConfirmKill {
        session_idx: usize,
        clean_worktree: bool,
    },
    ConfirmQuit,
    Error(String),
}

pub struct App {
    pub sessions: Vec<Session>,
    pub focused: usize,
    pub layout_mode: LayoutMode,
    pub master_ratio: f64,
    pub master_count: usize,
    pub repo_path: PathBuf,
    pub config: Config,
    pub show_dialog: Option<Dialog>,
    pub input_mode: InputMode,
    pub should_quit: bool,
    pub last_tick: Instant,
}

impl App {
    pub fn new(repo_path: PathBuf, config: Config) -> Self {
        let layout_mode = match config.default_layout.as_str() {
            "monocle" => LayoutMode::Monocle,
            "grid" => LayoutMode::Grid,
            _ => LayoutMode::MasterStack,
        };

        Self {
            sessions: Vec::new(),
            focused: 0,
            layout_mode,
            master_ratio: config.master_ratio,
            master_count: config.master_count,
            repo_path,
            config,
            show_dialog: None,
            input_mode: InputMode::Normal,
            should_quit: false,
            last_tick: Instant::now(),
        }
    }

    pub fn handle_action(&mut self, action: Action) -> Result<()> {
        match action {
            Action::FocusNext => {
                if !self.sessions.is_empty() {
                    self.focused = (self.focused + 1) % self.sessions.len();
                }
            }
            Action::FocusPrev => {
                if !self.sessions.is_empty() {
                    self.focused = if self.focused == 0 {
                        self.sessions.len() - 1
                    } else {
                        self.focused - 1
                    };
                }
            }
            Action::SwapNext => {
                let len = self.sessions.len();
                if len > 1 {
                    let next = (self.focused + 1) % len;
                    self.sessions.swap(self.focused, next);
                    self.focused = next;
                }
            }
            Action::SwapPrev => {
                let len = self.sessions.len();
                if len > 1 {
                    let prev = if self.focused == 0 { len - 1 } else { self.focused - 1 };
                    self.sessions.swap(self.focused, prev);
                    self.focused = prev;
                }
            }
            Action::ZoomMaster => {
                if !self.sessions.is_empty() && self.focused != 0 {
                    self.sessions.swap(0, self.focused);
                    self.focused = 0;
                }
            }
            Action::ShrinkMaster => {
                self.master_ratio = (self.master_ratio - 0.05).max(0.1);
            }
            Action::GrowMaster => {
                self.master_ratio = (self.master_ratio + 0.05).min(0.9);
            }
            Action::IncMasterCount => {
                self.master_count += 1;
            }
            Action::DecMasterCount => {
                if self.master_count > 1 {
                    self.master_count -= 1;
                }
            }
            Action::LayoutMasterStack => {
                self.layout_mode = LayoutMode::MasterStack;
            }
            Action::LayoutMonocle => {
                self.layout_mode = LayoutMode::Monocle;
            }
            Action::LayoutGrid => {
                self.layout_mode = LayoutMode::Grid;
            }
            Action::NewSession => {
                self.show_dialog = Some(Dialog::NewSession {
                    name_input: String::new(),
                });
                self.input_mode = InputMode::Dialog;
            }
            Action::KillSession => {
                if !self.sessions.is_empty() {
                    self.show_dialog = Some(Dialog::ConfirmKill {
                        session_idx: self.focused,
                        clean_worktree: true,
                    });
                    self.input_mode = InputMode::Dialog;
                }
            }
            Action::PauseSession => {
                if let Some(session) = self.sessions.get_mut(self.focused) {
                    if session.status == SessionStatus::Running
                        || session.status == SessionStatus::Waiting
                    {
                        let _ = session.kill();
                        session.status = SessionStatus::Paused;
                    }
                }
            }
            Action::ResumeSession => {
                if let Some(session) = self.sessions.get_mut(self.focused) {
                    if session.status == SessionStatus::Paused {
                        let cmd = format!("{} --worktree {}", self.config.default_command, session.name);
                        if let Err(e) = session.spawn(&cmd, &self.repo_path, 24, 80) {
                            self.show_dialog = Some(Dialog::Error(format!("Resume failed: {}", e)));
                            self.input_mode = InputMode::Dialog;
                        }
                    }
                }
            }
            Action::EnterPane => {
                if !self.sessions.is_empty() {
                    let session = &self.sessions[self.focused];
                    if session.status != SessionStatus::Done
                        && session.status != SessionStatus::Paused
                    {
                        self.input_mode = InputMode::PaneFocused;
                    }
                }
            }
            Action::ExitPane => {
                self.input_mode = InputMode::Normal;
                self.show_dialog = None;
            }
            Action::SearchSession => {
                if !self.sessions.is_empty() {
                    self.show_dialog = Some(Dialog::SearchSession {
                        query: String::new(),
                        selected: 0,
                    });
                    self.input_mode = InputMode::Dialog;
                }
            }
            Action::ToggleZoom => {
                if self.sessions.len() > 1 {
                    if self.focused != 0 {
                        self.sessions.swap(0, self.focused);
                        self.focused = 0;
                    }
                    self.layout_mode = LayoutMode::MasterStack;
                }
            }
            Action::FocusPane(idx) => {
                if idx < self.sessions.len() {
                    self.focused = idx;
                }
            }
            Action::CleanupWorktrees => {
                match self.cleanup_orphaned_worktrees() {
                    Ok(0) => {
                        self.show_dialog = Some(Dialog::Error("No orphaned worktrees found.".to_string()));
                        self.input_mode = InputMode::Dialog;
                    }
                    Ok(n) => {
                        self.show_dialog = Some(Dialog::Error(format!("Cleaned up {} orphaned worktree(s).", n)));
                        self.input_mode = InputMode::Dialog;
                    }
                    Err(e) => {
                        self.show_dialog = Some(Dialog::Error(format!("Cleanup failed: {}", e)));
                        self.input_mode = InputMode::Dialog;
                    }
                }
            }
            Action::Quit => {
                let has_active = self.sessions.iter().any(|s| {
                    s.status == SessionStatus::Running || s.status == SessionStatus::Waiting
                });
                if has_active {
                    self.show_dialog = Some(Dialog::ConfirmQuit);
                    self.input_mode = InputMode::Dialog;
                } else {
                    self.should_quit = true;
                }
            }
            Action::DialogConfirm => {
                self.handle_dialog_confirm()?;
            }
            Action::DialogCancel => {
                self.show_dialog = None;
                self.input_mode = InputMode::Normal;
            }
            Action::DialogInput(c) => {
                self.handle_dialog_input(c);
            }
            Action::DialogBackspace => {
                self.handle_dialog_backspace();
            }
            Action::DialogUp => {
                self.handle_dialog_up();
            }
            Action::DialogDown => {
                self.handle_dialog_down();
            }
            Action::PassThrough(key) => {
                self.handle_passthrough(key)?;
            }
        }
        Ok(())
    }

    fn handle_dialog_confirm(&mut self) -> Result<()> {
        let dialog = self.show_dialog.take();
        self.input_mode = InputMode::Normal;

        match dialog {
            Some(Dialog::NewSession { name_input }) => {
                self.create_session(&name_input)?;
            }
            Some(Dialog::SearchSession { query, selected }) => {
                let matches: Vec<usize> = self
                    .sessions
                    .iter()
                    .enumerate()
                    .filter(|(_, s)| {
                        if query.is_empty() {
                            true
                        } else {
                            let q = query.to_lowercase();
                            s.name.to_lowercase().contains(&q)
                        }
                    })
                    .map(|(i, _)| i)
                    .collect();

                if let Some(&idx) = matches.get(selected) {
                    self.focused = idx;
                }
            }
            Some(Dialog::ConfirmKill { session_idx, clean_worktree }) => {
                self.kill_session(session_idx, clean_worktree)?;
            }
            Some(Dialog::ConfirmQuit) => {
                self.shutdown()?;
                self.should_quit = true;
            }
            Some(Dialog::Error(_)) => {}
            None => {}
        }
        Ok(())
    }

    fn handle_dialog_input(&mut self, c: char) {
        if let Some(Dialog::ConfirmKill {
            ref mut clean_worktree, ..
        }) = self.show_dialog
        {
            if c == 'w' || c == 'W' {
                *clean_worktree = !*clean_worktree;
            }
            return;
        }
        if let Some(Dialog::NewSession {
            ref mut name_input,
        }) = self.show_dialog
        {
            name_input.push(c);
        } else if let Some(Dialog::SearchSession {
            ref mut query,
            ref mut selected,
        }) = self.show_dialog
        {
            query.push(c);
            *selected = 0;
        }
    }

    fn handle_dialog_backspace(&mut self) {
        if let Some(Dialog::NewSession {
            ref mut name_input,
        }) = self.show_dialog
        {
            name_input.pop();
        } else if let Some(Dialog::SearchSession {
            ref mut query,
            ref mut selected,
        }) = self.show_dialog
        {
            query.pop();
            *selected = 0;
        }
    }

    fn handle_dialog_up(&mut self) {
        if let Some(Dialog::SearchSession {
            ref mut selected, ..
        }) = self.show_dialog
        {
            *selected = selected.saturating_sub(1);
        }
    }

    fn handle_dialog_down(&mut self) {
        if let Some(Dialog::SearchSession {
            ref query,
            ref mut selected,
        }) = self.show_dialog
        {
            let match_count = self
                .sessions
                .iter()
                .filter(|s| {
                    if query.is_empty() {
                        true
                    } else {
                        let q = query.to_lowercase();
                        s.name.to_lowercase().contains(&q)
                    }
                })
                .count();

            if match_count > 0 {
                *selected = (*selected + 1).min(match_count - 1);
            }
        }
    }

    fn handle_passthrough(&mut self, key: crossterm::event::KeyEvent) -> Result<()> {
        use crossterm::event::KeyCode;

        if let Some(session) = self.sessions.get_mut(self.focused) {
            let data = match key.code {
                KeyCode::Char(c) => {
                    let mut buf = [0u8; 4];
                    let s = c.encode_utf8(&mut buf);
                    s.as_bytes().to_vec()
                }
                KeyCode::Enter => vec![b'\r'],
                KeyCode::Backspace => vec![0x7f],
                KeyCode::Tab => vec![b'\t'],
                KeyCode::Left => vec![0x1b, b'[', b'D'],
                KeyCode::Right => vec![0x1b, b'[', b'C'],
                KeyCode::Up => vec![0x1b, b'[', b'A'],
                KeyCode::Down => vec![0x1b, b'[', b'B'],
                KeyCode::Delete => vec![0x1b, b'[', b'3', b'~'],
                KeyCode::Home => vec![0x1b, b'[', b'H'],
                KeyCode::End => vec![0x1b, b'[', b'F'],
                _ => return Ok(()),
            };
            session.write_input(&data)?;
        }
        Ok(())
    }

    pub fn create_session(&mut self, name: &str) -> Result<()> {
        let display_name = if name.trim().is_empty() {
            format!("session-{}", self.sessions.len() + 1)
        } else {
            name.to_string()
        };

        let id = uuid::Uuid::new_v4().to_string();
        let mut session = Session::new(id, display_name.clone());

        // Build command: claude --worktree <name>
        let cmd = format!("{} --worktree {}", self.config.default_command, display_name);
        if let Err(e) = session.spawn(&cmd, &self.repo_path, 24, 80) {
            self.show_dialog = Some(Dialog::Error(format!("Failed to spawn session: {}", e)));
            self.input_mode = InputMode::Dialog;
            return Ok(());
        }

        self.sessions.push(session);
        self.focused = self.sessions.len() - 1;
        Ok(())
    }

    pub fn kill_session(&mut self, idx: usize, clean_worktree: bool) -> Result<()> {
        if idx >= self.sessions.len() {
            return Ok(());
        }

        let session = self.sessions.remove(idx);
        let session_name = session.name.clone();
        let mut session = session;
        let _ = session.kill();

        if clean_worktree {
            self.cleanup_worktree(&session_name);
        }

        if !self.sessions.is_empty() {
            self.focused = self.focused.min(self.sessions.len() - 1);
        } else {
            self.focused = 0;
        }

        Ok(())
    }

    /// Remove a worktree from disk via `git worktree remove`
    fn cleanup_worktree(&self, name: &str) {
        let worktree_path = self.repo_path.join(".claude").join("worktrees").join(name);
        if worktree_path.exists() {
            // Try git worktree remove first (proper cleanup)
            let result = std::process::Command::new("git")
                .args(["worktree", "remove", "--force"])
                .arg(&worktree_path)
                .current_dir(&self.repo_path)
                .output();

            if result.is_err() || !result.as_ref().unwrap().status.success() {
                // Fallback: just remove the directory
                let _ = std::fs::remove_dir_all(&worktree_path);
                // Also prune stale worktree references
                let _ = std::process::Command::new("git")
                    .args(["worktree", "prune"])
                    .current_dir(&self.repo_path)
                    .output();
            }
        }
    }

    /// Clean up all orphaned worktrees (exist on disk but have no active session)
    pub fn cleanup_orphaned_worktrees(&self) -> Result<usize> {
        let worktrees_dir = self.repo_path.join(".claude").join("worktrees");
        if !worktrees_dir.exists() {
            return Ok(0);
        }

        let active_names: std::collections::HashSet<String> =
            self.sessions.iter().map(|s| s.name.clone()).collect();

        let mut cleaned = 0;
        if let Ok(entries) = std::fs::read_dir(&worktrees_dir) {
            for entry in entries.flatten() {
                let dir_name = entry.file_name().to_string_lossy().to_string();
                if !active_names.contains(&dir_name) && entry.path().is_dir() {
                    self.cleanup_worktree(&dir_name);
                    cleaned += 1;
                }
            }
        }

        Ok(cleaned)
    }

    pub fn shutdown(&mut self) -> Result<()> {
        self.save_sessions()?;

        for session in &mut self.sessions {
            if session.status != SessionStatus::Done {
                let _ = session.kill();
                session.status = SessionStatus::Paused;
            }
        }

        Ok(())
    }

    pub fn save_sessions(&self) -> Result<()> {
        let infos: Vec<SessionInfo> = self.sessions.iter().map(SessionInfo::from).collect();
        let file = SessionsFile { sessions: infos };
        let dir = Config::config_dir();
        std::fs::create_dir_all(&dir)?;
        let json = serde_json::to_string_pretty(&file)?;
        std::fs::write(Config::sessions_path(), json)?;
        Ok(())
    }

    pub fn load_sessions(&mut self) -> Result<()> {
        let path = Config::sessions_path();
        if !path.exists() {
            return Ok(());
        }
        let contents = std::fs::read_to_string(&path)?;
        let file: SessionsFile = serde_json::from_str(&contents)?;

        for info in file.sessions {
            let session = Session::new(info.id, info.name);
            self.sessions.push(session);
        }
        Ok(())
    }

    pub fn update_statuses(&mut self) {
        for session in &mut self.sessions {
            if session.status == SessionStatus::Paused || session.status == SessionStatus::Done {
                continue;
            }

            let reader_active = *session.reader_active.lock().unwrap();
            let last_output = *session.last_output.lock().unwrap();
            let parser = session.parser.lock().unwrap();

            if !reader_active && session.pty_handle.is_some() {
                session.status = SessionStatus::Done;
            } else {
                let detected =
                    crate::session::status::detect_status(&parser, last_output, reader_active);
                session.status = detected;
            }
        }
    }
}
