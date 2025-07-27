use std::process::Command;

pub fn run(merge: bool) {
    // Get current branch
    let current_branch = match get_current_branch() {
        Ok(branch) => branch,
        Err(msg) => {
            eprintln!("{}", format_error_message(msg));
            return;
        }
    };

    // Get upstream branch
    let upstream = match get_upstream_branch(&current_branch) {
        Ok(upstream) => upstream,
        Err(msg) => {
            eprintln!("{}", format_error_message(msg));
            return;
        }
    };

    println!("{}", format_sync_start_message(&current_branch, &upstream));

    // Fetch from remote
    if let Err(msg) = fetch_upstream(&upstream) {
        eprintln!("{}", format_error_message(msg));
        return;
    }

    // Check if we're ahead of upstream
    let status = match get_sync_status(&current_branch, &upstream) {
        Ok(status) => status,
        Err(msg) => {
            eprintln!("{}", format_error_message(msg));
            return;
        }
    };

    match status {
        SyncStatus::UpToDate => {
            println!("{}", format_up_to_date_message());
        }
        SyncStatus::Behind(count) => {
            println!("{}", format_behind_message(count));
            if let Err(msg) = sync_with_upstream(&upstream, merge) {
                eprintln!("{}", format_error_message(msg));
            } else {
                println!("{}", format_sync_success_message(merge));
            }
        }
        SyncStatus::Ahead(count) => {
            println!("{}", format_ahead_message(count));
        }
        SyncStatus::Diverged(behind, ahead) => {
            println!("{}", format_diverged_message(behind, ahead));
            if merge {
                if let Err(msg) = sync_with_upstream(&upstream, merge) {
                    eprintln!("{}", format_error_message(msg));
                } else {
                    println!("{}", format_sync_success_message(merge));
                }
            } else {
                println!("{}", format_diverged_help_message());
            }
        }
    }
}

#[derive(Debug, PartialEq)]
enum SyncStatus {
    UpToDate,
    Behind(u32),
    Ahead(u32),
    Diverged(u32, u32), // behind, ahead
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

// Helper function to get upstream branch
fn get_upstream_branch(branch: &str) -> Result<String, &'static str> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", &format!("{branch}@{{u}}")])
        .output()
        .map_err(|_| "Failed to get upstream branch")?;

    if !output.status.success() {
        return Err("No upstream branch configured");
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

// Helper function to fetch from upstream
fn fetch_upstream(upstream: &str) -> Result<(), &'static str> {
    let remote = upstream.split('/').next().unwrap_or("origin");

    let status = Command::new("git")
        .args(["fetch", remote])
        .status()
        .map_err(|_| "Failed to execute fetch command")?;

    if !status.success() {
        return Err("Failed to fetch from remote");
    }

    Ok(())
}

// Helper function to get sync status
fn get_sync_status(branch: &str, upstream: &str) -> Result<SyncStatus, &'static str> {
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
    let (behind, ahead) = parse_sync_counts(&counts)?;

    Ok(match (behind, ahead) {
        (0, 0) => SyncStatus::UpToDate,
        (b, 0) if b > 0 => SyncStatus::Behind(b),
        (0, a) if a > 0 => SyncStatus::Ahead(a),
        (b, a) if b > 0 && a > 0 => SyncStatus::Diverged(b, a),
        _ => SyncStatus::UpToDate,
    })
}

// Helper function to parse sync counts
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

// Helper function to sync with upstream
fn sync_with_upstream(upstream: &str, merge: bool) -> Result<(), &'static str> {
    let args = if merge {
        ["merge", upstream]
    } else {
        ["rebase", upstream]
    };

    let status = Command::new("git")
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
