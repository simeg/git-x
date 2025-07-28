use crate::{GitXError, Result};
use std::process::Command;

pub fn run(commit_hash: String, rebase: bool) -> Result<String> {
    // Validate the commit hash exists
    validate_commit_hash_result(&commit_hash)?;

    // Get current staged and unstaged changes
    let has_changes = check_for_changes_result()?;

    if !has_changes {
        return Err(GitXError::GitCommand(
            "No staged changes found. Please stage your changes first with 'git add'".to_string(),
        ));
    }

    // Get the short commit hash for better UX
    let short_hash = get_short_commit_hash_result(&commit_hash)?;

    let mut output = Vec::new();
    output.push(format_creating_fixup_message(&short_hash));

    // Create the fixup commit
    create_fixup_commit_result(&commit_hash)?;

    output.push(format_fixup_created_message(&short_hash));

    // Optionally run interactive rebase with autosquash
    if rebase {
        output.push(format_starting_rebase_message().to_string());
        match run_autosquash_rebase_result(&commit_hash) {
            Ok(_) => output.push(format_rebase_success_message().to_string()),
            Err(_) => {
                output
                    .push("Rebase failed. You may need to handle conflicts manually.".to_string());
                output.push(format_manual_rebase_hint(&commit_hash));
            }
        }
    } else {
        output.push(format_manual_rebase_hint(&commit_hash));
    }

    Ok(output.join("\n"))
}

// Helper function to validate commit hash exists (new version)
pub fn validate_commit_hash_result(commit_hash: &str) -> Result<()> {
    let output = Command::new("git")
        .args([
            "rev-parse",
            "--verify",
            &format!("{commit_hash}^{{commit}}"),
        ])
        .output()
        .map_err(|_| GitXError::GitCommand("Failed to validate commit hash".to_string()))?;

    if !output.status.success() {
        return Err(GitXError::GitCommand(
            "Commit hash does not exist".to_string(),
        ));
    }

    Ok(())
}

// Helper function to check for changes to commit (new version)
pub fn check_for_changes_result() -> Result<bool> {
    let output = Command::new("git")
        .args(["diff", "--cached", "--quiet"])
        .status()
        .map_err(|_| GitXError::GitCommand("Failed to check for staged changes".to_string()))?;

    // If staged changes exist, we're good
    if !output.success() {
        return Ok(true);
    }

    // Check for unstaged changes
    let output = Command::new("git")
        .args(["diff", "--quiet"])
        .status()
        .map_err(|_| GitXError::GitCommand("Failed to check for unstaged changes".to_string()))?;

    // If unstaged changes exist, we need to stage them
    if !output.success() {
        return Err(GitXError::GitCommand(
            "You have unstaged changes. Please stage them first with 'git add'".to_string(),
        ));
    }

    Ok(false)
}

// Helper function to get short commit hash (new version)
pub fn get_short_commit_hash_result(commit_hash: &str) -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--short", commit_hash])
        .output()
        .map_err(|_| GitXError::GitCommand("Failed to get short commit hash".to_string()))?;

    if !output.status.success() {
        return Err(GitXError::GitCommand(
            "Failed to resolve commit hash".to_string(),
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

// Helper function to create fixup commit (new version)
fn create_fixup_commit_result(commit_hash: &str) -> Result<()> {
    let status = Command::new("git")
        .args(["commit", &format!("--fixup={commit_hash}")])
        .status()
        .map_err(|_| GitXError::GitCommand("Failed to create fixup commit".to_string()))?;

    if !status.success() {
        return Err(GitXError::GitCommand(
            "Failed to create fixup commit".to_string(),
        ));
    }

    Ok(())
}

// Helper function to run autosquash rebase (new version)
fn run_autosquash_rebase_result(commit_hash: &str) -> Result<()> {
    // Find the parent of the target commit for rebase
    let output = Command::new("git")
        .args(["rev-parse", &format!("{commit_hash}^")])
        .output()
        .map_err(|_| GitXError::GitCommand("Failed to find parent commit".to_string()))?;

    if !output.status.success() {
        return Err(GitXError::GitCommand(
            "Cannot rebase - commit has no parent".to_string(),
        ));
    }

    let parent_hash_string = String::from_utf8_lossy(&output.stdout);
    let parent_hash = parent_hash_string.trim();

    let status = Command::new("git")
        .args(["rebase", "-i", "--autosquash", parent_hash])
        .status()
        .map_err(|_| GitXError::GitCommand("Failed to start interactive rebase".to_string()))?;

    if !status.success() {
        return Err(GitXError::GitCommand(
            "Interactive rebase failed".to_string(),
        ));
    }

    Ok(())
}

// Helper function to get git commit args for fixup
pub fn get_git_fixup_args() -> [&'static str; 2] {
    ["commit", "--fixup"]
}

// Helper function to get git rebase args
pub fn get_git_rebase_args() -> [&'static str; 3] {
    ["rebase", "-i", "--autosquash"]
}

// Helper function to format error message
pub fn format_error_message(msg: &str) -> String {
    format!("❌ {msg}")
}

// Helper function to format no changes message
pub fn format_no_changes_message() -> &'static str {
    "❌ No staged changes found. Please stage your changes first with 'git add'"
}

// Helper function to format creating fixup message
pub fn format_creating_fixup_message(short_hash: &str) -> String {
    format!("🔧 Creating fixup commit for {short_hash}...")
}

// Helper function to format fixup created message
pub fn format_fixup_created_message(short_hash: &str) -> String {
    format!("✅ Fixup commit created for {short_hash}")
}

// Helper function to format starting rebase message
pub fn format_starting_rebase_message() -> &'static str {
    "🔄 Starting interactive rebase with autosquash..."
}

// Helper function to format rebase success message
pub fn format_rebase_success_message() -> &'static str {
    "✅ Interactive rebase completed successfully"
}

// Helper function to format manual rebase hint
pub fn format_manual_rebase_hint(commit_hash: &str) -> String {
    format!("💡 To squash the fixup commit, run: git rebase -i --autosquash {commit_hash}^")
}

// Helper function to check if commit hash is valid format
pub fn is_valid_commit_hash_format(hash: &str) -> bool {
    if hash.is_empty() {
        return false;
    }

    // Must be 4-40 characters long (short to full hash)
    if hash.len() < 4 || hash.len() > 40 {
        return false;
    }

    // Must contain only hex characters
    hash.chars().all(|c| c.is_ascii_hexdigit())
}

// Helper function to format commit validation rules
pub fn get_commit_hash_validation_rules() -> &'static [&'static str] {
    &[
        "Must be 4-40 characters long",
        "Must contain only hex characters (0-9, a-f)",
        "Must reference an existing commit",
    ]
}
