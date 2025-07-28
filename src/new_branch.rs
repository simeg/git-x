use crate::{GitXError, Result};
use std::process::Command;

pub fn run(branch_name: String, from: Option<String>) -> Result<String> {
    // Validate branch name
    validate_branch_name_result(&branch_name)?;

    // Check if branch already exists
    if branch_exists(&branch_name) {
        return Err(GitXError::GitCommand(format!(
            "Branch '{branch_name}' already exists"
        )));
    }

    // Determine base branch
    let base_branch = match from {
        Some(ref branch) => {
            if !branch_exists(branch) && !is_valid_ref(branch) {
                return Err(GitXError::GitCommand(format!(
                    "Base branch or ref '{branch}' does not exist"
                )));
            }
            branch.clone()
        }
        None => get_current_branch_result()?,
    };

    let mut output = Vec::new();
    output.push(format_creating_branch_message(&branch_name, &base_branch));

    // Create the new branch
    create_branch_result(&branch_name, &base_branch)?;

    // Switch to the new branch
    switch_to_branch_result(&branch_name)?;

    output.push(format_success_message(&branch_name));
    Ok(output.join("\n"))
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

// Helper function to check if branch exists
fn branch_exists(branch_name: &str) -> bool {
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

// Helper function to check if ref is valid
fn is_valid_ref(ref_name: &str) -> bool {
    Command::new("git")
        .args(["rev-parse", "--verify", "--quiet", ref_name])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

// Helper function to get current branch (new version)
fn get_current_branch_result() -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .map_err(|_| GitXError::GitCommand("Failed to get current branch".to_string()))?;

    if !output.status.success() {
        return Err(GitXError::GitCommand("Not in a git repository".to_string()));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

// Helper function to create branch (new version)
fn create_branch_result(branch_name: &str, base_branch: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["branch", branch_name, base_branch])
        .output()
        .map_err(|_| GitXError::GitCommand("Failed to execute git branch command".to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(GitXError::GitCommand(format!(
            "Failed to create branch: {}",
            stderr.trim()
        )));
    }

    Ok(())
}

// Helper function to switch to branch (new version)
fn switch_to_branch_result(branch_name: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["switch", branch_name])
        .output()
        .map_err(|_| GitXError::GitCommand("Failed to execute git switch command".to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(GitXError::GitCommand(format!(
            "Failed to switch to new branch: {}",
            stderr.trim()
        )));
    }

    Ok(())
}

// Helper function to get branch validation error messages
pub fn get_validation_rules() -> &'static [&'static str] {
    &[
        "Cannot be empty",
        "Cannot start with a dash",
        "Cannot contain '..'",
        "Cannot contain spaces",
        "Cannot contain ~^:?*[\\",
    ]
}

// Helper function to format error message
pub fn format_error_message(msg: &str) -> String {
    format!("❌ {msg}")
}

// Helper function to format branch exists message
pub fn format_branch_exists_message(branch_name: &str) -> String {
    format!("❌ Branch '{branch_name}' already exists")
}

// Helper function to format invalid base message
pub fn format_invalid_base_message(base_branch: &str) -> String {
    format!("❌ Base branch or ref '{base_branch}' does not exist")
}

// Helper function to format creating branch message
pub fn format_creating_branch_message(branch_name: &str, base_branch: &str) -> String {
    format!("🌿 Creating branch '{branch_name}' from '{base_branch}'...")
}

// Helper function to format success message
pub fn format_success_message(branch_name: &str) -> String {
    format!("✅ Created and switched to branch '{branch_name}'")
}

// Helper function to get git branch creation args
pub fn get_git_branch_args() -> [&'static str; 2] {
    ["branch", "-"]
}

// Helper function to get git switch args
pub fn get_git_switch_args() -> [&'static str; 2] {
    ["switch", "-"]
}
