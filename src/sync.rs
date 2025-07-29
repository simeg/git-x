use crate::GitXError;
use crate::command::Command;
use crate::core::git::{GitOperations, RemoteOperations};
use std::process::Command as StdCommand;

pub fn run(merge: bool) -> Result<(), GitXError> {
    let cmd = SyncCommand;
    cmd.execute(merge)
}

/// Command implementation for git sync
pub struct SyncCommand;

impl Command for SyncCommand {
    type Input = bool;
    type Output = ();

    fn execute(&self, merge: bool) -> Result<(), GitXError> {
        run_sync(merge)
    }

    fn name(&self) -> &'static str {
        "sync"
    }

    fn description(&self) -> &'static str {
        "Sync current branch with its upstream"
    }
}

fn run_sync(merge: bool) -> Result<(), GitXError> {
    // Get current branch
    let current_branch = GitOperations::current_branch()
        .map_err(|e| GitXError::GitCommand(format!("Failed to get current branch: {e}")))?;

    // Get upstream branch
    let upstream = GitOperations::upstream_branch()
        .map_err(|e| GitXError::GitCommand(format!("Failed to get upstream branch: {e}")))?;

    println!(
        "🔄 Syncing branch '{}' with '{}'...",
        &current_branch, &upstream
    );

    // Fetch from remote
    let remote = upstream.split('/').next().unwrap_or("origin");
    RemoteOperations::fetch(Some(remote))
        .map_err(|e| GitXError::GitCommand(format!("Failed to fetch from remote: {e}")))?;

    // Check if we're ahead of upstream
    let (ahead, behind) = GitOperations::ahead_behind_counts()
        .map_err(|e| GitXError::GitCommand(format!("Failed to get ahead/behind counts: {e}")))?;

    let status = match (behind, ahead) {
        (0, 0) => SyncStatus::UpToDate,
        (b, 0) if b > 0 => SyncStatus::Behind(b),
        (0, a) if a > 0 => SyncStatus::Ahead(a),
        (b, a) if b > 0 && a > 0 => SyncStatus::Diverged(b, a),
        _ => SyncStatus::UpToDate,
    };

    match status {
        SyncStatus::UpToDate => {
            println!("✅ Branch is up to date with upstream");
        }
        SyncStatus::Behind(count) => {
            println!("{count}");
            let args = if merge {
                vec!["merge", &upstream]
            } else {
                vec!["rebase", &upstream]
            };

            match GitOperations::run_status(&args) {
                Err(e) => return Err(GitXError::GitCommand(format!("Sync failed: {e}"))),
                Ok(()) => println!("{}", format_sync_success_message(merge)),
            }
        }
        SyncStatus::Ahead(count) => {
            println!("{count}");
        }
        SyncStatus::Diverged(behind, ahead) => {
            println!("🔀 Branch has diverged: {behind} behind, {ahead} ahead");
            if merge {
                let args = if merge {
                    vec!["merge", &upstream]
                } else {
                    vec!["rebase", &upstream]
                };

                match GitOperations::run_status(&args) {
                    Err(e) => return Err(GitXError::GitCommand(format!("Sync failed: {e}"))),
                    Ok(()) => println!("{}", format_sync_success_message(merge)),
                }
            } else {
                println!("💡 Use --merge flag to merge changes, or handle manually");
            }
        }
    }

    Ok(())
}

#[derive(Debug, PartialEq)]
pub enum SyncStatus {
    UpToDate,
    Behind(u32),
    Ahead(u32),
    Diverged(u32, u32), // behind, ahead
}

pub fn get_upstream_branch(branch: &str) -> Result<String, &'static str> {
    let output = StdCommand::new("git")
        .args(["rev-parse", "--abbrev-ref", &format!("{branch}@{{u}}")])
        .output()
        .map_err(|_| "Failed to get upstream branch")?;

    if !output.status.success() {
        return Err("No upstream branch configured");
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn fetch_upstream(upstream: &str) -> Result<(), &'static str> {
    let remote = upstream.split('/').next().unwrap_or("origin");

    let status = StdCommand::new("git")
        .args(["fetch", remote])
        .status()
        .map_err(|_| "Failed to execute fetch command")?;

    if !status.success() {
        return Err("Failed to fetch from remote");
    }

    Ok(())
}

pub fn get_sync_status(branch: &str, upstream: &str) -> Result<SyncStatus, &'static str> {
    let output = StdCommand::new("git")
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
    let (behind, ahead) = parse_sync_counts(&counts)?;

    Ok(match (behind, ahead) {
        (0, 0) => SyncStatus::UpToDate,
        (b, 0) if b > 0 => SyncStatus::Behind(b),
        (0, a) if a > 0 => SyncStatus::Ahead(a),
        (b, a) if b > 0 && a > 0 => SyncStatus::Diverged(b, a),
        _ => SyncStatus::UpToDate,
    })
}

pub fn parse_sync_counts(output: &str) -> Result<(u32, u32), &'static str> {
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

pub fn sync_with_upstream(upstream: &str, merge: bool) -> Result<(), &'static str> {
    let args = if merge {
        ["merge", upstream]
    } else {
        ["rebase", upstream]
    };

    let status = StdCommand::new("git")
        .args(args)
        .status()
        .map_err(|_| "Failed to execute sync command")?;

    if !status.success() {
        return Err(if merge {
            "Merge failed"
        } else {
            "Rebase failed"
        });
    }

    Ok(())
}

// Helper function to format sync success message
pub fn format_sync_success_message(merge: bool) -> String {
    if merge {
        "✅ Successfully merged upstream changes".to_string()
    } else {
        "✅ Successfully rebased onto upstream".to_string()
    }
}
