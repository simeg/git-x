use crate::cli::StashBranchAction;
use std::process::Command;

pub fn run(action: StashBranchAction) {
    match action {
        StashBranchAction::Create { branch_name, stash_ref } => {
            create_branch_from_stash(branch_name, stash_ref)
        }
        StashBranchAction::Clean { older_than, dry_run } => {
            clean_old_stashes(older_than, dry_run)
        }
        StashBranchAction::ApplyByBranch { branch_name, list_only } => {
            apply_stashes_by_branch(branch_name, list_only)
        }
    }
}

fn create_branch_from_stash(branch_name: String, stash_ref: Option<String>) {
    // Validate branch name
    if let Err(msg) = validate_branch_name(&branch_name) {
        eprintln!("{}", format_error_message(msg));
        return;
    }

    // Check if branch already exists
    if branch_exists(&branch_name) {
        eprintln!("{}", format_branch_exists_message(&branch_name));
        return;
    }

    // Determine stash reference
    let stash = stash_ref.unwrap_or_else(|| "stash@{0}".to_string());

    // Validate stash exists
    if let Err(msg) = validate_stash_exists(&stash) {
        eprintln!("{}", format_error_message(msg));
        return;
    }

    println!("{}", format_creating_branch_message(&branch_name, &stash));

    // Create branch from stash
    if let Err(msg) = create_branch_from_stash_ref(&branch_name, &stash) {
        eprintln!("{}", format_error_message(msg));
        return;
    }

    println!("{}", format_branch_created_message(&branch_name));
}

fn clean_old_stashes(older_than: Option<String>, dry_run: bool) {
    // Get all stashes with timestamps
    let stashes = match get_stash_list_with_dates() {
        Ok(stashes) => stashes,
        Err(msg) => {
            eprintln!("{}", format_error_message(msg));
            return;
        }
    };

    if stashes.is_empty() {
        println!("{}", format_no_stashes_message());
        return;
    }

    // Filter stashes by age if specified
    let stashes_to_clean = if let Some(age) = older_than {
        match filter_stashes_by_age(&stashes, &age) {
            Ok(filtered) => filtered,
            Err(msg) => {
                eprintln!("{}", format_error_message(msg));
                return;
            }
        }
    } else {
        stashes
    };

    if stashes_to_clean.is_empty() {
        println!("{}", format_no_old_stashes_message());
        return;
    }

    println!("{}", format_stashes_to_clean_message(stashes_to_clean.len(), dry_run));

    for stash in &stashes_to_clean {
        println!("  {}", format_stash_entry(&stash.name, &stash.message));
    }

    if !dry_run {
        for stash in &stashes_to_clean {
            if let Err(msg) = delete_stash(&stash.name) {
                eprintln!("{}", format_error_message(&format!("Failed to delete {}: {}", stash.name, msg)));
            }
        }
        println!("{}", format_cleanup_complete_message(stashes_to_clean.len()));
    }
}

fn apply_stashes_by_branch(branch_name: String, list_only: bool) {
    // Get all stashes with their branch information
    let stashes = match get_stash_list_with_branches() {
        Ok(stashes) => stashes,
        Err(msg) => {
            eprintln!("{}", format_error_message(msg));
            return;
        }
    };

    // Filter stashes by branch
    let branch_stashes: Vec<_> = stashes
        .into_iter()
        .filter(|s| s.branch == branch_name)
        .collect();

    if branch_stashes.is_empty() {
        println!("{}", format_no_stashes_for_branch_message(&branch_name));
        return;
    }

    if list_only {
        println!("{}", format_stashes_for_branch_header(&branch_name, branch_stashes.len()));
        for stash in &branch_stashes {
            println!("  {}", format_stash_entry(&stash.name, &stash.message));
        }
    } else {
        println!("{}", format_applying_stashes_message(&branch_name, branch_stashes.len()));
        
        for stash in &branch_stashes {
            match apply_stash(&stash.name) {
                Ok(()) => println!("  ✅ Applied {}", stash.name),
                Err(msg) => eprintln!("  ❌ Failed to apply {}: {}", stash.name, msg),
            }
        }
    }
}

#[derive(Debug, Clone)]
struct StashInfo {
    name: String,
    message: String,
    branch: String,
    #[allow(dead_code)]
    timestamp: String,
}

// Helper function to validate branch name
fn validate_branch_name(name: &str) -> Result<(), &'static str> {
    if name.is_empty() {
        return Err("Branch name cannot be empty");
    }

    if name.starts_with('-') {
        return Err("Branch name cannot start with a dash");
    }

    if name.contains("..") {
        return Err("Branch name cannot contain '..'");
    }

    if name.contains(' ') {
        return Err("Branch name cannot contain spaces");
    }

    Ok(())
}

// Helper function to check if branch exists
fn branch_exists(branch_name: &str) -> bool {
    Command::new("git")
        .args(["show-ref", "--verify", "--quiet", &format!("refs/heads/{branch_name}")])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

// Helper function to validate stash exists
fn validate_stash_exists(stash_ref: &str) -> Result<(), &'static str> {
    let output = Command::new("git")
        .args(["rev-parse", "--verify", stash_ref])
        .output()
        .map_err(|_| "Failed to validate stash reference")?;

    if !output.status.success() {
        return Err("Stash reference does not exist");
    }

    Ok(())
}

// Helper function to create branch from stash
fn create_branch_from_stash_ref(branch_name: &str, stash_ref: &str) -> Result<(), &'static str> {
    let status = Command::new("git")
        .args(["stash", "branch", branch_name, stash_ref])
        .status()
        .map_err(|_| "Failed to create branch from stash")?;

    if !status.success() {
        return Err("Failed to create branch from stash");
    }

    Ok(())
}

// Helper function to get stash list with dates
fn get_stash_list_with_dates() -> Result<Vec<StashInfo>, &'static str> {
    let output = Command::new("git")
        .args(["stash", "list", "--pretty=format:%gd|%s|%gD"])
        .output()
        .map_err(|_| "Failed to get stash list")?;

    if !output.status.success() {
        return Err("Failed to retrieve stash list");
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

// Helper function to get stash list with branches
fn get_stash_list_with_branches() -> Result<Vec<StashInfo>, &'static str> {
    let output = Command::new("git")
        .args(["stash", "list", "--pretty=format:%gd|%s"])
        .output()
        .map_err(|_| "Failed to get stash list")?;

    if !output.status.success() {
        return Err("Failed to retrieve stash list");
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
fn parse_stash_line_with_date(line: &str) -> Option<StashInfo> {
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
fn parse_stash_line_with_branch(line: &str) -> Option<StashInfo> {
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
fn extract_branch_from_message(message: &str) -> String {
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

// Helper function to filter stashes by age
fn filter_stashes_by_age(stashes: &[StashInfo], age: &str) -> Result<Vec<StashInfo>, &'static str> {
    // For simplicity, we'll implement basic age filtering
    // In a real implementation, you'd parse the age string and compare timestamps
    if age.ends_with('d') || age.ends_with('w') || age.ends_with('m') {
        // This is a placeholder - real implementation would parse timestamps
        Ok(stashes.to_vec())
    } else {
        Err("Invalid age format. Use format like '7d', '2w', '1m'")
    }
}

// Helper function to delete stash
fn delete_stash(stash_name: &str) -> Result<(), &'static str> {
    let status = Command::new("git")
        .args(["stash", "drop", stash_name])
        .status()
        .map_err(|_| "Failed to delete stash")?;

    if !status.success() {
        return Err("Failed to delete stash");
    }

    Ok(())
}

// Helper function to apply stash
fn apply_stash(stash_name: &str) -> Result<(), &'static str> {
    let status = Command::new("git")
        .args(["stash", "apply", stash_name])
        .status()
        .map_err(|_| "Failed to apply stash")?;

    if !status.success() {
        return Err("Failed to apply stash");
    }

    Ok(())
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