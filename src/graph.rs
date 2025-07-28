use crate::{GitXError, Result};
use std::process::Command;

pub fn run() -> Result<String> {
    let output = Command::new("git")
        .args(get_git_log_args())
        .output()
        .map_err(|_| GitXError::GitCommand("Failed to run git log".to_string()))?;

    if output.status.success() {
        let result = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(result)
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        Err(GitXError::GitCommand(format_git_error(&err)))
    }
}

// Helper function to get git log arguments
pub fn get_git_log_args() -> [&'static str; 5] {
    ["log", "--oneline", "--graph", "--decorate", "--all"]
}

// Helper function to format error message
pub fn format_git_error(stderr: &str) -> String {
    format!("❌ git log failed:\n{stderr}")
}
