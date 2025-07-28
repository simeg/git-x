use crate::{GitXError, Result};
use std::path::Path;
use std::process::Command;

pub fn run() -> Result<String> {
    // Get repo name
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .map_err(|_| GitXError::GitCommand("Failed to get repo root".to_string()))?;

    if !output.status.success() {
        return Err(GitXError::GitCommand("Not in a git repository".to_string()));
    }

    let repo_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let repo_name = extract_repo_name(&repo_path);

    // Get current branch
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .map_err(|_| GitXError::GitCommand("Failed to get current branch".to_string()))?;

    if !output.status.success() {
        return Err(GitXError::GitCommand(
            "Failed to get current branch".to_string(),
        ));
    }

    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Get upstream tracking branch
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"])
        .output()
        .map_err(|_| GitXError::GitCommand(format!("Failed to get upstream for {branch}")))?;

    let tracking_raw = if output.status.success() {
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    } else {
        String::new()
    };
    let tracking = format_tracking_branch(&tracking_raw);

    // Get ahead/behind counts
    let output = Command::new("git")
        .args(["rev-list", "--left-right", "--count", "HEAD...@{u}"])
        .output()
        .map_err(|_| GitXError::GitCommand("Failed to get ahead/behind count".to_string()))?;

    let (ahead, behind) = if output.status.success() {
        let counts = String::from_utf8_lossy(&output.stdout).trim().to_string();
        parse_ahead_behind_counts(&counts)
    } else {
        ("0".to_string(), "0".to_string())
    };

    // Get last commit message and relative date
    let output = Command::new("git")
        .args(["log", "-1", "--pretty=format:%s (%cr)"])
        .output()
        .map_err(|_| GitXError::GitCommand("Failed to get last commit".to_string()))?;

    if !output.status.success() {
        return Err(GitXError::GitCommand(
            "Failed to get last commit".to_string(),
        ));
    }

    let last_commit = String::from_utf8_lossy(&output.stdout).trim().to_string();

    let bold = console::Style::new().bold();

    // Format all the info
    let result = format!(
        "Repo: {}\nBranch: {}\nTracking: {}\nAhead: {} Behind: {}\nLast Commit: \"{}\"",
        bold.apply_to(repo_name),
        bold.apply_to(branch),
        bold.apply_to(tracking),
        bold.apply_to(&ahead),
        bold.apply_to(&behind),
        bold.apply_to(last_commit)
    );

    Ok(result)
}

// Helper function to extract repo name from path
pub fn extract_repo_name(repo_path: &str) -> String {
    Path::new(repo_path)
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default()
}

// Helper function to parse ahead/behind counts
pub fn parse_ahead_behind_counts(counts_output: &str) -> (String, String) {
    let mut parts = counts_output.split_whitespace();
    let ahead = parts.next().unwrap_or("0").to_string();
    let behind = parts.next().unwrap_or("0").to_string();
    (ahead, behind)
}

// Helper function to format tracking branch
pub fn format_tracking_branch(tracking_raw: &str) -> String {
    if tracking_raw.is_empty() {
        "(no upstream)".to_string()
    } else {
        tracking_raw.to_string()
    }
}
