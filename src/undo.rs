use crate::{GitXError, Result};
use std::process::Command;

pub fn run() -> Result<String> {
    let status = Command::new("git")
        .args(get_git_reset_args())
        .status()
        .map_err(|_| GitXError::GitCommand("Failed to execute git reset".to_string()))?;

    if status.success() {
        Ok(format_success_message().to_string())
    } else {
        Err(GitXError::GitCommand(format_error_message().to_string()))
    }
}

// Helper function to get git reset command args
pub fn get_git_reset_args() -> [&'static str; 3] {
    ["reset", "--soft", "HEAD~1"]
}

// Helper function to format success message
pub fn format_success_message() -> &'static str {
    "Last commit undone (soft reset). Changes kept in working directory."
}

// Helper function to format error message
pub fn format_error_message() -> &'static str {
    "❌ Failed to undo last commit."
}
