use crate::cli::UpstreamAction;
use crate::command::Command;
use crate::core::git::{GitOperations, RemoteOperations};
use crate::{GitXError, Result};
use std::collections::HashMap;
use std::process::Command as StdCommand;

pub fn run(action: UpstreamAction) -> Result<()> {
    let cmd = UpstreamCommand;
    cmd.execute(action)
}

/// Command implementation for git upstream
pub struct UpstreamCommand;

impl Command for UpstreamCommand {
    type Input = UpstreamAction;
    type Output = ();

    fn execute(&self, action: UpstreamAction) -> Result<()> {
        run_upstream(action)
    }

    fn name(&self) -> &'static str {
        "upstream"
    }

    fn description(&self) -> &'static str {
        "Manage upstream tracking for branches"
    }
}

fn run_upstream(action: UpstreamAction) -> Result<()> {
    match action {
        UpstreamAction::Set { upstream } => set_upstream(upstream),
        UpstreamAction::Status => show_upstream_status(),
        UpstreamAction::SyncAll { dry_run, merge } => sync_all_branches(dry_run, merge),
    }
}

fn set_upstream(upstream: String) -> Result<()> {
    // Validate upstream format
    validate_upstream_format(&upstream).map_err(|e| GitXError::GitCommand(e.to_string()))?;

    // Check if upstream exists
    validate_upstream_exists(&upstream).map_err(|e| GitXError::GitCommand(e.to_string()))?;

    // Get current branch
    let current_branch = GitOperations::current_branch()
        .map_err(|e| GitXError::GitCommand(format!("Failed to get current branch: {e}")))?;

    println!(
        "🔗 Setting upstream for '{}' to '{}'...",
        &current_branch, &upstream
    );

    // Set upstream
    RemoteOperations::set_upstream(&current_branch, &upstream)
        .map_err(|e| GitXError::GitCommand(e.to_string()))?;

    println!(
        "✅ Upstream for '{}' set to '{}'",
        &current_branch, &upstream
    );
    Ok(())
}

fn show_upstream_status() -> Result<()> {
    // Get all local branches
    let branches = GitOperations::local_branches()
        .map_err(|e| GitXError::GitCommand(format!("Failed to get local branches: {e}")))?;

    if branches.is_empty() {
        println!("ℹ️ No local branches found");
        return Ok(());
    }

    // Get upstream info for each branch
    let mut branch_upstreams = HashMap::new();
    for branch in &branches {
        if let Ok(upstream) = get_branch_upstream(branch) {
            branch_upstreams.insert(branch.clone(), Some(upstream));
        } else {
            branch_upstreams.insert(branch.clone(), None);
        }
    }

    // Get current branch for highlighting
    let current_branch = GitOperations::current_branch().unwrap_or_default();

    println!("🔗 Upstream status for all branches:\n");

    for branch in &branches {
        let is_current = branch == &current_branch;
        let upstream = branch_upstreams.get(branch).unwrap();

        match upstream {
            Some(upstream_ref) => {
                // Check sync status
                let sync_status =
                    get_branch_sync_status(branch, upstream_ref).unwrap_or(SyncStatus::Unknown);

                println!(
                    "{}",
                    format_branch_with_upstream(branch, upstream_ref, &sync_status, is_current)
                );
            }
            None => {
                println!("{}", format_branch_without_upstream(branch, is_current));
            }
        }
    }
    Ok(())
}

fn sync_all_branches(dry_run: bool, merge: bool) -> Result<()> {
    // Get all branches with upstreams
    let branches =
        get_branches_with_upstreams().map_err(|e| GitXError::GitCommand(e.to_string()))?;

    if branches.is_empty() {
        println!("{}", format_no_upstream_branches_message());
        return Ok(());
    }

    println!(
        "{}",
        format_sync_all_start_message(branches.len(), dry_run, merge)
    );

    let mut sync_results = Vec::new();

    for (branch, upstream) in &branches {
        let sync_status = match get_branch_sync_status(branch, upstream) {
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
                    match sync_branch_with_upstream(branch, upstream, merge) {
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

    // Print results
    println!("{}", format_sync_results_header());
    for (branch, result) in &sync_results {
        println!("{}", format_sync_result_line(branch, result));
    }

    // Print summary
    let synced_count = sync_results
        .iter()
        .filter(|(_, r)| matches!(r, SyncResult::Synced | SyncResult::WouldSync))
        .count();
    println!("{}", format_sync_summary(synced_count, dry_run));
    Ok(())
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

pub fn validate_upstream_format(upstream: &str) -> Result<()> {
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

pub fn validate_upstream_exists(upstream: &str) -> Result<()> {
    let output = StdCommand::new("git")
        .args(["rev-parse", "--verify", upstream])
        .output()
        .map_err(GitXError::Io)?;

    if !output.status.success() {
        return Err(GitXError::GitCommand(
            "Upstream branch does not exist".to_string(),
        ));
    }

    Ok(())
}

pub fn get_all_local_branches() -> Result<Vec<String>> {
    let output = StdCommand::new("git")
        .args(["branch", "--format=%(refname:short)"])
        .output()
        .map_err(GitXError::Io)?;

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

pub fn get_branch_upstream(branch: &str) -> Result<String> {
    let output = StdCommand::new("git")
        .args(["rev-parse", "--abbrev-ref", &format!("{branch}@{{u}}")])
        .output()
        .map_err(GitXError::Io)?;

    if !output.status.success() {
        return Err(GitXError::GitCommand("No upstream configured".to_string()));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn get_branch_sync_status(branch: &str, upstream: &str) -> Result<SyncStatus> {
    let output = StdCommand::new("git")
        .args([
            "rev-list",
            "--left-right",
            "--count",
            &format!("{upstream}...{branch}"),
        ])
        .output()
        .map_err(GitXError::Io)?;

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
        .ok_or_else(|| GitXError::Parse("Invalid sync count format".to_string()))?;

    let ahead: u32 = parts
        .next()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| GitXError::Parse("Invalid sync count format".to_string()))?;

    Ok(match (behind, ahead) {
        (0, 0) => SyncStatus::UpToDate,
        (b, 0) if b > 0 => SyncStatus::Behind(b),
        (0, a) if a > 0 => SyncStatus::Ahead(a),
        (b, a) if b > 0 && a > 0 => SyncStatus::Diverged(b, a),
        _ => SyncStatus::Unknown,
    })
}

pub fn get_branches_with_upstreams() -> Result<Vec<(String, String)>> {
    let branches = get_all_local_branches()?;
    let mut result = Vec::new();

    for branch in branches {
        if let Ok(upstream) = get_branch_upstream(&branch) {
            result.push((branch, upstream));
        }
    }

    Ok(result)
}

fn sync_branch_with_upstream(branch: &str, upstream: &str, merge: bool) -> Result<()> {
    // Switch to the branch first
    let status = StdCommand::new("git")
        .args(["checkout", branch])
        .status()
        .map_err(GitXError::Io)?;

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

    let status = StdCommand::new("git")
        .args(args)
        .status()
        .map_err(GitXError::Io)?;

    if !status.success() {
        return Err(if merge {
            GitXError::GitCommand("Merge failed".to_string())
        } else {
            GitXError::GitCommand("Rebase failed".to_string())
        });
    }

    Ok(())
}

pub fn format_upstream_set_message(branch: &str, upstream: &str) -> String {
    format!("✅ Upstream for '{branch}' set to '{upstream}'")
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
