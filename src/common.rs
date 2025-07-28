use crate::{GitXError, Result};
use console::style;
use std::process::Command;

/// Common utilities for git operations
pub struct GitCommand;

impl GitCommand {
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
}

/// Common formatting utilities
pub struct Format;

impl Format {
    /// Format an error message with emoji
    pub fn error(msg: &str) -> String {
        format!("{} {}", style("❌").bold(), msg)
    }

    /// Format a success message with emoji
    pub fn success(msg: &str) -> String {
        format!("{} {}", style("✅").bold(), msg)
    }

    /// Format an info message with emoji
    pub fn info(msg: &str) -> String {
        format!("{} {}", style("ℹ️").bold(), msg)
    }

    /// Format a warning message with emoji
    pub fn warning(msg: &str) -> String {
        format!("{} {}", style("⚠️").bold().yellow(), msg)
    }

    /// Format text with bold styling
    pub fn bold(text: &str) -> String {
        style(text).bold().to_string()
    }

    /// Format text with color
    pub fn colored(text: &str, color: console::Color) -> String {
        style(text).fg(color).to_string()
    }
}

/// Common validation utilities
pub struct Validate;

impl Validate {
    /// Validate that a commit exists
    pub fn commit_exists(commit: &str) -> Result<()> {
        if GitCommand::commit_exists(commit)? {
            Ok(())
        } else {
            Err(GitXError::GitCommand(format!(
                "Commit '{commit}' does not exist"
            )))
        }
    }

    /// Validate that we're in a git repository
    pub fn in_git_repo() -> Result<()> {
        match GitCommand::repo_root() {
            Ok(_) => Ok(()),
            Err(_) => Err(GitXError::GitCommand("Not in a git repository".to_string())),
        }
    }

    /// Validate branch name format
    pub fn branch_name(name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(GitXError::Parse("Branch name cannot be empty".to_string()));
        }

        // Check for invalid characters
        let invalid_chars = [' ', '~', '^', ':', '?', '*', '[', '\\'];
        if name.chars().any(|c| invalid_chars.contains(&c)) {
            return Err(GitXError::Parse(format!(
                "Branch name '{name}' contains invalid characters"
            )));
        }

        // Check for reserved names
        if name == "HEAD" || name.starts_with('-') {
            return Err(GitXError::Parse(format!(
                "Branch name '{name}' is reserved"
            )));
        }

        Ok(())
    }
}

/// Repository information utilities
pub struct RepoInfo;

impl RepoInfo {
    /// Get repository name from path
    pub fn name() -> Result<String> {
        let repo_path = GitCommand::repo_root()?;
        Ok(std::path::Path::new(&repo_path)
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default())
    }

    /// Get current branch with upstream info
    pub fn branch_status() -> Result<BranchStatus> {
        let current = GitCommand::current_branch()?;
        let upstream = GitCommand::upstream_branch().ok();
        let (ahead, behind) = GitCommand::ahead_behind_counts().unwrap_or((0, 0));

        Ok(BranchStatus {
            current,
            upstream,
            ahead,
            behind,
        })
    }
}

/// Branch status information
#[derive(Debug)]
pub struct BranchStatus {
    pub current: String,
    pub upstream: Option<String>,
    pub ahead: u32,
    pub behind: u32,
}

impl BranchStatus {
    /// Format upstream tracking info
    pub fn format_tracking(&self) -> String {
        self.upstream
            .as_ref()
            .map(|u| u.to_string())
            .unwrap_or_else(|| "(no upstream)".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_branch_name_valid() {
        assert!(Validate::branch_name("feature-branch").is_ok());
        assert!(Validate::branch_name("feature/branch").is_ok());
        assert!(Validate::branch_name("hotfix-123").is_ok());
    }

    #[test]
    fn test_validate_branch_name_invalid() {
        assert!(Validate::branch_name("").is_err());
        assert!(Validate::branch_name("branch with spaces").is_err());
        assert!(Validate::branch_name("HEAD").is_err());
        assert!(Validate::branch_name("-invalid").is_err());
        assert!(Validate::branch_name("branch~invalid").is_err());
    }

    #[test]
    fn test_format_error() {
        let formatted = Format::error("Test error");
        assert!(formatted.contains("❌"));
        assert!(formatted.contains("Test error"));
    }

    #[test]
    fn test_format_success() {
        let formatted = Format::success("Test success");
        assert!(formatted.contains("✅"));
        assert!(formatted.contains("Test success"));
    }
}
