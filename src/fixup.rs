use crate::core::validation::Validate;
use crate::{GitXError, Result};
use std::process::Command;

pub fn run(commit_hash: String, rebase: bool) -> Result<()> {
    // Validate commit hash format first
    Validate::commit_hash(&commit_hash)?;

    // Validate the commit hash exists
    validate_commit_hash(&commit_hash).map_err(|e| GitXError::GitCommand(e.to_string()))?;

    // Get current staged and unstaged changes
    let has_changes = check_for_changes().map_err(|e| GitXError::GitCommand(e.to_string()))?;

    if !has_changes {
        return Err(GitXError::GitCommand(
            "No staged changes found. Please stage your changes first with 'git add'".to_string(),
        ));
    }

    // Get the short commit hash for better UX
    let short_hash =
        get_short_commit_hash(&commit_hash).map_err(|e| GitXError::GitCommand(e.to_string()))?;

    println!("{}", &short_hash);

    // Create the fixup commit
    create_fixup_commit(&commit_hash).map_err(|e| GitXError::GitCommand(e.to_string()))?;

    let short_hash2 = &short_hash;
    println!("{short_hash2}");

    // Optionally run interactive rebase with autosquash
    if rebase {
        println!("🔄 Starting interactive rebase with autosquash...");
        if let Err(msg) = run_autosquash_rebase(&commit_hash) {
            let msg1 = &msg.to_string();
            eprintln!("{msg1}");
            let commit_hash1 = &commit_hash;
            eprintln!("{commit_hash1}");
            return Ok(()); // Don't fail the whole command if rebase fails
        }
        println!("✅ Interactive rebase completed successfully");
    } else {
        let commit_hash1 = &commit_hash;
        println!("{commit_hash1}");
    }
    Ok(())
}

fn validate_commit_hash(commit_hash: &str) -> Result<()> {
    let output = Command::new("git")
        .args([
            "rev-parse",
            "--verify",
            &format!("{commit_hash}^{{commit}}"),
        ])
        .output()
        .map_err(GitXError::Io)?;

    if !output.status.success() {
        return Err(GitXError::GitCommand(
            "Commit hash does not exist".to_string(),
        ));
    }

    Ok(())
}

fn check_for_changes() -> Result<bool> {
    let output = Command::new("git")
        .args(["diff", "--cached", "--quiet"])
        .status()
        .map_err(GitXError::Io)?;

    // If staged changes exist, we're good
    if !output.success() {
        return Ok(true);
    }

    // Check for unstaged changes
    let output = Command::new("git")
        .args(["diff", "--quiet"])
        .status()
        .map_err(GitXError::Io)?;

    // If unstaged changes exist, we need to stage them
    if !output.success() {
        return Err(GitXError::GitCommand(
            "You have unstaged changes. Please stage them first with 'git add'".to_string(),
        ));
    }

    Ok(false)
}

fn get_short_commit_hash(commit_hash: &str) -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--short", commit_hash])
        .output()
        .map_err(GitXError::Io)?;

    if !output.status.success() {
        return Err(GitXError::GitCommand(
            "Failed to resolve commit hash".to_string(),
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn create_fixup_commit(commit_hash: &str) -> Result<()> {
    let status = Command::new("git")
        .args(["commit", &format!("--fixup={commit_hash}")])
        .status()
        .map_err(GitXError::Io)?;

    if !status.success() {
        return Err(GitXError::GitCommand(
            "Failed to create fixup commit".to_string(),
        ));
    }

    Ok(())
}

fn run_autosquash_rebase(commit_hash: &str) -> Result<()> {
    // Find the parent of the target commit for rebase
    let output = Command::new("git")
        .args(["rev-parse", &format!("{commit_hash}^")])
        .output()
        .map_err(GitXError::Io)?;

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
        .map_err(GitXError::Io)?;

    if !status.success() {
        return Err(GitXError::GitCommand(
            "Interactive rebase failed".to_string(),
        ));
    }

    Ok(())
}
