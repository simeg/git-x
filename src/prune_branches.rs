use crate::{GitXError, Result};
use std::io::{BufRead, BufReader};
use std::process::Command;

pub fn run(except: Option<String>) -> Result<String> {
    let protected_branches = get_all_protected_branches(except.as_deref());

    // Step 1: Get current branch
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .map_err(|_| GitXError::GitCommand("Failed to get current branch".to_string()))?;

    if !output.status.success() {
        return Err(GitXError::GitCommand(
            "Could not determine current branch".to_string(),
        ));
    }

    let current_branch = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Step 2: Get merged branches
    let output = Command::new("git")
        .args(["branch", "--merged"])
        .output()
        .map_err(|_| GitXError::GitCommand("Failed to get merged branches".to_string()))?;

    if !output.status.success() {
        return Err(GitXError::GitCommand(
            "Failed to list merged branches".to_string(),
        ));
    }

    let reader = BufReader::new(output.stdout.as_slice());
    let branches: Vec<String> = reader
        .lines()
        .map_while(|line| line.ok())
        .map(|b| clean_git_branch_name(&b))
        .filter(|b| !is_branch_protected(b, &current_branch, &protected_branches))
        .collect();

    if branches.is_empty() {
        return Ok(format_no_branches_to_prune_message().to_string());
    }

    // Step 3: Delete branches
    let mut output = Vec::new();
    for branch in branches {
        let delete_args = get_git_branch_delete_args(&branch);
        let status = Command::new("git")
            .args(delete_args)
            .status()
            .map_err(|_| GitXError::GitCommand("Failed to delete branch".to_string()))?;

        if status.success() {
            output.push(format_branch_deleted_message(&branch));
        } else {
            output.push(format_branch_delete_failed_message(&branch));
        }
    }
    Ok(output.join("\n"))
}

// Helper function to get default protected branches
const DEFAULT_PROTECTED_BRANCHES: &[&str] = &["main", "master", "develop"];

pub fn get_default_protected_branches() -> &'static [&'static str] {
    DEFAULT_PROTECTED_BRANCHES
}

// Helper function to parse except string into vec
pub fn parse_except_branches(except: &str) -> Vec<String> {
    except
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

// Helper function to get all protected branches
pub fn get_all_protected_branches(except: Option<&str>) -> Vec<String> {
    let mut protected: Vec<String> = DEFAULT_PROTECTED_BRANCHES
        .iter()
        .map(|&s| s.to_string())
        .collect();

    if let Some(except_str) = except {
        protected.extend(parse_except_branches(except_str));
    }

    protected
}

// Helper function to clean branch name from git output
pub fn clean_git_branch_name(branch: &str) -> String {
    branch.trim().trim_start_matches("* ").to_string()
}

// Helper function to check if branch should be protected
pub fn is_branch_protected(
    branch: &str,
    current_branch: &str,
    protected_branches: &[String],
) -> bool {
    branch == current_branch || protected_branches.iter().any(|pb| pb == branch)
}

// Helper function to get git branch delete args
pub fn get_git_branch_delete_args(branch: &str) -> [&str; 3] {
    ["branch", "-d", branch]
}

// Helper function to format success message
pub fn format_branch_deleted_message(branch: &str) -> String {
    format!("🧹 Deleted merged branch '{branch}'")
}

// Helper function to format failure message
pub fn format_branch_delete_failed_message(branch: &str) -> String {
    format!("⚠️ Failed to delete branch '{branch}'")
}

// Helper function to format no branches message
pub fn format_no_branches_to_prune_message() -> &'static str {
    "✅ No merged branches to prune."
}
