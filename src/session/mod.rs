pub mod pty;
pub mod status;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use self::pty::PtyHandle;
use self::status::SessionStatus;

pub struct Session {
    pub id: String,
    pub name: String,
    pub pty_handle: Option<PtyHandle>,
    pub parser: Arc<Mutex<vt100::Parser>>,
    pub status: SessionStatus,
    pub created_at: DateTime<Utc>,
    pub last_output: Arc<Mutex<Instant>>,
    pub reader_active: Arc<Mutex<bool>>,
    pub writer: Option<Box<dyn std::io::Write + Send>>,
}

impl Session {
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            pty_handle: None,
            parser: Arc::new(Mutex::new(vt100::Parser::new(24, 80, 0))),
            status: SessionStatus::Paused,
            created_at: Utc::now(),
            last_output: Arc::new(Mutex::new(Instant::now())),
            reader_active: Arc::new(Mutex::new(false)),
            writer: None,
        }
    }

    pub fn spawn(&mut self, command: &str, cwd: &std::path::Path, rows: u16, cols: u16) -> Result<()> {
        let mut handle = pty::spawn_pty(cwd, command, rows, cols)?;

        let reader = handle.reader.take();

        let writer = handle.pair.master.take_writer()?;
        self.writer = Some(writer);

        self.pty_handle = Some(handle);
        self.status = SessionStatus::Running;

        // Reset parser to correct size
        {
            let mut p = self.parser.lock().unwrap();
            *p = vt100::Parser::new(rows, cols, 0);
        }

        // Spawn reader thread (blocking I/O)
        if let Some(mut reader) = reader {
            let parser = self.parser.clone();
            let last_output = self.last_output.clone();
            let reader_active = self.reader_active.clone();

            *reader_active.lock().unwrap() = true;

            std::thread::spawn(move || {
                let mut buf = [0u8; 4096];
                loop {
                    match reader.read(&mut buf) {
                        Ok(0) => break,
                        Ok(n) => {
                            if let Ok(mut p) = parser.lock() {
                                p.process(&buf[..n]);
                            }
                            if let Ok(mut t) = last_output.lock() {
                                *t = Instant::now();
                            }
                        }
                        Err(_) => break,
                    }
                }
                if let Ok(mut active) = reader_active.lock() {
                    *active = false;
                }
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
            if let Ok(mut p) = self.parser.lock() {
                p.set_size(rows, cols);
            }
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
    pub name: String,
    pub status: SessionStatus,
    pub created_at: DateTime<Utc>,
}

impl From<&Session> for SessionInfo {
    fn from(s: &Session) -> Self {
        SessionInfo {
            id: s.id.clone(),
            name: s.name.clone(),
            status: s.status,
            created_at: s.created_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionsFile {
    pub sessions: Vec<SessionInfo>,
}
