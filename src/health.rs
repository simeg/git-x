use crate::{GitXError, Result};
use console::Style;
use std::process::Command;

pub fn run() -> Result<String> {
    let bold = Style::new().bold();
    let green = Style::new().green().bold();
    let yellow = Style::new().yellow().bold();
    let red = Style::new().red().bold();

    let mut output = Vec::new();

    output.push(format!("{}", bold.apply_to("Repository Health Check")));
    output.push(format!("{}", bold.apply_to("=========================")));
    output.push(String::new());

    // Check if we're in a git repository
    if !is_git_repo(&std::env::current_dir().unwrap_or_else(|_| ".".into())) {
        output.push(format!("{} Not in a Git repository", red.apply_to("✗")));
        return Ok(output.join("\n"));
    }

    // 1. Check repository status
    output.push(check_repo_status(&green, &yellow, &red)?);

    // 2. Check for untracked files
    output.push(check_untracked_files(&green, &yellow, &red)?);

    // 3. Check for stale branches
    output.push(check_stale_branches(&green, &yellow, &red)?);

    // 4. Check repository size
    output.push(check_repo_size(&green, &yellow, &red)?);

    // 5. Check for uncommitted changes
    output.push(check_uncommitted_changes(&green, &yellow, &red)?);

    output.push(String::new());
    output.push(format!("{}", bold.apply_to("Health check complete!")));

    Ok(output.join("\n"))
}

pub fn is_git_repo(path: &std::path::Path) -> bool {
    Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .current_dir(path)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn check_repo_status(green: &Style, _yellow: &Style, red: &Style) -> Result<String> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .map_err(|_| GitXError::GitCommand("Failed to run git status".to_string()))?;

    if !output.status.success() {
        return Err(GitXError::GitCommand(
            "Failed to get repository status".to_string(),
        ));
    }

    let status_output = String::from_utf8_lossy(&output.stdout);

    if status_output.trim().is_empty() {
        Ok(format!(
            "{} Working directory is clean",
            green.apply_to("✓")
        ))
    } else {
        Ok(format!(
            "{} Working directory has changes",
            red.apply_to("✗")
        ))
    }
}

fn check_untracked_files(green: &Style, yellow: &Style, _red: &Style) -> Result<String> {
    let output = Command::new("git")
        .args(["ls-files", "--others", "--exclude-standard"])
        .output()
        .map_err(|_| GitXError::GitCommand("Failed to list untracked files".to_string()))?;

    if !output.status.success() {
        return Err(GitXError::GitCommand(
            "Failed to get untracked files".to_string(),
        ));
    }

    let untracked = String::from_utf8_lossy(&output.stdout);
    let untracked_files: Vec<&str> = untracked.lines().collect();

    if untracked_files.is_empty() {
        Ok(format!("{} No untracked files", green.apply_to("✓")))
    } else {
        Ok(format!(
            "{} {} untracked files found",
            yellow.apply_to("!"),
            untracked_files.len()
        ))
    }
}

fn check_stale_branches(green: &Style, yellow: &Style, _red: &Style) -> Result<String> {
    let output = Command::new("git")
        .args([
            "for-each-ref",
            "--format=%(refname:short) %(committerdate:relative)",
            "refs/heads/",
        ])
        .output()
        .map_err(|_| GitXError::GitCommand("Failed to list branches".to_string()))?;

    if !output.status.success() {
        return Err(GitXError::GitCommand(
            "Failed to get branch information".to_string(),
        ));
    }

    let branches = String::from_utf8_lossy(&output.stdout);
    let mut stale_count = 0;

    for line in branches.lines() {
        if line.contains("months ago") || line.contains("year") {
            stale_count += 1;
        }
    }

    if stale_count == 0 {
        Ok(format!(
            "{} No stale branches (older than 1 month)",
            green.apply_to("✓")
        ))
    } else {
        Ok(format!(
            "{} {} potentially stale branches found",
            yellow.apply_to("!"),
            stale_count
        ))
    }
}

fn check_repo_size(green: &Style, yellow: &Style, red: &Style) -> Result<String> {
    let output = Command::new("du")
        .args(["-sh", ".git"])
        .output()
        .map_err(|_| GitXError::GitCommand("Failed to check repository size".to_string()))?;

    if !output.status.success() {
        return Err(GitXError::GitCommand(
            "Failed to get repository size".to_string(),
        ));
    }

    let size_output = String::from_utf8_lossy(&output.stdout);
    let size = size_output.split_whitespace().next().unwrap_or("unknown");

    // Simple heuristic for repository size warnings
    if size.ends_with('K')
        || (size.ends_with('M') && size.chars().next().unwrap_or('0').to_digit(10).unwrap_or(0) < 5)
    {
        Ok(format!(
            "{} Repository size: {} (healthy)",
            green.apply_to("✓"),
            size
        ))
    } else if size.ends_with('M')
        || (size.ends_with('G') && size.chars().next().unwrap_or('0').to_digit(10).unwrap_or(0) < 1)
    {
        Ok(format!(
            "{} Repository size: {} (moderate)",
            yellow.apply_to("!"),
            size
        ))
    } else {
        Ok(format!(
            "{} Repository size: {} (large - consider cleanup)",
            red.apply_to("✗"),
            size
        ))
    }
}

fn check_uncommitted_changes(green: &Style, yellow: &Style, _red: &Style) -> Result<String> {
    let output = Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .output()
        .map_err(|_| GitXError::GitCommand("Failed to check staged changes".to_string()))?;

    if !output.status.success() {
        return Err(GitXError::GitCommand(
            "Failed to get staged changes".to_string(),
        ));
    }

    let staged = String::from_utf8_lossy(&output.stdout);
    let staged_files: Vec<&str> = staged.lines().filter(|line| !line.is_empty()).collect();

    if staged_files.is_empty() {
        Ok(format!("{} No staged changes", green.apply_to("✓")))
    } else {
        Ok(format!(
            "{} {} files staged for commit",
            yellow.apply_to("!"),
            staged_files.len()
        ))
    }
}
