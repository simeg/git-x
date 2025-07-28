use crate::cli::UpstreamAction;
use crate::{GitXError, Result};
use std::collections::HashMap;
use std::process::Command;

pub fn run(action: UpstreamAction) -> Result<String> {
    match action {
        UpstreamAction::Set { upstream } => set_upstream_result(upstream),
        UpstreamAction::Status => show_upstream_status_result(),
        UpstreamAction::SyncAll { dry_run, merge } => sync_all_branches_result(dry_run, merge),
    }
}

fn set_upstream_result(upstream: String) -> Result<String> {
    // Validate upstream format
    validate_upstream_format_result(&upstream)?;

    // Check if upstream exists
    validate_upstream_exists_result(&upstream)?;

    // Get current branch
    let current_branch = get_current_branch_result()?;

    let mut output = Vec::new();
    output.push(format_setting_upstream_message(&current_branch, &upstream));

    // Set upstream
    set_branch_upstream_result(&current_branch, &upstream)?;

    output.push(format_upstream_set_message(&current_branch, &upstream));
    Ok(output.join("\n"))
}

fn show_upstream_status_result() -> Result<String> {
    // Get all local branches
    let branches = get_all_local_branches_result()?;

    if branches.is_empty() {
        return Ok(format_no_branches_message().to_string());
    }

    // Get upstream info for each branch
    let mut branch_upstreams = HashMap::new();
    for branch in &branches {
        if let Ok(upstream) = get_branch_upstream_result(branch) {
            branch_upstreams.insert(branch.clone(), Some(upstream));
        } else {
            branch_upstreams.insert(branch.clone(), None);
        }
    }

    // Get current branch for highlighting
    let current_branch = get_current_branch_result().unwrap_or_default();

    let mut output = Vec::new();
    output.push(format_upstream_status_header().to_string());

    for branch in &branches {
        let is_current = branch == &current_branch;
        let upstream = branch_upstreams.get(branch).unwrap();

        match upstream {
            Some(upstream_ref) => {
                // Check sync status
                let sync_status = get_branch_sync_status_result(branch, upstream_ref)
                    .unwrap_or(SyncStatus::Unknown);

                output.push(format_branch_with_upstream(
                    branch,
                    upstream_ref,
                    &sync_status,
                    is_current,
                ));
            }
            None => {
                output.push(format_branch_without_upstream(branch, is_current));
            }
        }
    }

    Ok(output.join("\n"))
}

fn sync_all_branches_result(dry_run: bool, merge: bool) -> Result<String> {
    // Get all branches with upstreams
    let branches = get_branches_with_upstreams_result()?;

    if branches.is_empty() {
        return Ok(format_no_upstream_branches_message().to_string());
    }

    let mut output = Vec::new();
    output.push(format_sync_all_start_message(
        branches.len(),
        dry_run,
        merge,
    ));

    let mut sync_results = Vec::new();

    for (branch, upstream) in &branches {
        let sync_status = match get_branch_sync_status_result(branch, upstream) {
            Ok(status) => status,
            Err(_) => {
                sync_results.push((
                    branch.clone(),
                    SyncResult::Error("Failed to get sync status".to_string()),
                ));
                continue;
            }
        };

        match sync_status {
            SyncStatus::UpToDate => {
                sync_results.push((branch.clone(), SyncResult::UpToDate));
            }
            SyncStatus::Behind(_) | SyncStatus::Diverged(_, _) => {
                if dry_run {
                    sync_results.push((branch.clone(), SyncResult::WouldSync));
                } else {
                    match sync_branch_with_upstream_result(branch, upstream, merge) {
                        Ok(()) => sync_results.push((branch.clone(), SyncResult::Synced)),
                        Err(msg) => {
                            sync_results.push((branch.clone(), SyncResult::Error(msg.to_string())))
                        }
                    }
                }
            }
            SyncStatus::Ahead(_) => {
                sync_results.push((branch.clone(), SyncResult::Ahead));
            }
            SyncStatus::Unknown => {
                sync_results.push((
                    branch.clone(),
                    SyncResult::Error("Unknown sync status".to_string()),
                ));
            }
        }
    }

    // Add results
    output.push(format_sync_results_header().to_string());
    for (branch, result) in &sync_results {
        output.push(format_sync_result_line(branch, result));
    }

    // Add summary
    let synced_count = sync_results
        .iter()
        .filter(|(_, r)| matches!(r, SyncResult::Synced | SyncResult::WouldSync))
        .count();
    output.push(format_sync_summary(synced_count, dry_run));

    Ok(output.join("\n"))
}

#[derive(Debug, Clone)]
pub enum SyncStatus {
    UpToDate,
    Behind(u32),
    Ahead(u32),
    Diverged(u32, u32), // behind, ahead
    Unknown,
}

#[derive(Debug, Clone)]
pub enum SyncResult {
    UpToDate,
    Synced,
    WouldSync,
    Ahead,
    Error(String),
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

// Helper function to get all local branches (new version)
pub fn get_all_local_branches_result() -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(["branch", "--format=%(refname:short)"])
        .output()
        .map_err(|_| GitXError::GitCommand("Failed to get local branches".to_string()))?;

    if !output.status.success() {
        return Err(GitXError::GitCommand(
            "Failed to list local branches".to_string(),
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let branches: Vec<String> = stdout
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();

    Ok(branches)
}

// Helper function to get branches with upstreams (new version)
pub fn get_branches_with_upstreams_result() -> Result<Vec<(String, String)>> {
    let branches = get_all_local_branches_result()?;
    let mut result = Vec::new();

    for branch in branches {
        if let Ok(upstream) = get_branch_upstream_result(&branch) {
            result.push((branch, upstream));
        }
    }

    Ok(result)
}

// Helper function to validate upstream format (new version)
fn validate_upstream_format_result(upstream: &str) -> Result<()> {
    if upstream.is_empty() {
        return Err(GitXError::GitCommand(
            "Upstream cannot be empty".to_string(),
        ));
    }

    if !upstream.contains('/') {
        return Err(GitXError::GitCommand(
            "Upstream must be in format 'remote/branch' (e.g., origin/main)".to_string(),
        ));
    }

    let parts: Vec<&str> = upstream.split('/').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return Err(GitXError::GitCommand(
            "Invalid upstream format. Use 'remote/branch' format".to_string(),
        ));
    }

    Ok(())
}

// Helper function to validate upstream exists (new version)
fn validate_upstream_exists_result(upstream: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["rev-parse", "--verify", upstream])
        .output()
        .map_err(|_| GitXError::GitCommand("Failed to validate upstream".to_string()))?;

    if !output.status.success() {
        return Err(GitXError::GitCommand(
            "Upstream branch does not exist".to_string(),
        ));
    }

    Ok(())
}

// Helper function to set branch upstream (new version)
fn set_branch_upstream_result(branch: &str, upstream: &str) -> Result<()> {
    let status = Command::new("git")
        .args(["branch", "--set-upstream-to", upstream, branch])
        .status()
        .map_err(|_| GitXError::GitCommand("Failed to set upstream".to_string()))?;

    if !status.success() {
        return Err(GitXError::GitCommand(
            "Failed to set upstream branch".to_string(),
        ));
    }

    Ok(())
}

// Helper function to get branch upstream (new version)
fn get_branch_upstream_result(branch: &str) -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", &format!("{branch}@{{u}}")])
        .output()
        .map_err(|_| GitXError::GitCommand("Failed to get upstream".to_string()))?;

    if !output.status.success() {
        return Err(GitXError::GitCommand("No upstream configured".to_string()));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

// Helper function to get branch sync status (new version)
fn get_branch_sync_status_result(branch: &str, upstream: &str) -> Result<SyncStatus> {
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
    let mut parts = counts.split_whitespace();

    let behind: u32 = parts
        .next()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| GitXError::GitCommand("Invalid sync count format".to_string()))?;

    let ahead: u32 = parts
        .next()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| GitXError::GitCommand("Invalid sync count format".to_string()))?;

    Ok(match (behind, ahead) {
        (0, 0) => SyncStatus::UpToDate,
        (b, 0) if b > 0 => SyncStatus::Behind(b),
        (0, a) if a > 0 => SyncStatus::Ahead(a),
        (b, a) if b > 0 && a > 0 => SyncStatus::Diverged(b, a),
        _ => SyncStatus::Unknown,
    })
}

// Helper function to sync branch with upstream (new version)
fn sync_branch_with_upstream_result(branch: &str, upstream: &str, merge: bool) -> Result<()> {
    // Switch to the branch first
    let status = Command::new("git")
        .args(["checkout", branch])
        .status()
        .map_err(|_| GitXError::GitCommand("Failed to checkout branch".to_string()))?;

    if !status.success() {
        return Err(GitXError::GitCommand(
            "Failed to checkout branch".to_string(),
        ));
    }

    // Sync with upstream
    let args = if merge {
        ["merge", upstream]
    } else {
        ["rebase", upstream]
    };

    let status = Command::new("git")
        .args(args)
        .status()
        .map_err(|_| GitXError::GitCommand("Failed to sync with upstream".to_string()))?;

    if !status.success() {
        return Err(GitXError::GitCommand(if merge {
            "Merge failed".to_string()
        } else {
            "Rebase failed".to_string()
        }));
    }

    Ok(())
}

// Helper function to get git branch set-upstream args
pub fn get_git_branch_set_upstream_args() -> [&'static str; 2] {
    ["branch", "--set-upstream-to"]
}

// Backward compatibility functions for tests
pub fn validate_upstream_format(upstream: &str) -> std::result::Result<(), &'static str> {
    if upstream.is_empty() {
        return Err("Upstream cannot be empty");
    }

    if !upstream.contains('/') {
        return Err("Upstream must be in format 'remote/branch' (e.g., origin/main)");
    }

    let parts: Vec<&str> = upstream.split('/').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return Err("Invalid upstream format. Use 'remote/branch' format");
    }

    Ok(())
}

pub fn validate_upstream_exists(upstream: &str) -> std::result::Result<(), &'static str> {
    match validate_upstream_exists_result(upstream) {
        Ok(()) => Ok(()),
        Err(_) => Err("Upstream branch does not exist"),
    }
}

pub fn get_current_branch() -> std::result::Result<String, &'static str> {
    match get_current_branch_result() {
        Ok(branch) => Ok(branch),
        Err(_) => Err("Not in a git repository"),
    }
}

pub fn get_all_local_branches() -> std::result::Result<Vec<String>, &'static str> {
    match get_all_local_branches_result() {
        Ok(branches) => Ok(branches),
        Err(_) => Err("Failed to list local branches"),
    }
}

pub fn get_branch_sync_status(
    branch: &str,
    upstream: &str,
) -> std::result::Result<SyncStatus, &'static str> {
    match get_branch_sync_status_result(branch, upstream) {
        Ok(status) => Ok(status),
        Err(_) => Err("Failed to compare with upstream"),
    }
}

pub fn get_branches_with_upstreams() -> std::result::Result<Vec<(String, String)>, &'static str> {
    match get_branches_with_upstreams_result() {
        Ok(branches) => Ok(branches),
        Err(_) => Err("Failed to get branches with upstreams"),
    }
}

pub fn get_branch_upstream(branch: &str) -> std::result::Result<String, &'static str> {
    match get_branch_upstream_result(branch) {
        Ok(upstream) => Ok(upstream),
        Err(_) => Err("No upstream configured"),
    }
}

// Formatting functions
pub fn format_error_message(msg: &str) -> String {
    format!("❌ {msg}")
}

pub fn format_setting_upstream_message(branch: &str, upstream: &str) -> String {
    format!("🔗 Setting upstream for '{branch}' to '{upstream}'...")
}

pub fn format_upstream_set_message(branch: &str, upstream: &str) -> String {
    format!("✅ Upstream for '{branch}' set to '{upstream}'")
}

pub fn format_no_branches_message() -> &'static str {
    "ℹ️ No local branches found"
}

pub fn format_upstream_status_header() -> &'static str {
    "🔗 Upstream status for all branches:\n"
}

pub fn format_branch_with_upstream(
    branch: &str,
    upstream: &str,
    sync_status: &SyncStatus,
    is_current: bool,
) -> String {
    let current_indicator = if is_current { "* " } else { "  " };
    let status_text = match sync_status {
        SyncStatus::UpToDate => "✅ up-to-date",
        SyncStatus::Behind(n) => &format!("⬇️ {n} behind"),
        SyncStatus::Ahead(n) => &format!("⬆️ {n} ahead"),
        SyncStatus::Diverged(b, a) => &format!("🔀 {b} behind, {a} ahead"),
        SyncStatus::Unknown => "❓ unknown",
    };

    format!("{current_indicator}{branch} -> {upstream} ({status_text})")
}

pub fn format_branch_without_upstream(branch: &str, is_current: bool) -> String {
    let current_indicator = if is_current { "* " } else { "  " };
    format!("{current_indicator}{branch} -> (no upstream)")
}

pub fn format_no_upstream_branches_message() -> &'static str {
    "ℹ️ No branches with upstream configuration found"
}

pub fn format_sync_all_start_message(count: usize, dry_run: bool, merge: bool) -> String {
    let action = if merge { "merge" } else { "rebase" };
    if dry_run {
        format!("🧪 (dry run) Would sync {count} branch(es) with upstream using {action}:")
    } else {
        format!("🔄 Syncing {count} branch(es) with upstream using {action}:")
    }
}

pub fn format_sync_results_header() -> &'static str {
    "\n📊 Sync results:"
}

pub fn format_sync_result_line(branch: &str, result: &SyncResult) -> String {
    match result {
        SyncResult::UpToDate => format!("  ✅ {branch}: already up-to-date"),
        SyncResult::Synced => format!("  ✅ {branch}: synced successfully"),
        SyncResult::WouldSync => format!("  🔄 {branch}: would be synced"),
        SyncResult::Ahead => format!("  ⬆️ {branch}: ahead of upstream (skipped)"),
        SyncResult::Error(msg) => format!("  ❌ {branch}: {msg}"),
    }
}

pub fn format_sync_summary(synced_count: usize, dry_run: bool) -> String {
    if dry_run {
        format!(
            "\n💡 Would sync {synced_count} branch(es). Run without --dry-run to apply changes."
        )
    } else {
        format!("\n✅ Synced {synced_count} branch(es) successfully.")
    }
}
