use crate::{GitXError, Result};
use std::process::Command;

/// Core git operations abstraction
pub struct GitOperations;

impl GitOperations {
    /// Execute a git command and return stdout as String
    pub fn run(args: &[&str]) -> Result<String> {
        let output = Command::new("git").args(args).output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            let stderr_output = String::from_utf8_lossy(&output.stderr);
            let stderr = stderr_output.trim();
            Err(GitXError::GitCommand(stderr.to_string()))
        }
    }

    /// Execute a git command and return success status
    pub fn run_status(args: &[&str]) -> Result<()> {
        let status = Command::new("git").args(args).status()?;

        if status.success() {
            Ok(())
        } else {
            Err(GitXError::GitCommand(format!(
                "Git command failed: git {}",
                args.join(" ")
            )))
        }
    }

    /// Get current branch name
    pub fn current_branch() -> Result<String> {
        Self::run(&["rev-parse", "--abbrev-ref", "HEAD"])
    }

    /// Get repository root path
    pub fn repo_root() -> Result<String> {
        Self::run(&["rev-parse", "--show-toplevel"])
    }

    /// Check if a commit exists
    pub fn commit_exists(commit: &str) -> Result<bool> {
        match Self::run(&["rev-parse", "--verify", &format!("{commit}^{{commit}}")]) {
            Ok(_) => Ok(true),
            Err(GitXError::GitCommand(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// Get short commit hash
    pub fn short_hash(commit: &str) -> Result<String> {
        Self::run(&["rev-parse", "--short", commit])
    }

    /// Get upstream branch for current branch
    pub fn upstream_branch() -> Result<String> {
        Self::run(&["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"])
    }

    /// Get ahead/behind counts compared to upstream
    pub fn ahead_behind_counts() -> Result<(u32, u32)> {
        let output = Self::run(&["rev-list", "--left-right", "--count", "HEAD...@{u}"])?;
        let mut parts = output.split_whitespace();
        let ahead = parts.next().unwrap_or("0").parse().unwrap_or(0);
        let behind = parts.next().unwrap_or("0").parse().unwrap_or(0);
        Ok((ahead, behind))
    }

    /// Get branch information in an optimized way to reduce git calls
    pub fn branch_info_optimized() -> Result<(String, Option<String>, u32, u32)> {
        // Get current branch
        let current = Self::current_branch()?;

        // Try to get upstream - if this fails, there's no upstream
        match Self::upstream_branch() {
            Ok(upstream) => {
                // Only check ahead/behind if upstream exists
                let (ahead, behind) = Self::ahead_behind_counts().unwrap_or((0, 0));
                Ok((current, Some(upstream), ahead, behind))
            }
            Err(_) => {
                // No upstream configured
                Ok((current, None, 0, 0))
            }
        }
    }

    /// Get all local branches
    pub fn local_branches() -> Result<Vec<String>> {
        let output = Self::run(&["branch", "--format=%(refname:short)"])?;
        let branches: Vec<String> = output
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|branch| !branch.is_empty())
            .collect();
        Ok(branches)
    }

    /// Get recent branches sorted by commit date
    pub fn recent_branches(limit: Option<usize>) -> Result<Vec<String>> {
        let output = Self::run(&[
            "for-each-ref",
            "--sort=-committerdate",
            "--format=%(refname:short)",
            "refs/heads/",
        ])?;

        let current_branch = Self::current_branch().unwrap_or_default();
        let mut branches: Vec<String> = output
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|branch| !branch.is_empty() && branch != &current_branch)
            .collect();

        if let Some(limit) = limit {
            branches.truncate(limit);
        }

        Ok(branches)
    }

    /// Get merged branches
    pub fn merged_branches() -> Result<Vec<String>> {
        let output = Self::run(&["branch", "--merged"])?;
        let branches: Vec<String> = output
            .lines()
            .map(|line| line.trim().trim_start_matches("* ").to_string())
            .filter(|branch| !branch.is_empty())
            .collect();
        Ok(branches)
    }

    /// Check if working directory is clean
    pub fn is_working_directory_clean() -> Result<bool> {
        let output = Self::run(&["status", "--porcelain"])?;
        Ok(output.trim().is_empty())
    }

    /// Get staged files
    pub fn staged_files() -> Result<Vec<String>> {
        let output = Self::run(&["diff", "--cached", "--name-only"])?;
        let files: Vec<String> = output
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|file| !file.is_empty())
            .collect();
        Ok(files)
    }
}

/// Branch operations
pub struct BranchOperations;

impl BranchOperations {
    /// Create a new branch
    pub fn create(name: &str, from: Option<&str>) -> Result<()> {
        let mut args = vec!["checkout", "-b", name];
        if let Some(base) = from {
            args.push(base);
        }
        GitOperations::run_status(&args)
    }

    /// Delete a branch
    pub fn delete(name: &str, force: bool) -> Result<()> {
        let flag = if force { "-D" } else { "-d" };
        GitOperations::run_status(&["branch", flag, name])
    }

    /// Rename current branch
    pub fn rename(new_name: &str) -> Result<()> {
        GitOperations::run_status(&["branch", "-m", new_name])
    }

    /// Switch to a branch
    pub fn switch(name: &str) -> Result<()> {
        GitOperations::run_status(&["checkout", name])
    }

    /// Check if branch exists
    pub fn exists(name: &str) -> Result<bool> {
        match GitOperations::run(&["rev-parse", "--verify", &format!("refs/heads/{name}")]) {
            Ok(_) => Ok(true),
            Err(GitXError::GitCommand(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }
}

/// Commit operations
pub struct CommitOperations;

impl CommitOperations {
    /// Create a fixup commit
    pub fn fixup(commit_hash: &str) -> Result<()> {
        GitOperations::run_status(&["commit", "--fixup", commit_hash])
    }

    /// Undo last commit (soft reset)
    pub fn undo_last() -> Result<()> {
        GitOperations::run_status(&["reset", "--soft", "HEAD~1"])
    }

    /// Get commit message
    pub fn get_message(commit_hash: &str) -> Result<String> {
        GitOperations::run(&["log", "-1", "--pretty=format:%s", commit_hash])
    }

    /// Get commit author
    pub fn get_author(commit_hash: &str) -> Result<String> {
        GitOperations::run(&["log", "-1", "--pretty=format:%an <%ae>", commit_hash])
    }
}

/// Remote operations
pub struct RemoteOperations;

impl RemoteOperations {
    /// Set upstream for current branch
    pub fn set_upstream(remote: &str, branch: &str) -> Result<()> {
        GitOperations::run_status(&["branch", "--set-upstream-to", &format!("{remote}/{branch}")])
    }

    /// Push to remote
    pub fn push(remote: Option<&str>, branch: Option<&str>) -> Result<()> {
        let mut args = vec!["push"];
        if let Some(r) = remote {
            args.push(r);
        }
        if let Some(b) = branch {
            args.push(b);
        }
        GitOperations::run_status(&args)
    }

    /// Fetch from remote
    pub fn fetch(remote: Option<&str>) -> Result<()> {
        let mut args = vec!["fetch"];
        if let Some(r) = remote {
            args.push(r);
        }
        GitOperations::run_status(&args)
    }

    /// Get remotes
    pub fn list() -> Result<Vec<String>> {
        let output = GitOperations::run(&["remote"])?;
        let remotes: Vec<String> = output
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|remote| !remote.is_empty())
            .collect();
        Ok(remotes)
    }
}
