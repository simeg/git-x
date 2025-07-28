use crate::cli::StashBranchAction;
use crate::{GitXError, Result};
use std::process::Command;

pub fn run(action: StashBranchAction) -> Result<String> {
    match action {
        StashBranchAction::Create {
            branch_name,
            stash_ref,
        } => create_branch_from_stash_result(branch_name, stash_ref),
        StashBranchAction::Clean {
            older_than,
            dry_run,
        } => clean_old_stashes_result(older_than, dry_run),
        StashBranchAction::ApplyByBranch {
            branch_name,
            list_only,
        } => apply_stashes_by_branch_result(branch_name, list_only),
    }
}

fn create_branch_from_stash_result(
    branch_name: String,
    stash_ref: Option<String>,
) -> Result<String> {
    // Validate branch name
    validate_branch_name_result(&branch_name)?;

    // Check if branch already exists
    if branch_exists(&branch_name) {
        return Err(GitXError::GitCommand(format!(
            "Branch '{branch_name}' already exists"
        )));
    }

    // Determine stash reference
    let stash = stash_ref.unwrap_or_else(|| "stash@{0}".to_string());

    // Validate stash exists
    validate_stash_exists_result(&stash)?;

    let mut output = Vec::new();
    output.push(format_creating_branch_message(&branch_name, &stash));

    // Create branch from stash
    create_branch_from_stash_ref_result(&branch_name, &stash)?;

    output.push(format_branch_created_message(&branch_name));
    Ok(output.join("\n"))
}

fn clean_old_stashes_result(older_than: Option<String>, dry_run: bool) -> Result<String> {
    // Get all stashes with timestamps
    let stashes = get_stash_list_with_dates_result()?;

    if stashes.is_empty() {
        return Ok(format_no_stashes_message().to_string());
    }

    // Filter stashes by age if specified
    let stashes_to_clean = if let Some(age) = older_than {
        filter_stashes_by_age_result(&stashes, &age)?
    } else {
        stashes
    };

    if stashes_to_clean.is_empty() {
        return Ok(format_no_old_stashes_message().to_string());
    }

    let mut output = Vec::new();
    output.push(format_stashes_to_clean_message(
        stashes_to_clean.len(),
        dry_run,
    ));

    for stash in &stashes_to_clean {
        output.push(format!(
            "  {}",
            format_stash_entry(&stash.name, &stash.message)
        ));
    }

    if !dry_run {
        for stash in &stashes_to_clean {
            delete_stash_result(&stash.name)?;
        }
        output.push(format_cleanup_complete_message(stashes_to_clean.len()));
    }

    Ok(output.join("\n"))
}

fn apply_stashes_by_branch_result(branch_name: String, list_only: bool) -> Result<String> {
    // Get all stashes with their branch information
    let stashes = get_stash_list_with_branches_result()?;

    // Filter stashes by branch
    let branch_stashes: Vec<_> = stashes
        .into_iter()
        .filter(|s| s.branch == branch_name)
        .collect();

    if branch_stashes.is_empty() {
        return Ok(format_no_stashes_for_branch_message(&branch_name));
    }

    let mut output = Vec::new();

    if list_only {
        output.push(format_stashes_for_branch_header(
            &branch_name,
            branch_stashes.len(),
        ));
        for stash in &branch_stashes {
            output.push(format!(
                "  {}",
                format_stash_entry(&stash.name, &stash.message)
            ));
        }
    } else {
        output.push(format_applying_stashes_message(
            &branch_name,
            branch_stashes.len(),
        ));

        for stash in &branch_stashes {
            match apply_stash_result(&stash.name) {
                Ok(()) => output.push(format!("  ✅ Applied {}", stash.name)),
                Err(err) => output.push(format!("  ❌ Failed to apply {}: {}", stash.name, err)),
            }
        }
    }

    Ok(output.join("\n"))
}

#[derive(Debug, Clone)]
pub struct StashInfo {
    pub name: String,
    pub message: String,
    pub branch: String,
    #[allow(dead_code)]
    pub timestamp: String,
}

// Helper function to validate branch name (new version)
fn validate_branch_name_result(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(GitXError::GitCommand(
            "Branch name cannot be empty".to_string(),
        ));
    }

    if name.starts_with('-') {
        return Err(GitXError::GitCommand(
            "Branch name cannot start with a dash".to_string(),
        ));
    }

    if name.contains("..") {
        return Err(GitXError::GitCommand(
            "Branch name cannot contain '..'".to_string(),
        ));
    }

    if name.contains(' ') {
        return Err(GitXError::GitCommand(
            "Branch name cannot contain spaces".to_string(),
        ));
    }

    // Check for invalid characters
    const INVALID_CHARS: &[char] = &['~', '^', ':', '?', '*', '[', '\\'];
    if name.chars().any(|c| INVALID_CHARS.contains(&c)) {
        return Err(GitXError::GitCommand(
            "Branch name contains invalid characters".to_string(),
        ));
    }

    Ok(())
}

// Helper function to validate stash exists (new version)
fn validate_stash_exists_result(stash_ref: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["rev-parse", "--verify", stash_ref])
        .output()
        .map_err(|_| GitXError::GitCommand("Failed to validate stash reference".to_string()))?;

    if !output.status.success() {
        return Err(GitXError::GitCommand(
            "Stash reference does not exist".to_string(),
        ));
    }

    Ok(())
}

// Helper function to create branch from stash (new version)
fn create_branch_from_stash_ref_result(branch_name: &str, stash_ref: &str) -> Result<()> {
    let status = Command::new("git")
        .args(["stash", "branch", branch_name, stash_ref])
        .status()
        .map_err(|_| GitXError::GitCommand("Failed to create branch from stash".to_string()))?;

    if !status.success() {
        return Err(GitXError::GitCommand(
            "Failed to create branch from stash".to_string(),
        ));
    }

    Ok(())
}

// Helper function to delete stash (new version)
fn delete_stash_result(stash_name: &str) -> Result<()> {
    let status = Command::new("git")
        .args(["stash", "drop", stash_name])
        .status()
        .map_err(|_| GitXError::GitCommand("Failed to delete stash".to_string()))?;

    if !status.success() {
        return Err(GitXError::GitCommand("Failed to delete stash".to_string()));
    }

    Ok(())
}

// Helper function to apply stash (new version)
fn apply_stash_result(stash_name: &str) -> Result<()> {
    let status = Command::new("git")
        .args(["stash", "apply", stash_name])
        .status()
        .map_err(|_| GitXError::GitCommand("Failed to apply stash".to_string()))?;

    if !status.success() {
        return Err(GitXError::GitCommand("Failed to apply stash".to_string()));
    }

    Ok(())
}

// Helper function to check if branch exists
pub fn branch_exists(branch_name: &str) -> bool {
    Command::new("git")
        .args([
            "show-ref",
            "--verify",
            "--quiet",
            &format!("refs/heads/{branch_name}"),
        ])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

// Helper function to get stash list with dates (new version)
fn get_stash_list_with_dates_result() -> Result<Vec<StashInfo>> {
    let output = Command::new("git")
        .args(["stash", "list", "--pretty=format:%gd|%s|%gD"])
        .output()
        .map_err(|_| GitXError::GitCommand("Failed to get stash list".to_string()))?;

    if !output.status.success() {
        return Err(GitXError::GitCommand(
            "Failed to retrieve stash list".to_string(),
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut stashes = Vec::new();

    for line in stdout.lines() {
        if let Some(stash) = parse_stash_line_with_date(line) {
            stashes.push(stash);
        }
    }

    Ok(stashes)
}

// Helper function to get stash list with branches (new version)
fn get_stash_list_with_branches_result() -> Result<Vec<StashInfo>> {
    let output = Command::new("git")
        .args(["stash", "list", "--pretty=format:%gd|%s"])
        .output()
        .map_err(|_| GitXError::GitCommand("Failed to get stash list".to_string()))?;

    if !output.status.success() {
        return Err(GitXError::GitCommand(
            "Failed to retrieve stash list".to_string(),
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut stashes = Vec::new();

    for line in stdout.lines() {
        if let Some(stash) = parse_stash_line_with_branch(line) {
            stashes.push(stash);
        }
    }

    Ok(stashes)
}

// Helper function to parse stash line with date
pub fn parse_stash_line_with_date(line: &str) -> Option<StashInfo> {
    let parts: Vec<&str> = line.splitn(3, '|').collect();
    if parts.len() != 3 {
        return None;
    }

    Some(StashInfo {
        name: parts[0].to_string(),
        message: parts[1].to_string(),
        branch: extract_branch_from_message(parts[1]),
        timestamp: parts[2].to_string(),
    })
}

// Helper function to parse stash line with branch
pub fn parse_stash_line_with_branch(line: &str) -> Option<StashInfo> {
    let parts: Vec<&str> = line.splitn(2, '|').collect();
    if parts.len() != 2 {
        return None;
    }

    Some(StashInfo {
        name: parts[0].to_string(),
        message: parts[1].to_string(),
        branch: extract_branch_from_message(parts[1]),
        timestamp: String::new(),
    })
}

// Helper function to extract branch name from stash message
pub fn extract_branch_from_message(message: &str) -> String {
    // Stash messages typically start with "On branch_name:" or "WIP on branch_name:"
    if let Some(start) = message.find("On ") {
        let rest = &message[start + 3..];
        if let Some(end) = rest.find(':') {
            return rest[..end].to_string();
        }
    }

    if let Some(start) = message.find("WIP on ") {
        let rest = &message[start + 7..];
        if let Some(end) = rest.find(':') {
            return rest[..end].to_string();
        }
    }

    "unknown".to_string()
}

// Helper function to filter stashes by age (new version)
pub fn filter_stashes_by_age_result(stashes: &[StashInfo], age: &str) -> Result<Vec<StashInfo>> {
    // For simplicity, we'll implement basic age filtering
    // In a real implementation, you'd parse the age string and compare timestamps
    if age.ends_with('d') || age.ends_with('w') || age.ends_with('m') {
        // This is a placeholder - real implementation would parse timestamps
        Ok(stashes.to_vec())
    } else {
        Err(GitXError::GitCommand(
            "Invalid age format. Use format like '7d', '2w', '1m'".to_string(),
        ))
    }
}

// Helper function to get git stash branch args
pub fn get_git_stash_branch_args() -> [&'static str; 2] {
    ["stash", "branch"]
}

// Helper function to get git stash drop args
pub fn get_git_stash_drop_args() -> [&'static str; 2] {
    ["stash", "drop"]
}

// Formatting functions
pub fn format_error_message(msg: &str) -> String {
    format!("❌ {msg}")
}

pub fn format_branch_exists_message(branch_name: &str) -> String {
    format!("❌ Branch '{branch_name}' already exists")
}

pub fn format_creating_branch_message(branch_name: &str, stash_ref: &str) -> String {
    format!("🌿 Creating branch '{branch_name}' from {stash_ref}...")
}

pub fn format_branch_created_message(branch_name: &str) -> String {
    format!("✅ Branch '{branch_name}' created and checked out")
}

pub fn format_no_stashes_message() -> &'static str {
    "ℹ️ No stashes found"
}

pub fn format_no_old_stashes_message() -> &'static str {
    "✅ No old stashes to clean"
}

pub fn format_stashes_to_clean_message(count: usize, dry_run: bool) -> String {
    if dry_run {
        format!("🧪 (dry run) Would clean {count} stash(es):")
    } else {
        format!("🧹 Cleaning {count} stash(es):")
    }
}

pub fn format_cleanup_complete_message(count: usize) -> String {
    format!("✅ Cleaned {count} stash(es)")
}

pub fn format_no_stashes_for_branch_message(branch_name: &str) -> String {
    format!("ℹ️ No stashes found for branch '{branch_name}'")
}

pub fn format_stashes_for_branch_header(branch_name: &str, count: usize) -> String {
    format!("📋 Found {count} stash(es) for branch '{branch_name}':")
}

pub fn format_applying_stashes_message(branch_name: &str, count: usize) -> String {
    format!("🔄 Applying {count} stash(es) from branch '{branch_name}':")
}

pub fn format_stash_entry(name: &str, message: &str) -> String {
    format!("{name}: {message}")
}

// Backward compatibility function for tests
pub fn validate_branch_name(name: &str) -> std::result::Result<(), &'static str> {
    match validate_branch_name_result(name) {
        Ok(()) => Ok(()),
        Err(_) => Err("Invalid branch name"),
    }
}

pub fn validate_stash_exists(stash_ref: &str) -> std::result::Result<(), &'static str> {
    match validate_stash_exists_result(stash_ref) {
        Ok(()) => Ok(()),
        Err(_) => Err("Stash reference does not exist"),
    }
}

pub fn filter_stashes_by_age(
    stashes: &[StashInfo],
    age: &str,
) -> std::result::Result<Vec<StashInfo>, &'static str> {
    match filter_stashes_by_age_result(stashes, age) {
        Ok(filtered) => Ok(filtered),
        Err(_) => Err("Invalid age format. Use format like '7d', '2w', '1m'"),
    }
}
