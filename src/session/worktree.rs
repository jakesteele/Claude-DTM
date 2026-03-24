use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Get the base directory for all worktrees: ~/claude-dtm-worktrees/<repo-name>/
fn worktrees_base_dir(repo_path: &Path) -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let repo_name = repo_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "repo".to_string());
    home.join("claude-dtm-worktrees").join(repo_name)
}

/// Sanitize a session name for use as a directory name
fn sanitize_dir_name(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ' ' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c => c,
        })
        .collect()
}

pub fn create_worktree(
    repo_path: &Path,
    branch_name: &str,
    base_branch: &str,
    session_name: &str,
) -> Result<PathBuf> {
    let base_dir = worktrees_base_dir(repo_path);
    std::fs::create_dir_all(&base_dir)?;

    // Use session name for the folder, fall back to branch if empty
    let dir_name = if session_name.trim().is_empty() {
        sanitize_dir_name(branch_name)
    } else {
        sanitize_dir_name(session_name)
    };
    let worktree_dir = base_dir.join(&dir_name);

    let output = Command::new("git")
        .args(["worktree", "add", "-b", branch_name])
        .arg(&worktree_dir)
        .arg(base_branch)
        .current_dir(repo_path)
        .output()
        .context("Failed to run git worktree add")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // If branch already exists, try without -b
        if stderr.contains("already exists") {
            let output2 = Command::new("git")
                .args(["worktree", "add"])
                .arg(&worktree_dir)
                .arg(branch_name)
                .current_dir(repo_path)
                .output()
                .context("Failed to run git worktree add (existing branch)")?;

            if !output2.status.success() {
                let stderr2 = String::from_utf8_lossy(&output2.stderr);
                bail!("git worktree add failed: {}", stderr2.trim());
            }
        } else {
            bail!("git worktree add failed: {}", stderr.trim());
        }
    }

    Ok(worktree_dir)
}

pub fn remove_worktree(repo_path: &Path, worktree_path: &Path) -> Result<()> {
    let output = Command::new("git")
        .args(["worktree", "remove", "--force"])
        .arg(worktree_path)
        .current_dir(repo_path)
        .output()
        .context("Failed to run git worktree remove")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git worktree remove failed: {}", stderr.trim());
    }

    Ok(())
}

pub fn delete_branch(repo_path: &Path, branch_name: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["branch", "-D", branch_name])
        .current_dir(repo_path)
        .output()
        .context("Failed to run git branch -D")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git branch -D failed: {}", stderr.trim());
    }

    Ok(())
}

pub fn list_worktrees(repo_path: &Path) -> Result<Vec<(String, PathBuf)>> {
    let output = Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir(repo_path)
        .output()
        .context("Failed to run git worktree list")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut result = Vec::new();
    let mut current_path: Option<PathBuf> = None;

    for line in stdout.lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            current_path = Some(PathBuf::from(path));
        } else if let Some(branch) = line.strip_prefix("branch refs/heads/") {
            if let Some(path) = current_path.take() {
                result.push((branch.to_string(), path));
            }
        } else if line.is_empty() {
            current_path = None;
        }
    }

    Ok(result)
}
