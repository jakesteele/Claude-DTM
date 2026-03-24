use anyhow::{Context, Result};
use portable_pty::{CommandBuilder, NativePtySystem, PtyPair, PtySize, PtySystem};
use std::io::Read;
use std::path::Path;

pub struct PtyHandle {
    pub pair: PtyPair,
    pub child: Box<dyn portable_pty::Child + Send + Sync>,
    pub reader: Option<Box<dyn Read + Send>>,
}

pub fn spawn_pty(
    worktree_path: &Path,
    command: &str,
    rows: u16,
    cols: u16,
) -> Result<PtyHandle> {
    let pty_system = NativePtySystem::default();
    let pair = pty_system
        .openpty(PtySize {
            rows: rows.max(2),
            cols: cols.max(2),
            pixel_width: 0,
            pixel_height: 0,
        })
        .context("Failed to open PTY")?;

    let parts: Vec<&str> = command.split_whitespace().collect();
    let (cmd, args) = if parts.is_empty() {
        ("claude", vec![])
    } else {
        (parts[0], parts[1..].to_vec())
    };

    let mut builder = CommandBuilder::new(cmd);
    for arg in args {
        builder.arg(arg);
    }
    builder.cwd(worktree_path);

    // Inherit environment
    for (key, value) in std::env::vars() {
        builder.env(key, value);
    }

    let reader = pair.master.try_clone_reader().context("Failed to clone PTY reader")?;
    let child = pair.slave.spawn_command(builder).context("Failed to spawn command in PTY")?;

    Ok(PtyHandle {
        pair,
        child,
        reader: Some(reader),
    })
}

pub fn resize_pty(pair: &PtyPair, rows: u16, cols: u16) -> Result<()> {
    pair.master
        .resize(PtySize {
            rows: rows.max(2),
            cols: cols.max(2),
            pixel_width: 0,
            pixel_height: 0,
        })
        .context("Failed to resize PTY")?;
    Ok(())
}

pub fn write_to_pty(pair: &PtyPair, data: &[u8]) -> Result<()> {
    use std::io::Write;
    let mut writer = pair.master.take_writer().context("Failed to get PTY writer")?;
    writer.write_all(data)?;
    writer.flush()?;
    Ok(())
}
