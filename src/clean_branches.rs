use crate::{GitXError, Result};
use std::process::Command;

pub fn run(dry_run: bool) {
    match run_clean_branches(dry_run) {
        Ok(output) => print!("{output}"),
        Err(e) => eprintln!("{}", crate::common::Format::error(&e.to_string())),
    }
}

fn run_clean_branches(dry_run: bool) -> Result<String> {
    let output = Command::new("git")
        .args(get_git_branch_args())
        .output()
        .map_err(|e| GitXError::Io(e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(GitXError::GitCommand(format!("Failed to list merged branches: {}", stderr.trim())));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let branches: Vec<String> = stdout
        .lines()
        .map(clean_branch_name)
        .filter(|branch| !is_protected_branch(branch))
        .collect();

    let mut deleted = Vec::new();
    let mut result = String::new();

    for branch in branches {
        if dry_run {
            result.push_str(&format_dry_run_message(&branch));
            result.push('\n');
            deleted.push(branch);
        } else {
            let delete_args = get_git_delete_args(&branch);
            let status = Command::new("git")
                .args(delete_args)
                .status()
                .map_err(|e| GitXError::Io(e))?;

            if status.success() {
                deleted.push(branch);
            }
        }
    }

    if deleted.is_empty() {
        result.push_str(&format_no_branches_message());
    } else {
        result.push_str(&format_deletion_summary(deleted.len(), dry_run));
        result.push('\n');
        for branch in deleted {
            result.push_str(&format!("  {branch}\n"));
        }
    }

    Ok(result)
}

// Helper function to get git branch args
pub fn get_git_branch_args() -> [&'static str; 2] {
    ["branch", "--merged"]
}

// Helper function to get protected branches
const PROTECTED_BRANCHES: &[&str] = &["main", "master", "develop"];

pub fn get_protected_branches() -> &'static [&'static str] {
    PROTECTED_BRANCHES
}

// Helper function to clean branch name
pub fn clean_branch_name(line: &str) -> String {
    line.trim().trim_start_matches('*').trim().to_string()
}

// Helper function to is_protected_branch
pub fn is_protected_branch(branch: &str) -> bool {
    PROTECTED_BRANCHES.contains(&branch)
}

// Helper function to get git delete args
pub fn get_git_delete_args(branch: &str) -> [&str; 3] {
    ["branch", "-d", branch]
}

// Helper function to format dry run message
pub fn format_dry_run_message(branch: &str) -> String {
    format!("(dry run) Would delete: {branch}")
}

// Helper function to format no branches message
pub fn format_no_branches_message() -> &'static str {
    "No merged branches to delete."
}

// Helper function to format deletion summary
pub fn format_deletion_summary(count: usize, dry_run: bool) -> String {
    if dry_run {
        format!("🧪 (dry run) {count} branches would be deleted:")
    } else {
        format!("🧹 Deleted {count} merged branches:")
    }
}
