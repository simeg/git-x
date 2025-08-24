use crate::core::git::*;
use crate::core::safety::Safety;
use crate::core::traits::*;
use crate::{GitXError, Result};

/// Stash-related commands grouped together
pub struct StashCommands;

impl StashCommands {
    /// Create a branch from a stash
    pub fn create_branch(branch_name: String, stash_ref: Option<String>) -> Result<String> {
        StashCommand::new(StashBranchAction::Create {
            branch_name,
            stash_ref,
        })
        .execute()
    }

    /// Clean old stashes
    pub fn clean(older_than: Option<String>, dry_run: bool) -> Result<String> {
        StashCommand::new(StashBranchAction::Clean {
            older_than,
            dry_run,
        })
        .execute()
    }

    /// Apply stashes by branch
    pub fn apply_by_branch(branch_name: String, list_only: bool) -> Result<String> {
        StashCommand::new(StashBranchAction::ApplyByBranch {
            branch_name,
            list_only,
        })
        .execute()
    }

    /// Interactive stash management
    pub fn interactive() -> Result<String> {
        StashCommand::new(StashBranchAction::Interactive).execute()
    }

    /// Export stashes to patch files
    pub fn export(output_dir: String, stash_ref: Option<String>) -> Result<String> {
        StashCommand::new(StashBranchAction::Export {
            output_dir,
            stash_ref,
        })
        .execute()
    }
}

/// Stash branch actions
#[derive(Debug, Clone)]
pub enum StashBranchAction {
    Create {
        branch_name: String,
        stash_ref: Option<String>,
    },
    Clean {
        older_than: Option<String>,
        dry_run: bool,
    },
    ApplyByBranch {
        branch_name: String,
        list_only: bool,
    },
    Interactive,
    Export {
        output_dir: String,
        stash_ref: Option<String>,
    },
}

/// Stash information structure
#[derive(Debug, Clone)]
pub struct StashInfo {
    pub name: String,
    pub message: String,
    pub branch: String,
    pub timestamp: String,
}

/// Command for managing stash-branch operations
pub struct StashCommand {
    action: StashBranchAction,
}

impl StashCommand {
    pub fn new(action: StashBranchAction) -> Self {
        Self { action }
    }

    fn execute_action(&self) -> Result<String> {
        match &self.action {
            StashBranchAction::Create {
                branch_name,
                stash_ref,
            } => self.create_branch_from_stash(branch_name, stash_ref),
            StashBranchAction::Clean {
                older_than,
                dry_run,
            } => self.clean_old_stashes(older_than, *dry_run),
            StashBranchAction::ApplyByBranch {
                branch_name,
                list_only,
            } => self.apply_stashes_by_branch(branch_name, *list_only),
            StashBranchAction::Interactive => self.interactive_stash_management(),
            StashBranchAction::Export {
                output_dir,
                stash_ref,
            } => self.export_stashes_to_patches(output_dir, stash_ref),
        }
    }

    fn create_branch_from_stash(
        &self,
        branch_name: &str,
        stash_ref: &Option<String>,
    ) -> Result<String> {
        // Validate branch name
        self.validate_branch_name(branch_name)?;

        // Check if branch already exists
        if BranchOperations::exists(branch_name).unwrap_or(false) {
            return Err(GitXError::GitCommand(format!(
                "Branch '{branch_name}' already exists"
            )));
        }

        // Determine stash reference
        let stash = stash_ref.clone().unwrap_or_else(|| "stash@{0}".to_string());

        // Validate stash exists
        self.validate_stash_exists(&stash)?;

        // Create branch from stash
        GitOperations::run_status(&["stash", "branch", branch_name, &stash])?;

        Ok(format!(
            "✅ Created branch '{branch_name}' from stash '{stash}'"
        ))
    }

    fn clean_old_stashes(&self, older_than: &Option<String>, dry_run: bool) -> Result<String> {
        // Get all stashes with timestamps
        let stashes = self.get_stash_list_with_dates()?;

        if stashes.is_empty() {
            return Ok("ℹ️ No stashes found".to_string());
        }

        // Filter stashes by age if specified
        let stashes_to_clean = if let Some(age) = older_than {
            self.filter_stashes_by_age(&stashes, age)?
        } else {
            stashes
        };

        if stashes_to_clean.is_empty() {
            return Ok("✅ No old stashes to clean".to_string());
        }

        let count = stashes_to_clean.len();
        let mut result = if dry_run {
            format!("🧪 (dry run) Would clean {count} stash(es):\n")
        } else {
            format!("🧹 Cleaning {count} stash(es):\n")
        };

        for stash in &stashes_to_clean {
            result.push_str(&format!("  {}: {}\n", stash.name, stash.message));
        }

        if !dry_run {
            // Safety confirmation for destructive operation
            let stash_names: Vec<_> = stashes_to_clean.iter().map(|s| s.name.as_str()).collect();
            let details = format!(
                "This will delete {} stashes: {}",
                stashes_to_clean.len(),
                stash_names.join(", ")
            );

            let confirmed = Safety::confirm_destructive_operation("Clean old stashes", &details)?;
            if !confirmed {
                return Ok("Operation cancelled by user.".to_string());
            }

            let mut deleted_count = 0;
            for stash in &stashes_to_clean {
                match self.delete_stash(&stash.name) {
                    Ok(()) => deleted_count += 1,
                    Err(e) => {
                        result.push_str(&format!("❌ Failed to delete {}: {}\n", stash.name, e));
                    }
                }
            }
            result.push_str(&format!("✅ Successfully deleted {deleted_count} stashes"));
        }

        Ok(result)
    }

    fn apply_stashes_by_branch(&self, branch_name: &str, list_only: bool) -> Result<String> {
        // Get all stashes with their branch information
        let stashes = self.get_stash_list_with_branches()?;

        // Filter stashes by branch
        let branch_stashes: Vec<_> = stashes
            .into_iter()
            .filter(|s| s.branch == branch_name)
            .collect();

        if branch_stashes.is_empty() {
            return Ok(format!("No stashes found for branch '{branch_name}'"));
        }

        let count = branch_stashes.len();
        let mut result = if list_only {
            format!("📋 Found {count} stash(es) for branch '{branch_name}':\n")
        } else {
            format!("🔄 Applying {count} stash(es) from branch '{branch_name}':\n")
        };

        for stash in &branch_stashes {
            if list_only {
                result.push_str(&format!("  {}: {}\n", stash.name, stash.message));
            } else {
                match self.apply_stash(&stash.name) {
                    Ok(()) => result.push_str(&format!("  ✅ Applied {}\n", stash.name)),
                    Err(e) => {
                        result.push_str(&format!("  ❌ Failed to apply {}: {}\n", stash.name, e))
                    }
                }
            }
        }

        Ok(result)
    }

    fn interactive_stash_management(&self) -> Result<String> {
        use dialoguer::{MultiSelect, Select, theme::ColorfulTheme};

        // Get all stashes
        let stashes = self.get_stash_list_with_branches()?;

        if stashes.is_empty() {
            return Ok("📝 No stashes found".to_string());
        }

        // Create display items for selection
        let stash_display: Vec<String> = stashes
            .iter()
            .map(|s| format!("{}: {} (from {})", s.name, s.message, s.branch))
            .collect();

        // Action selection menu
        let actions = vec![
            "Apply selected stash",
            "Delete selected stashes",
            "Create branch from stash",
            "Show stash diff",
            "List all stashes",
            "Exit",
        ];

        let theme = ColorfulTheme::default();
        let action_selection = Select::with_theme(&theme)
            .with_prompt("📋 What would you like to do?")
            .items(&actions)
            .default(0)
            .interact();

        match action_selection {
            Ok(0) => {
                // Apply selected stash
                let selection = Select::with_theme(&theme)
                    .with_prompt("🎯 Select stash to apply")
                    .items(&stash_display)
                    .interact()?;

                self.apply_stash(&stashes[selection].name)?;
                Ok(format!("✅ Applied stash: {}", stashes[selection].name))
            }
            Ok(1) => {
                // Delete selected stashes
                let selections = MultiSelect::with_theme(&theme)
                    .with_prompt(
                        "🗑️ Select stashes to delete (use Space to select, Enter to confirm)",
                    )
                    .items(&stash_display)
                    .interact()?;

                if selections.is_empty() {
                    return Ok("No stashes selected for deletion".to_string());
                }

                let mut deleted_count = 0;
                for &idx in selections.iter().rev() {
                    // Delete in reverse order to maintain indices
                    if self.delete_stash(&stashes[idx].name).is_ok() {
                        deleted_count += 1;
                    }
                }

                Ok(format!("✅ Deleted {deleted_count} stash(es)"))
            }
            Ok(2) => {
                // Create branch from stash
                let selection = Select::with_theme(&theme)
                    .with_prompt("🌱 Select stash to create branch from")
                    .items(&stash_display)
                    .interact()?;

                let branch_name = dialoguer::Input::<String>::with_theme(&theme)
                    .with_prompt("🌿 Enter new branch name")
                    .interact()?;

                self.validate_branch_name(&branch_name)?;

                GitOperations::run_status(&[
                    "stash",
                    "branch",
                    &branch_name,
                    &stashes[selection].name,
                ])?;

                Ok(format!(
                    "✅ Created branch '{}' from stash '{}'",
                    branch_name, stashes[selection].name
                ))
            }
            Ok(3) => {
                // Show stash diff
                let selection = Select::with_theme(&theme)
                    .with_prompt("🔍 Select stash to view diff")
                    .items(&stash_display)
                    .interact()?;

                let diff = GitOperations::run(&["stash", "show", "-p", &stashes[selection].name])?;
                Ok(format!(
                    "📊 Diff for {}:\n{}",
                    stashes[selection].name, diff
                ))
            }
            Ok(4) => {
                // List all stashes
                let mut result = "📝 All stashes:\n".to_string();
                for stash in &stashes {
                    result.push_str(&format!(
                        "  {}: {} (from {})\n",
                        stash.name, stash.message, stash.branch
                    ));
                }
                Ok(result)
            }
            Ok(_) | Err(_) => Ok("👋 Goodbye!".to_string()),
        }
    }

    fn export_stashes_to_patches(
        &self,
        output_dir: &str,
        stash_ref: &Option<String>,
    ) -> Result<String> {
        use std::fs;
        use std::path::Path;

        // Create output directory if it doesn't exist
        let output_path = Path::new(output_dir);
        if !output_path.exists() {
            fs::create_dir_all(output_path)
                .map_err(|e| GitXError::GitCommand(format!("Failed to create directory: {e}")))?;
        }

        let stashes = if let Some(specific_stash) = stash_ref {
            // Export only the specific stash
            self.validate_stash_exists(specific_stash)?;
            vec![self.get_stash_info(specific_stash)?]
        } else {
            // Export all stashes
            self.get_stash_list_with_branches()?
        };

        if stashes.is_empty() {
            return Ok("📝 No stashes to export".to_string());
        }

        let mut exported_count = 0;
        for stash in &stashes {
            // Generate patch content
            let patch_content = GitOperations::run(&["stash", "show", "-p", &stash.name])?;

            // Generate filename (sanitize stash name)
            let safe_name = stash.name.replace(['@', '{', '}'], "");
            let filename = format!("{safe_name}.patch");
            let file_path = output_path.join(filename);

            // Write patch file
            fs::write(&file_path, patch_content)
                .map_err(|e| GitXError::GitCommand(format!("Failed to write patch file: {e}")))?;

            exported_count += 1;
        }

        Ok(format!(
            "✅ Exported {exported_count} stash(es) to patch files in '{output_dir}'"
        ))
    }

    fn get_stash_info(&self, stash_ref: &str) -> Result<StashInfo> {
        let output = GitOperations::run(&["stash", "list", "--pretty=format:%gd|%s", stash_ref])?;

        if let Some(line) = output.lines().next()
            && let Some(stash) = self.parse_stash_line_with_branch(line)
        {
            return Ok(stash);
        }

        Err(GitXError::GitCommand(
            "Could not get stash information".to_string(),
        ))
    }

    // Helper methods
    fn validate_branch_name(&self, name: &str) -> Result<()> {
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

        Ok(())
    }

    fn validate_stash_exists(&self, stash_ref: &str) -> Result<()> {
        match GitOperations::run(&["rev-parse", "--verify", stash_ref]) {
            Ok(_) => Ok(()),
            Err(_) => Err(GitXError::GitCommand(
                "Stash reference does not exist".to_string(),
            )),
        }
    }

    fn get_stash_list_with_dates(&self) -> Result<Vec<StashInfo>> {
        let output = GitOperations::run(&["stash", "list", "--pretty=format:%gd|%s|%gD"])?;

        let mut stashes = Vec::new();
        for line in output.lines() {
            if let Some(stash) = self.parse_stash_line_with_date(line) {
                stashes.push(stash);
            }
        }

        Ok(stashes)
    }

    fn get_stash_list_with_branches(&self) -> Result<Vec<StashInfo>> {
        let output = GitOperations::run(&["stash", "list", "--pretty=format:%gd|%s"])?;

        let mut stashes = Vec::new();
        for line in output.lines() {
            if let Some(stash) = self.parse_stash_line_with_branch(line) {
                stashes.push(stash);
            }
        }

        Ok(stashes)
    }

    fn parse_stash_line_with_date(&self, line: &str) -> Option<StashInfo> {
        let parts: Vec<&str> = line.splitn(3, '|').collect();
        if parts.len() != 3 {
            return None;
        }

        Some(StashInfo {
            name: parts[0].to_string(),
            message: parts[1].to_string(),
            branch: self.extract_branch_from_message(parts[1]),
            timestamp: parts[2].to_string(),
        })
    }

    fn parse_stash_line_with_branch(&self, line: &str) -> Option<StashInfo> {
        let parts: Vec<&str> = line.splitn(2, '|').collect();
        if parts.len() != 2 {
            return None;
        }

        Some(StashInfo {
            name: parts[0].to_string(),
            message: parts[1].to_string(),
            branch: self.extract_branch_from_message(parts[1]),
            timestamp: String::new(),
        })
    }

    fn extract_branch_from_message(&self, message: &str) -> String {
        // Stash messages typically start with "On branch_name:" or "WIP on branch_name:"
        if let Some(start) = message.find("On ") {
            let rest = &message[start + 3..];
            if let Some(end) = rest.find(':') {
                return rest[..end].to_string();
            }
        }

        if let Some(start) = message.find("WIP on ") {
            let rest = &message[start + 7..];
            if let Some(end) = rest.find(':') {
                return rest[..end].to_string();
            }
        }

        "unknown".to_string()
    }

    fn filter_stashes_by_age(&self, stashes: &[StashInfo], age: &str) -> Result<Vec<StashInfo>> {
        // For simplicity, we'll implement basic age filtering
        // In a real implementation, you'd parse the age string and compare timestamps
        if age.ends_with('d') || age.ends_with('w') || age.ends_with('m') {
            // This is a placeholder - real implementation would parse timestamps
            Ok(stashes.to_vec())
        } else {
            Err(GitXError::GitCommand(
                "Invalid age format. Use format like '7d', '2w', '1m'".to_string(),
            ))
        }
    }

    fn delete_stash(&self, stash_name: &str) -> Result<()> {
        GitOperations::run_status(&["stash", "drop", stash_name])
    }

    fn apply_stash(&self, stash_name: &str) -> Result<()> {
        GitOperations::run_status(&["stash", "apply", stash_name])
    }
}

impl Command for StashCommand {
    fn execute(&self) -> Result<String> {
        self.execute_action()
    }

    fn name(&self) -> &'static str {
        "stash-branch"
    }

    fn description(&self) -> &'static str {
        "Create branches from stashes or manage stash cleanup"
    }
}

impl GitCommand for StashCommand {}

impl Destructive for StashCommand {
    fn destruction_description(&self) -> String {
        match &self.action {
            StashBranchAction::Create { branch_name, .. } => {
                format!("This will create a new branch '{branch_name}' and remove the stash")
            }
            StashBranchAction::Clean { dry_run: true, .. } => {
                "This is a dry run - no stashes will be deleted".to_string()
            }
            StashBranchAction::Clean { dry_run: false, .. } => {
                "This will permanently delete the selected stashes".to_string()
            }
            StashBranchAction::ApplyByBranch {
                list_only: true, ..
            } => "This will only list stashes without applying them".to_string(),
            StashBranchAction::ApplyByBranch {
                list_only: false, ..
            } => "This will apply stashes to your working directory".to_string(),
            StashBranchAction::Interactive => {
                "Interactive stash management - actions will be confirmed individually".to_string()
            }
            StashBranchAction::Export { .. } => {
                "This will export stashes as patch files to the specified directory".to_string()
            }
        }
    }
}

// Public utility functions for testing and external use
pub mod utils {
    use super::StashInfo;
    use crate::core::git::GitOperations;
    use crate::{GitXError, Result};

    pub fn validate_branch_name(name: &str) -> Result<()> {
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

        Ok(())
    }

    pub fn validate_stash_exists(stash_ref: &str) -> Result<()> {
        match GitOperations::run(&["rev-parse", "--verify", stash_ref]) {
            Ok(_) => Ok(()),
            Err(_) => Err(GitXError::GitCommand(
                "Stash reference does not exist".to_string(),
            )),
        }
    }

    pub fn parse_stash_line_with_date(line: &str) -> Option<StashInfo> {
        let parts: Vec<&str> = line.splitn(3, '|').collect();
        if parts.len() != 3 {
            return None;
        }

        Some(StashInfo {
            name: parts[0].to_string(),
            message: parts[1].to_string(),
            branch: extract_branch_from_message(parts[1]),
            timestamp: parts[2].to_string(),
        })
    }

    pub fn parse_stash_line_with_branch(line: &str) -> Option<StashInfo> {
        let parts: Vec<&str> = line.splitn(2, '|').collect();
        if parts.len() != 2 {
            return None;
        }

        Some(StashInfo {
            name: parts[0].to_string(),
            message: parts[1].to_string(),
            branch: extract_branch_from_message(parts[1]),
            timestamp: String::new(),
        })
    }

    pub fn extract_branch_from_message(message: &str) -> String {
        // Stash messages typically start with "On branch_name:" or "WIP on branch_name:"
        if let Some(start) = message.find("On ") {
            let rest = &message[start + 3..];
            if let Some(end) = rest.find(':') {
                return rest[..end].to_string();
            }
        }

        if let Some(start) = message.find("WIP on ") {
            let rest = &message[start + 7..];
            if let Some(end) = rest.find(':') {
                return rest[..end].to_string();
            }
        }

        "unknown".to_string()
    }

    pub fn filter_stashes_by_age(stashes: &[StashInfo], age: &str) -> Result<Vec<StashInfo>> {
        // For simplicity, we'll implement basic age filtering
        // In a real implementation, you'd parse the age string and compare timestamps
        if age.ends_with('d') || age.ends_with('w') || age.ends_with('m') {
            // This is a placeholder - real implementation would parse timestamps
            Ok(stashes.to_vec())
        } else {
            Err(GitXError::GitCommand(
                "Invalid age format. Use format like '7d', '2w', '1m'".to_string(),
            ))
        }
    }

    pub fn format_applying_stashes_message(branch_name: &str, count: usize) -> String {
        format!("🔄 Applying {count} stash(es) from branch '{branch_name}':")
    }
}
