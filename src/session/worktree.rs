use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn create_worktree(
    repo_path: &Path,
    branch_name: &str,
    base_branch: &str,
) -> Result<PathBuf> {
    let worktree_dir = repo_path.join(".worktrees").join(branch_name.replace('/', "_"));

    // Ensure .worktrees directory exists
    std::fs::create_dir_all(repo_path.join(".worktrees"))?;

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
