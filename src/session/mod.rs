pub mod pty;
pub mod status;
pub mod worktree;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

use self::pty::PtyHandle;
use self::status::SessionStatus;

pub struct Session {
    pub id: String,
    pub branch: String,
    pub worktree_path: PathBuf,
    pub pty_handle: Option<PtyHandle>,
    pub parser: Arc<Mutex<vt100::Parser>>,
    pub status: SessionStatus,
    pub created_at: DateTime<Utc>,
    pub last_output: Arc<Mutex<Instant>>,
    pub reader_active: Arc<Mutex<bool>>,
    pub writer: Option<Box<dyn std::io::Write + Send>>,
}

impl Session {
    pub fn new(id: String, branch: String, worktree_path: PathBuf) -> Self {
        Self {
            id,
            branch,
            worktree_path,
            pty_handle: None,
            parser: Arc::new(Mutex::new(vt100::Parser::new(24, 80, 0))),
            status: SessionStatus::Paused,
            created_at: Utc::now(),
            last_output: Arc::new(Mutex::new(Instant::now())),
            reader_active: Arc::new(Mutex::new(false)),
            writer: None,
        }
    }

    pub fn spawn(&mut self, command: &str, rows: u16, cols: u16) -> Result<()> {
        let mut handle = pty::spawn_pty(&self.worktree_path, command, rows, cols)?;

        let reader = handle.reader.take();

        let writer = handle.pair.master.take_writer()?;
        self.writer = Some(writer);

        self.pty_handle = Some(handle);
        self.status = SessionStatus::Running;

        // Reset parser to correct size
        {
            let parser = self.parser.clone();
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let mut p = parser.lock().await;
                *p = vt100::Parser::new(rows, cols, 0);
            });
        }

        // Spawn async reader task
        if let Some(mut reader) = reader {
            let parser = self.parser.clone();
            let last_output = self.last_output.clone();
            let reader_active = self.reader_active.clone();

            {
                let rt = tokio::runtime::Handle::current();
                rt.block_on(async {
                    *reader_active.lock().await = true;
                });
            }

            tokio::spawn(async move {
                let mut buf = [0u8; 4096];
                loop {
                    match reader.read(&mut buf) {
                        Ok(0) => break,
                        Ok(n) => {
                            let mut p = parser.lock().await;
                            p.process(&buf[..n]);
                            *last_output.lock().await = Instant::now();
                        }
                        Err(_) => break,
                    }
                }
                *reader_active.lock().await = false;
            });
        }

        Ok(())
    }

    pub fn write_input(&mut self, data: &[u8]) -> Result<()> {
        if let Some(ref mut writer) = self.writer {
            use std::io::Write;
            writer.write_all(data)?;
            writer.flush()?;
        }
        Ok(())
    }

    pub fn resize(&mut self, rows: u16, cols: u16) -> Result<()> {
        if let Some(ref handle) = self.pty_handle {
            pty::resize_pty(&handle.pair, rows, cols)?;
            let parser = self.parser.clone();
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let mut p = parser.lock().await;
                p.set_size(rows, cols);
            });
        }
        Ok(())
    }

    pub fn is_alive(&self) -> bool {
        self.pty_handle.is_some()
    }

    pub fn kill(&mut self) -> Result<()> {
        if let Some(mut handle) = self.pty_handle.take() {
            handle.child.kill()?;
        }
        self.writer = None;
        self.status = SessionStatus::Done;
        Ok(())
    }
}

/// Serializable session info for persistence
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub branch: String,
    pub worktree_path: PathBuf,
    pub status: SessionStatus,
    pub created_at: DateTime<Utc>,
}

impl From<&Session> for SessionInfo {
    fn from(s: &Session) -> Self {
        SessionInfo {
            id: s.id.clone(),
            branch: s.branch.clone(),
            worktree_path: s.worktree_path.clone(),
            status: s.status,
            created_at: s.created_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionsFile {
    pub sessions: Vec<SessionInfo>,
}
