use crate::cli::UpstreamAction;
use std::collections::HashMap;
use std::process::Command;

pub fn run(action: UpstreamAction) {
    match action {
        UpstreamAction::Set { upstream } => set_upstream(upstream),
        UpstreamAction::Status => show_upstream_status(),
        UpstreamAction::SyncAll { dry_run, merge } => sync_all_branches(dry_run, merge),
    }
}

fn set_upstream(upstream: String) {
    // Validate upstream format
    if let Err(msg) = validate_upstream_format(&upstream) {
        eprintln!("{}", format_error_message(msg));
        return;
    }

    // Check if upstream exists
    if let Err(msg) = validate_upstream_exists(&upstream) {
        eprintln!("{}", format_error_message(msg));
        return;
    }

    // Get current branch
    let current_branch = match get_current_branch() {
        Ok(branch) => branch,
        Err(msg) => {
            eprintln!("{}", format_error_message(msg));
            return;
        }
    };

    println!(
        "{}",
        format_setting_upstream_message(&current_branch, &upstream)
    );

    // Set upstream
    if let Err(msg) = set_branch_upstream(&current_branch, &upstream) {
        eprintln!("{}", format_error_message(msg));
        return;
    }

    println!(
        "{}",
        format_upstream_set_message(&current_branch, &upstream)
    );
}

fn show_upstream_status() {
    // Get all local branches
    let branches = match get_all_local_branches() {
        Ok(branches) => branches,
        Err(msg) => {
            eprintln!("{}", format_error_message(msg));
            return;
        }
    };

    if branches.is_empty() {
        println!("{}", format_no_branches_message());
        return;
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
    let current_branch = get_current_branch().unwrap_or_default();

    println!("{}", format_upstream_status_header());

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
}

fn sync_all_branches(dry_run: bool, merge: bool) {
    // Get all branches with upstreams
    let branches = match get_branches_with_upstreams() {
        Ok(branches) => branches,
        Err(msg) => {
            eprintln!("{}", format_error_message(msg));
            return;
        }
    };

    if branches.is_empty() {
        println!("{}", format_no_upstream_branches_message());
        return;
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

// Helper function to validate upstream format
fn validate_upstream_format(upstream: &str) -> Result<(), &'static str> {
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

// Helper function to validate upstream exists
fn validate_upstream_exists(upstream: &str) -> Result<(), &'static str> {
    let output = Command::new("git")
        .args(["rev-parse", "--verify", upstream])
        .output()
        .map_err(|_| "Failed to validate upstream")?;

    if !output.status.success() {
        return Err("Upstream branch does not exist");
    }

    Ok(())
}

// Helper function to get current branch
fn get_current_branch() -> Result<String, &'static str> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .map_err(|_| "Failed to get current branch")?;

    if !output.status.success() {
        return Err("Not in a git repository");
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

// Helper function to set branch upstream
fn set_branch_upstream(branch: &str, upstream: &str) -> Result<(), &'static str> {
    let status = Command::new("git")
        .args(["branch", "--set-upstream-to", upstream, branch])
        .status()
        .map_err(|_| "Failed to set upstream")?;

    if !status.success() {
        return Err("Failed to set upstream branch");
    }

    Ok(())
}

// Helper function to get all local branches
fn get_all_local_branches() -> Result<Vec<String>, &'static str> {
    let output = Command::new("git")
        .args(["branch", "--format=%(refname:short)"])
        .output()
        .map_err(|_| "Failed to get local branches")?;

    if !output.status.success() {
        return Err("Failed to list local branches");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let branches: Vec<String> = stdout
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();

    Ok(branches)
}

// Helper function to get branch upstream
fn get_branch_upstream(branch: &str) -> Result<String, &'static str> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", &format!("{branch}@{{u}}")])
        .output()
        .map_err(|_| "Failed to get upstream")?;

    if !output.status.success() {
        return Err("No upstream configured");
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

// Helper function to get branch sync status
fn get_branch_sync_status(branch: &str, upstream: &str) -> Result<SyncStatus, &'static str> {
    let output = Command::new("git")
        .args([
            "rev-list",
            "--left-right",
            "--count",
            &format!("{upstream}...{branch}"),
        ])
        .output()
        .map_err(|_| "Failed to get sync status")?;

    if !output.status.success() {
        return Err("Failed to compare with upstream");
    }

    let counts = String::from_utf8_lossy(&output.stdout);
    let mut parts = counts.split_whitespace();

    let behind: u32 = parts
        .next()
        .and_then(|s| s.parse().ok())
        .ok_or("Invalid sync count format")?;

    let ahead: u32 = parts
        .next()
        .and_then(|s| s.parse().ok())
        .ok_or("Invalid sync count format")?;

    Ok(match (behind, ahead) {
        (0, 0) => SyncStatus::UpToDate,
        (b, 0) if b > 0 => SyncStatus::Behind(b),
        (0, a) if a > 0 => SyncStatus::Ahead(a),
        (b, a) if b > 0 && a > 0 => SyncStatus::Diverged(b, a),
        _ => SyncStatus::Unknown,
    })
}

// Helper function to get branches with upstreams
fn get_branches_with_upstreams() -> Result<Vec<(String, String)>, &'static str> {
    let branches = get_all_local_branches()?;
    let mut result = Vec::new();

    for branch in branches {
        if let Ok(upstream) = get_branch_upstream(&branch) {
            result.push((branch, upstream));
        }
    }

    Ok(result)
}

// Helper function to sync branch with upstream
fn sync_branch_with_upstream(
    branch: &str,
    upstream: &str,
    merge: bool,
) -> Result<(), &'static str> {
    // Switch to the branch first
    let status = Command::new("git")
        .args(["checkout", branch])
        .status()
        .map_err(|_| "Failed to checkout branch")?;

    if !status.success() {
        return Err("Failed to checkout branch");
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
        .map_err(|_| "Failed to sync with upstream")?;

    if !status.success() {
        return Err(if merge {
            "Merge failed"
        } else {
            "Rebase failed"
        });
    }

    Ok(())
}

// Helper function to get git branch set-upstream args
pub fn get_git_branch_set_upstream_args() -> [&'static str; 2] {
    ["branch", "--set-upstream-to"]
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
