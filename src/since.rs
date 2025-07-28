use crate::{GitXError, Result};
use std::process::Command;

pub fn run(reference: String) -> Result<String> {
    let output = Command::new("git")
        .args([
            "log",
            &format_git_log_range(&reference),
            "--pretty=format:- %h %s",
        ])
        .output()
        .map_err(|_| GitXError::GitCommand("Failed to run git log".to_string()))?;

    if !output.status.success() {
        return Err(GitXError::GitCommand(format!(
            "Failed to retrieve commits since '{reference}'"
        )));
    }

    let log = String::from_utf8_lossy(&output.stdout);
    if is_log_empty(&log) {
        Ok(format!("✅ No new commits since {reference}"))
    } else {
        Ok(format!("🔍 Commits since {reference}:\n{log}"))
    }
}

// Helper function to format git log range
pub fn format_git_log_range(reference: &str) -> String {
    format!("{reference}..HEAD")
}

// Helper function to check if log output is empty
pub fn is_log_empty(log_output: &str) -> bool {
    log_output.trim().is_empty()
}
