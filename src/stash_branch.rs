use crate::cli::StashBranchAction;
use crate::command::Command;
use crate::core::git::{BranchOperations, GitOperations};
use crate::core::safety::Safety;
use crate::{GitXError, Result};
use std::process::Command as StdCommand;

pub fn run(action: StashBranchAction) -> Result<()> {
    let cmd = StashBranchCommand;
    cmd.execute(action)
}

/// Command implementation for git stash-branch
pub struct StashBranchCommand;

impl Command for StashBranchCommand {
    type Input = StashBranchAction;
    type Output = ();

    fn execute(&self, action: StashBranchAction) -> Result<()> {
        run_stash_branch(action)
    }

    fn name(&self) -> &'static str {
        "stash-branch"
    }

    fn description(&self) -> &'static str {
        "Create branches from stashes or manage stash cleanup"
    }

    fn is_destructive(&self) -> bool {
        true
    }
}

fn run_stash_branch(action: StashBranchAction) -> Result<()> {
    match action {
        StashBranchAction::Create {
            branch_name,
            stash_ref,
        } => create_branch_from_stash(branch_name, stash_ref),
        StashBranchAction::Clean {
            older_than,
            dry_run,
        } => clean_old_stashes(older_than, dry_run),
        StashBranchAction::ApplyByBranch {
            branch_name,
            list_only,
        } => apply_stashes_by_branch(branch_name, list_only),
    }
}

fn create_branch_from_stash(branch_name: String, stash_ref: Option<String>) -> Result<()> {
    // Validate branch name
    validate_branch_name(&branch_name).map_err(|e| GitXError::GitCommand(e.to_string()))?;

    // Check if branch already exists
    if BranchOperations::exists(&branch_name).unwrap_or(false) {
        return Err(GitXError::GitCommand(format!(
            "Branch '{branch_name}' already exists"
        )));
    }

    // Determine stash reference
    let stash = stash_ref.unwrap_or_else(|| "stash@{0}".to_string());

    // Validate stash exists
    validate_stash_exists(&stash).map_err(|e| GitXError::GitCommand(e.to_string()))?;

    println!("{}", &branch_name);

    // Create branch from stash
    create_branch_from_stash_ref(&branch_name, &stash)
        .map_err(|e| GitXError::GitCommand(e.to_string()))?;

    println!("{}", &branch_name);
    Ok(())
}

fn clean_old_stashes(older_than: Option<String>, dry_run: bool) -> Result<()> {
    // Get all stashes with timestamps
    let stashes = get_stash_list_with_dates().map_err(|e| GitXError::GitCommand(e.to_string()))?;

    if stashes.is_empty() {
        println!("ℹ️ No stashes found");
        return Ok(());
    }

    // Filter stashes by age if specified
    let stashes_to_clean = if let Some(age) = older_than {
        filter_stashes_by_age(&stashes, &age).map_err(|e| GitXError::GitCommand(e.to_string()))?
    } else {
        stashes
    };

    if stashes_to_clean.is_empty() {
        println!("✅ No old stashes to clean");
        return Ok(());
    }

    let count = stashes_to_clean.len();
    println!(
        "{}",
        if dry_run {
            format!("🧪 (dry run) Would clean {count} stash(es):")
        } else {
            format!("🧹 Cleaning {count} stash(es):")
        }
    );

    for stash in &stashes_to_clean {
        let name = &stash.name;
        let message = &stash.message;
        println!("  {name}: {message}");
    }

    if !dry_run {
        // Safety confirmation for destructive operation
        let stash_names: Vec<_> = stashes_to_clean.iter().map(|s| s.name.as_str()).collect();
        let details = format!(
            "This will delete {} stashes: {}",
            stashes_to_clean.len(),
            stash_names.join(", ")
        );

        match Safety::confirm_destructive_operation("Clean old stashes", &details) {
            Ok(confirmed) => {
                if !confirmed {
                    println!("Operation cancelled by user.");
                    return Ok(());
                }
            }
            Err(e) => {
                return Err(GitXError::GitCommand(format!(
                    "Error during confirmation: {e}"
                )));
            }
        }

        for stash in &stashes_to_clean {
            if let Err(msg) = delete_stash(&stash.name) {
                let msg1 = &format!("Failed to delete {}: {}", stash.name, msg);
                eprintln!("{msg1}");
            }
        }
        println!("{}", stashes_to_clean.len());
    }
    Ok(())
}

fn apply_stashes_by_branch(branch_name: String, list_only: bool) -> Result<()> {
    // Get all stashes with their branch information
    let stashes =
        get_stash_list_with_branches().map_err(|e| GitXError::GitCommand(e.to_string()))?;

    // Filter stashes by branch
    let branch_stashes: Vec<_> = stashes
        .into_iter()
        .filter(|s| s.branch == branch_name)
        .collect();

    if branch_stashes.is_empty() {
        println!("{}", &branch_name);
        return Ok(());
    }

    if list_only {
        println!(
            "{}",
            format_stashes_for_branch_header(&branch_name, branch_stashes.len())
        );
        for stash in &branch_stashes {
            let name = &stash.name;
            let message = &stash.message;
            println!("  {name}: {message}");
        }
    } else {
        println!(
            "{}",
            format_applying_stashes_message(&branch_name, branch_stashes.len())
        );

        for stash in &branch_stashes {
            match apply_stash(&stash.name) {
                Ok(()) => println!("  ✅ Applied {}", stash.name),
                Err(msg) => eprintln!("  ❌ Failed to apply {}: {}", stash.name, msg),
            }
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub struct StashInfo {
    pub name: String,
    pub message: String,
    pub branch: String,
    #[allow(dead_code)]
    pub timestamp: String,
}

pub fn validate_branch_name(name: &str) -> Result<()> {
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

    Ok(())
}

pub fn validate_stash_exists(stash_ref: &str) -> Result<()> {
    match GitOperations::run(&["rev-parse", "--verify", stash_ref]) {
        Ok(_) => Ok(()),
        Err(_) => Err(GitXError::GitCommand(
            "Stash reference does not exist".to_string(),
        )),
    }
}

fn create_branch_from_stash_ref(branch_name: &str, stash_ref: &str) -> Result<()> {
    let status = StdCommand::new("git")
        .args(["stash", "branch", branch_name, stash_ref])
        .status()
        .map_err(GitXError::Io)?;

    if !status.success() {
        return Err(GitXError::GitCommand(
            "Failed to create branch from stash".to_string(),
        ));
    }

    Ok(())
}

fn get_stash_list_with_dates() -> Result<Vec<StashInfo>> {
    let output = StdCommand::new("git")
        .args(["stash", "list", "--pretty=format:%gd|%s|%gD"])
        .output()
        .map_err(GitXError::Io)?;

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

fn get_stash_list_with_branches() -> Result<Vec<StashInfo>> {
    let output = StdCommand::new("git")
        .args(["stash", "list", "--pretty=format:%gd|%s"])
        .output()
        .map_err(GitXError::Io)?;

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

pub fn filter_stashes_by_age(stashes: &[StashInfo], age: &str) -> Result<Vec<StashInfo>> {
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

fn delete_stash(stash_name: &str) -> Result<()> {
    let status = StdCommand::new("git")
        .args(["stash", "drop", stash_name])
        .status()
        .map_err(GitXError::Io)?;

    if !status.success() {
        return Err(GitXError::GitCommand("Failed to delete stash".to_string()));
    }

    Ok(())
}

fn apply_stash(stash_name: &str) -> Result<()> {
    let status = StdCommand::new("git")
        .args(["stash", "apply", stash_name])
        .status()
        .map_err(GitXError::Io)?;

    if !status.success() {
        return Err(GitXError::GitCommand("Failed to apply stash".to_string()));
    }

    Ok(())
}

fn format_stashes_for_branch_header(branch_name: &str, count: usize) -> String {
    format!("📋 Found {count} stash(es) for branch '{branch_name}':")
}

pub fn format_applying_stashes_message(branch_name: &str, count: usize) -> String {
    format!("🔄 Applying {count} stash(es) from branch '{branch_name}':")
}
