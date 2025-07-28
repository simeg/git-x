use crate::{GitXError, Result};
use std::process::Command;

pub fn run(merge: bool) -> Result<String> {
    // Get current branch
    let current_branch = get_current_branch_result()?;

    // Get upstream branch
    let upstream = get_upstream_branch_result(&current_branch)?;

    let mut output = Vec::new();
    output.push(format_sync_start_message(&current_branch, &upstream));

    // Fetch from remote
    fetch_upstream_result(&upstream)?;

    // Check if we're ahead of upstream
    let status = get_sync_status_result(&current_branch, &upstream)?;

    match status {
        SyncStatus::UpToDate => {
            output.push(format_up_to_date_message().to_string());
        }
        SyncStatus::Behind(count) => {
            output.push(format_behind_message(count));
            sync_with_upstream_result(&upstream, merge)?;
            output.push(format_sync_success_message(merge));
        }
        SyncStatus::Ahead(count) => {
            output.push(format_ahead_message(count));
        }
        SyncStatus::Diverged(behind, ahead) => {
            output.push(format_diverged_message(behind, ahead));
            if merge {
                sync_with_upstream_result(&upstream, merge)?;
                output.push(format_sync_success_message(merge));
            } else {
                output.push(format_diverged_help_message().to_string());
            }
        }
    }
    Ok(output.join("\n"))
}

#[derive(Debug, PartialEq)]
pub enum SyncStatus {
    UpToDate,
    Behind(u32),
    Ahead(u32),
    Diverged(u32, u32), // behind, ahead
}

// Helper function to get current branch (new version)
pub fn get_current_branch_result() -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .map_err(|_| GitXError::GitCommand("Failed to get current branch".to_string()))?;

    if !output.status.success() {
        return Err(GitXError::GitCommand("Not in a git repository".to_string()));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

// Helper function to get upstream branch (new version)
pub fn get_upstream_branch_result(branch: &str) -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", &format!("{branch}@{{u}}")])
        .output()
        .map_err(|_| GitXError::GitCommand("Failed to get upstream branch".to_string()))?;

    if !output.status.success() {
        return Err(GitXError::GitCommand(
            "No upstream branch configured".to_string(),
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

// Helper function to fetch from upstream (new version)
pub fn fetch_upstream_result(upstream: &str) -> Result<()> {
    let remote = upstream.split('/').next().unwrap_or("origin");

    let status = Command::new("git")
        .args(["fetch", remote])
        .status()
        .map_err(|_| GitXError::GitCommand("Failed to execute fetch command".to_string()))?;

    if !status.success() {
        return Err(GitXError::GitCommand(
            "Failed to fetch from remote".to_string(),
        ));
    }

    Ok(())
}

// Helper function to get sync status (new version)
pub fn get_sync_status_result(branch: &str, upstream: &str) -> Result<SyncStatus> {
    let output = Command::new("git")
        .args([
            "rev-list",
            "--left-right",
            "--count",
            &format!("{upstream}...{branch}"),
        ])
        .output()
        .map_err(|_| GitXError::GitCommand("Failed to get sync status".to_string()))?;

    if !output.status.success() {
        return Err(GitXError::GitCommand(
            "Failed to compare with upstream".to_string(),
        ));
    }

    let counts = String::from_utf8_lossy(&output.stdout);
    let (behind, ahead) = parse_sync_counts_result(&counts)?;

    Ok(match (behind, ahead) {
        (0, 0) => SyncStatus::UpToDate,
        (b, 0) if b > 0 => SyncStatus::Behind(b),
        (0, a) if a > 0 => SyncStatus::Ahead(a),
        (b, a) if b > 0 && a > 0 => SyncStatus::Diverged(b, a),
        _ => SyncStatus::UpToDate,
    })
}

// Helper function to parse sync counts (new version)
pub fn parse_sync_counts_result(output: &str) -> Result<(u32, u32)> {
    let mut parts = output.split_whitespace();
    let behind = parts
        .next()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| GitXError::GitCommand("Invalid sync count format".to_string()))?;
    let ahead = parts
        .next()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| GitXError::GitCommand("Invalid sync count format".to_string()))?;

    Ok((behind, ahead))
}

// Helper function for backward compatibility with tests
pub fn parse_sync_counts(output: &str) -> std::result::Result<(u32, u32), &'static str> {
    let mut parts = output.split_whitespace();
    let behind = parts
        .next()
        .and_then(|s| s.parse().ok())
        .ok_or("Invalid sync count format")?;
    let ahead = parts
        .next()
        .and_then(|s| s.parse().ok())
        .ok_or("Invalid sync count format")?;

    Ok((behind, ahead))
}

// Helper function to sync with upstream (new version)
pub fn sync_with_upstream_result(upstream: &str, merge: bool) -> Result<()> {
    let args = if merge {
        ["merge", upstream]
    } else {
        ["rebase", upstream]
    };

    let status = Command::new("git")
        .args(args)
        .status()
        .map_err(|_| GitXError::GitCommand("Failed to execute sync command".to_string()))?;

    if !status.success() {
        return Err(GitXError::GitCommand(if merge {
            "Merge failed".to_string()
        } else {
            "Rebase failed".to_string()
        }));
    }

    Ok(())
}

// Helper function to format sync start message
pub fn format_sync_start_message(branch: &str, upstream: &str) -> String {
    format!("🔄 Syncing branch '{branch}' with '{upstream}'...")
}

// Helper function to format error message
pub fn format_error_message(msg: &str) -> String {
    format!("❌ {msg}")
}

// Helper function to format up to date message
pub fn format_up_to_date_message() -> &'static str {
    "✅ Branch is up to date with upstream"
}

// Helper function to format behind message
pub fn format_behind_message(count: u32) -> String {
    format!("⬇️ Branch is {count} commit(s) behind upstream")
}

// Helper function to format ahead message
pub fn format_ahead_message(count: u32) -> String {
    format!("⬆️ Branch is {count} commit(s) ahead of upstream")
}

// Helper function to format diverged message
pub fn format_diverged_message(behind: u32, ahead: u32) -> String {
    format!("🔀 Branch has diverged: {behind} behind, {ahead} ahead")
}

// Helper function to format diverged help message
pub fn format_diverged_help_message() -> &'static str {
    "💡 Use --merge flag to merge changes, or handle manually"
}

// Helper function to format sync success message
pub fn format_sync_success_message(merge: bool) -> String {
    if merge {
        "✅ Successfully merged upstream changes".to_string()
    } else {
        "✅ Successfully rebased onto upstream".to_string()
    }
}

// Backward compatibility functions for tests
pub fn fetch_upstream(upstream: &str) -> std::result::Result<(), &'static str> {
    match fetch_upstream_result(upstream) {
        Ok(()) => Ok(()),
        Err(_) => Err("Failed to fetch from remote"),
    }
}

pub fn get_sync_status(
    branch: &str,
    upstream: &str,
) -> std::result::Result<SyncStatus, &'static str> {
    match get_sync_status_result(branch, upstream) {
        Ok(status) => Ok(status),
        Err(_) => Err("Failed to compare with upstream"),
    }
}

pub fn sync_with_upstream(upstream: &str, merge: bool) -> std::result::Result<(), &'static str> {
    match sync_with_upstream_result(upstream, merge) {
        Ok(()) => Ok(()),
        Err(_) => {
            if merge {
                Err("Merge failed")
            } else {
                Err("Rebase failed")
            }
        }
    }
}

pub fn get_upstream_branch(branch: &str) -> std::result::Result<String, &'static str> {
    match get_upstream_branch_result(branch) {
        Ok(upstream) => Ok(upstream),
        Err(_) => Err("No upstream branch configured"),
    }
}

pub fn get_current_branch() -> std::result::Result<String, &'static str> {
    match get_current_branch_result() {
        Ok(branch) => Ok(branch),
        Err(_) => Err("Not in a git repository"),
    }
}
