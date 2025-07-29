use crate::core::git::GitOperations;
use crate::core::validation::Validate;
use crate::{GitXError, Result};
use std::process::Command;

pub fn run(branch_name: String, from: Option<String>) -> Result<String> {
    // Validate branch name format and safety
    Validate::branch_name(&branch_name)?;

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
    output.push(branch_name.clone());

    // Create the new branch
    create_branch_result(&branch_name, &base_branch)?;

    // Switch to the new branch
    switch_to_branch_result(&branch_name)?;

    output.push(branch_name.clone());
    Ok(output.join("\n"))
}

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

fn is_valid_ref(ref_name: &str) -> bool {
    Command::new("git")
        .args(["rev-parse", "--verify", "--quiet", ref_name])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn get_current_branch_result() -> Result<String> {
    GitOperations::current_branch()
        .map_err(|e| GitXError::GitCommand(format!("Failed to get current branch: {e}")))
}

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
