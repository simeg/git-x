use crate::{GitXError, Result};
use std::process::Command;

pub fn run(target: Option<String>) -> Result<String> {
    let target_branch = target.unwrap_or_else(get_default_target);

    // Get current branch name
    let current_branch_output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .map_err(|_| GitXError::GitCommand("Failed to get current branch".to_string()))?;

    if !current_branch_output.status.success() {
        return Err(GitXError::GitCommand("Not in a git repository".to_string()));
    }

    let current_branch = String::from_utf8_lossy(&current_branch_output.stdout)
        .trim()
        .to_string();

    let mut output = Vec::new();
    output.push(format_branch_comparison(&current_branch, &target_branch));

    // Get ahead/behind commit counts
    let rev_list_output = Command::new("git")
        .args([
            "rev-list",
            "--left-right",
            "--count",
            &format_rev_list_range(&target_branch, &current_branch),
        ])
        .output()
        .map_err(|_| GitXError::GitCommand("Failed to get ahead/behind count".to_string()))?;

    if !rev_list_output.status.success() {
        return Err(GitXError::GitCommand(
            "Failed to compare branches".to_string(),
        ));
    }

    let output_str = String::from_utf8_lossy(&rev_list_output.stdout);
    let (ahead, behind) = parse_commit_counts(&output_str);

    let (ahead_msg, behind_msg) = format_commit_counts(&ahead, &behind);
    output.push(ahead_msg);
    output.push(behind_msg);

    // Get diff summary
    let diff_output = Command::new("git")
        .args([
            "diff",
            "--name-status",
            &format_rev_list_range(&target_branch, &current_branch),
        ])
        .output()
        .map_err(|_| GitXError::GitCommand("Failed to get diff".to_string()))?;

    if !diff_output.status.success() {
        return Err(GitXError::GitCommand(
            "Failed to get file changes".to_string(),
        ));
    }

    let diff = String::from_utf8_lossy(&diff_output.stdout);

    output.push("Changes:".to_string());
    for line in diff.lines() {
        if let Some(formatted_line) = format_diff_line(line) {
            output.push(formatted_line);
        }
    }

    Ok(output.join("\n"))
}

// Helper function to get default target branch
const DEFAULT_TARGET: &str = "main";

pub fn get_default_target() -> String {
    DEFAULT_TARGET.to_string()
}

// Helper function to format branch comparison header
pub fn format_branch_comparison(current: &str, target: &str) -> String {
    format!("Branch: {current} vs {target}")
}

// Helper function to format commit counts
pub fn format_commit_counts(ahead: &str, behind: &str) -> (String, String) {
    (
        format!("+ {ahead} commits ahead"),
        format!("- {behind} commits behind"),
    )
}

// Helper function to format rev-list range
pub fn format_rev_list_range(target: &str, current: &str) -> String {
    format!("{target}...{current}")
}

// Helper function to parse ahead/behind counts
pub fn parse_commit_counts(output: &str) -> (String, String) {
    let mut counts = output.split_whitespace();
    let behind = counts.next().unwrap_or("0").to_string();
    let ahead = counts.next().unwrap_or("0").to_string();
    (ahead, behind)
}

// Helper function to convert git status to symbol
pub fn git_status_to_symbol(status: &str) -> &str {
    match status {
        "A" => "+",
        "M" => "~",
        "D" => "-",
        other => other,
    }
}

// Helper function to format diff line
pub fn format_diff_line(line: &str) -> Option<String> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 2 {
        let symbol = git_status_to_symbol(parts[0]);
        Some(format!(" - {} {}", symbol, parts[1]))
    } else {
        None
    }
}
