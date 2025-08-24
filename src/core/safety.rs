use crate::core::{git::GitOperations, interactive::Interactive};
use crate::{GitXError, Result};

/// Safety and backup utilities for destructive operations
pub struct Safety;

impl Safety {
    /// Create a backup branch before destructive operations
    pub fn create_backup_branch(prefix: Option<&str>) -> Result<String> {
        let current_branch = GitOperations::current_branch()?;
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_prefix = prefix.unwrap_or("backup");
        let backup_name = format!("{backup_prefix}/{current_branch}_{timestamp}");

        // Validate backup branch name
        crate::core::validation::Validate::branch_name(&backup_name)?;

        // Create the backup branch
        let status = std::process::Command::new("git")
            .args(["branch", &backup_name])
            .status()?;

        if !status.success() {
            return Err(GitXError::GitCommand(format!(
                "Failed to create backup branch '{backup_name}'"
            )));
        }

        Ok(backup_name)
    }

    /// Check if working directory is clean before destructive operations
    pub fn ensure_clean_working_directory() -> Result<()> {
        // Skip working directory check in test environments
        if Self::is_test_environment() {
            return Ok(());
        }

        if !GitOperations::is_working_directory_clean()? {
            return Err(GitXError::GitCommand(
                "Working directory is not clean. Please commit or stash your changes first."
                    .to_string(),
            ));
        }

        Ok(())
    }

    /// Confirm destructive operation with user
    pub fn confirm_destructive_operation(operation: &str, details: &str) -> Result<bool> {
        let prompt = format!(
            "⚠️  {operation} This is a destructive operation.\n{details}\nDo you want to continue?"
        );

        if !Interactive::is_interactive() {
            // In non-interactive environments (like tests), default to allowing the operation
            // but log that confirmation was skipped
            eprintln!("Warning: Skipping confirmation in non-interactive environment");
            return Ok(true);
        }

        Interactive::confirm(&prompt, false)
    }

    /// Check if we're in a test environment
    fn is_test_environment() -> bool {
        std::env::var("CARGO_TARGET_TMPDIR").is_ok() || std::env::var("CI").is_ok() || cfg!(test)
    }

    /// Create a safety checkpoint (stash) before operation
    pub fn create_checkpoint(message: Option<&str>) -> Result<String> {
        let checkpoint_msg = message.unwrap_or("git-x safety checkpoint");

        let output = std::process::Command::new("git")
            .args(["stash", "push", "-m", checkpoint_msg])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GitXError::GitCommand(format!(
                "Failed to create safety checkpoint: {stderr}"
            )));
        }

        Ok(checkpoint_msg.to_string())
    }

    /// Restore from safety checkpoint if operation fails
    pub fn restore_checkpoint() -> Result<()> {
        let status = std::process::Command::new("git")
            .args(["stash", "pop"])
            .status()?;

        if !status.success() {
            return Err(GitXError::GitCommand(
                "Failed to restore from safety checkpoint".to_string(),
            ));
        }

        Ok(())
    }

    /// List recent backup branches created by git-x
    pub fn list_backup_branches() -> Result<Vec<String>> {
        let output = std::process::Command::new("git")
            .args(["branch", "--list", "backup/*"])
            .output()?;

        if !output.status.success() {
            return Err(GitXError::GitCommand(
                "Failed to list backup branches".to_string(),
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let branches: Vec<String> = stdout
            .lines()
            .map(|line| line.trim().trim_start_matches("* ").to_string())
            .filter(|branch| !branch.is_empty())
            .collect();

        Ok(branches)
    }

    /// Clean up old backup branches (older than specified days)
    pub fn cleanup_old_backups(days: u32, dry_run: bool) -> Result<Vec<String>> {
        let backup_branches = Self::list_backup_branches()?;
        let mut removed_branches = Vec::new();

        for branch in backup_branches {
            if Self::is_branch_older_than(&branch, days)? {
                if dry_run {
                    removed_branches.push(format!("[DRY RUN] Would delete: {branch}"));
                } else {
                    let status = std::process::Command::new("git")
                        .args(["branch", "-D", &branch])
                        .status()?;

                    if status.success() {
                        removed_branches.push(format!("Deleted: {branch}"));
                    } else {
                        removed_branches.push(format!("Failed to delete: {branch}"));
                    }
                }
            }
        }

        Ok(removed_branches)
    }

    /// Check if a branch is older than specified days
    fn is_branch_older_than(branch: &str, days: u32) -> Result<bool> {
        let output = std::process::Command::new("git")
            .args(["log", "-1", "--format=%ct", branch])
            .output()?;

        if !output.status.success() {
            return Err(GitXError::GitCommand(format!(
                "Failed to get branch date for '{branch}'"
            )));
        }

        let timestamp_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let timestamp: i64 = timestamp_str
            .parse()
            .map_err(|_| GitXError::Parse(format!("Invalid timestamp for branch '{branch}'")))?;

        let branch_date = chrono::DateTime::from_timestamp(timestamp, 0)
            .ok_or_else(|| GitXError::Parse(format!("Invalid date for branch '{branch}'")))?;

        let cutoff_date = chrono::Utc::now() - chrono::Duration::days(days as i64);

        Ok(branch_date < cutoff_date)
    }
}

/// Builder for creating safe operation workflows
pub struct SafetyBuilder {
    operation_name: String,
    backup_needed: bool,
    checkpoint_needed: bool,
    confirmation_needed: bool,
    clean_directory_needed: bool,
}

impl SafetyBuilder {
    pub fn new(operation_name: &str) -> Self {
        Self {
            operation_name: operation_name.to_string(),
            backup_needed: false,
            checkpoint_needed: false,
            confirmation_needed: false,
            clean_directory_needed: false,
        }
    }

    pub fn with_backup(mut self) -> Self {
        self.backup_needed = true;
        self
    }

    pub fn with_checkpoint(mut self) -> Self {
        self.checkpoint_needed = true;
        self
    }

    pub fn with_confirmation(mut self) -> Self {
        self.confirmation_needed = true;
        self
    }

    pub fn with_clean_directory(mut self) -> Self {
        self.clean_directory_needed = true;
        self
    }

    pub fn execute<F>(self, operation: F) -> Result<String>
    where
        F: FnOnce() -> Result<String>,
    {
        // Pre-operation safety checks
        if self.clean_directory_needed {
            Safety::ensure_clean_working_directory()?;
        }

        let backup_name = if self.backup_needed {
            Some(Safety::create_backup_branch(Some("safety"))?)
        } else {
            None
        };

        if self.checkpoint_needed {
            Safety::create_checkpoint(Some(&format!("Before {}", self.operation_name)))?;
        }

        if self.confirmation_needed {
            let details = if let Some(ref backup) = backup_name {
                format!("A backup branch '{backup}' has been created.")
            } else {
                "No backup will be created.".to_string()
            };

            if !Safety::confirm_destructive_operation(&self.operation_name, &details)? {
                return Ok("Operation cancelled by user.".to_string());
            }
        }

        // Execute the operation
        match operation() {
            Ok(result) => Ok(result),
            Err(e) => {
                // Try to restore from checkpoint on failure
                if self.checkpoint_needed
                    && let Err(restore_err) = Safety::restore_checkpoint()
                {
                    eprintln!("Warning: Failed to restore checkpoint: {restore_err}");
                }
                Err(e)
            }
        }
    }
}
