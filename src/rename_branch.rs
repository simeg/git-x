use crate::{GitXError, Result};
use std::process::Command;

pub fn run(new_name: &str) -> Result<String> {
    // Step 1: Get current branch name
    let output = Command::new("git")
        .args(get_current_branch_args())
        .output()
        .map_err(|_| GitXError::GitCommand("Failed to execute git".to_string()))?;

    if !output.status.success() {
        return Err(GitXError::GitCommand(
            "Failed to get current branch name".to_string(),
        ));
    }

    let current_branch = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if is_branch_already_named(&current_branch, new_name) {
        return Ok(format_already_named_message(new_name));
    }

    let mut output = Vec::new();
    output.push(format_rename_start_message(&current_branch, new_name));

    // Step 2: Rename branch locally
    let status = Command::new("git")
        .args(get_local_rename_args(new_name))
        .status()
        .map_err(|_| GitXError::GitCommand("Failed to rename branch".to_string()))?;

    if !status.success() {
        return Err(GitXError::GitCommand(
            "Failed to rename local branch".to_string(),
        ));
    }

    // Step 3: Push the new branch to origin
    let status = Command::new("git")
        .args(get_push_new_branch_args(new_name))
        .status()
        .map_err(|_| GitXError::GitCommand("Failed to push new branch".to_string()))?;

    if !status.success() {
        return Err(GitXError::GitCommand(
            "Failed to push new branch to origin".to_string(),
        ));
    }

    // Step 4: Delete the old branch from origin
    let status = Command::new("git")
        .args(get_delete_old_branch_args(&current_branch))
        .status()
        .map_err(|_| GitXError::GitCommand("Failed to delete old branch".to_string()))?;

    if !status.success() {
        output.push(format_delete_failed_message(&current_branch));
    } else {
        output.push(format_delete_success_message(&current_branch));
    }

    output.push(format_rename_success_message().to_string());
    Ok(output.join("\n"))
}

// Helper function to get current branch args
pub fn get_current_branch_args() -> [&'static str; 3] {
    ["rev-parse", "--abbrev-ref", "HEAD"]
}

// Helper function to check if branch is already named
pub fn is_branch_already_named(current_branch: &str, new_name: &str) -> bool {
    current_branch == new_name
}

// Helper function to get local rename args
pub fn get_local_rename_args(new_name: &str) -> Vec<String> {
    vec!["branch".to_string(), "-m".to_string(), new_name.to_string()]
}

// Helper function to get push new branch args
pub fn get_push_new_branch_args(new_name: &str) -> Vec<String> {
    vec![
        "push".to_string(),
        "-u".to_string(),
        "origin".to_string(),
        new_name.to_string(),
    ]
}

// Helper function to get delete old branch args
pub fn get_delete_old_branch_args(old_branch: &str) -> Vec<String> {
    vec![
        "push".to_string(),
        "origin".to_string(),
        "--delete".to_string(),
        old_branch.to_string(),
    ]
}

// Helper function to format already named message
pub fn format_already_named_message(new_name: &str) -> String {
    format!("Current branch is already named '{new_name}'. Nothing to do.")
}

// Helper function to format rename start message
pub fn format_rename_start_message(current_branch: &str, new_name: &str) -> String {
    format!("Renaming branch '{current_branch}' to '{new_name}'")
}

// Helper function to format delete failed message
pub fn format_delete_failed_message(old_branch: &str) -> String {
    format!("Warning: Failed to delete old branch '{old_branch}' from origin.")
}

// Helper function to format delete success message
pub fn format_delete_success_message(old_branch: &str) -> String {
    format!("Deleted old branch '{old_branch}' from origin.")
}

// Helper function to format rename success message
pub fn format_rename_success_message() -> &'static str {
    "Branch renamed successfully."
}
