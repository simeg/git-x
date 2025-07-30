use crate::core::traits::*;
use crate::core::{git::*, interactive::Interactive, safety::Safety, validation::Validate};
use crate::{GitXError, Result};

/// Branch-related commands grouped together
pub struct BranchCommands;

impl BranchCommands {
    /// Create a new branch command
    pub fn new_branch(name: &str, from: Option<&str>) -> Result<String> {
        use crate::commands::repository::NewBranchCommand;
        NewBranchCommand::new(name.to_string(), from.map(|s| s.to_string())).execute()
    }

    /// Clean merged branches command
    pub fn clean_branches(dry_run: bool) -> Result<String> {
        CleanBranchesCommand::new(dry_run).execute()
    }

    /// Switch to recent branch command
    pub fn switch_recent() -> Result<String> {
        SwitchRecentCommand::new().execute()
    }

    /// Rename current branch command
    pub fn rename_branch(new_name: &str) -> Result<String> {
        RenameBranchCommand::new(new_name.to_string()).execute()
    }

    /// Prune remote branches command
    pub fn prune_branches(dry_run: bool) -> Result<String> {
        PruneBranchesCommand::new(dry_run).execute()
    }

    /// Stash current work into a branch
    pub fn stash_branch(branch_name: &str) -> Result<String> {
        StashBranchCommand::new(branch_name.to_string()).execute()
    }
}

/// Command to clean merged branches
pub struct CleanBranchesCommand {
    dry_run: bool,
}

impl CleanBranchesCommand {
    pub fn new(dry_run: bool) -> Self {
        Self { dry_run }
    }

    fn get_protected_branches() -> Vec<&'static str> {
        vec!["main", "master", "develop"]
    }

    fn is_protected_branch(branch: &str) -> bool {
        Self::get_protected_branches().contains(&branch)
    }
}

impl Command for CleanBranchesCommand {
    fn execute(&self) -> Result<String> {
        let merged_branches = GitOperations::merged_branches()?;
        let current_branch = GitOperations::current_branch()?;

        let branches_to_delete: Vec<String> = merged_branches
            .into_iter()
            .filter(|branch| branch != &current_branch)
            .filter(|branch| !Self::is_protected_branch(branch))
            .collect();

        if branches_to_delete.is_empty() {
            return Ok("No merged branches to delete.".to_string());
        }

        if self.dry_run {
            let mut result = format!(
                "🧪 (dry run) {} branches would be deleted:\n",
                branches_to_delete.len()
            );
            for branch in &branches_to_delete {
                result.push_str(&format!("(dry run) Would delete: {branch}\n"));
            }
            return Ok(result);
        }

        // Confirm deletion
        let details = format!(
            "This will delete {} merged branches: {}",
            branches_to_delete.len(),
            branches_to_delete.join(", ")
        );

        if !Safety::confirm_destructive_operation("Clean merged branches", &details)? {
            return Ok("Operation cancelled by user.".to_string());
        }

        let mut deleted = Vec::new();
        for branch in branches_to_delete {
            if BranchOperations::delete(&branch, false).is_ok() {
                deleted.push(branch);
            }
        }

        Ok(format!(
            "🧹 Deleted {} merged branches:\n{}",
            deleted.len(),
            deleted.join("\n")
        ))
    }

    fn name(&self) -> &'static str {
        "clean-branches"
    }

    fn description(&self) -> &'static str {
        "Delete merged branches"
    }
}

impl GitCommand for CleanBranchesCommand {}
impl DryRunnable for CleanBranchesCommand {
    fn execute_dry_run(&self) -> Result<String> {
        CleanBranchesCommand::new(true).execute()
    }

    fn is_dry_run(&self) -> bool {
        self.dry_run
    }
}

impl Destructive for CleanBranchesCommand {
    fn destruction_description(&self) -> String {
        "This will permanently delete merged branches".to_string()
    }
}

/// Command to switch to a recent branch
pub struct SwitchRecentCommand;

impl Default for SwitchRecentCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl SwitchRecentCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for SwitchRecentCommand {
    fn execute(&self) -> Result<String> {
        let branches = GitOperations::recent_branches(Some(10))?;

        if branches.is_empty() {
            return Err(GitXError::GitCommand(
                "No recent branches found".to_string(),
            ));
        }

        let selected_branch = if Interactive::is_interactive() {
            Interactive::branch_picker(&branches, Some("Select a recent branch to switch to"))?
        } else {
            // In non-interactive mode, just switch to the most recent branch
            branches[0].clone()
        };

        BranchOperations::switch(&selected_branch)?;
        Ok(format!("Switched to branch '{selected_branch}'"))
    }

    fn name(&self) -> &'static str {
        "switch-recent"
    }

    fn description(&self) -> &'static str {
        "Switch to a recently used branch"
    }
}

impl GitCommand for SwitchRecentCommand {}
impl crate::core::traits::Interactive for SwitchRecentCommand {
    fn execute_non_interactive(&self) -> Result<String> {
        let branches = GitOperations::recent_branches(Some(1))?;
        if branches.is_empty() {
            return Err(GitXError::GitCommand(
                "No recent branches found".to_string(),
            ));
        }
        BranchOperations::switch(&branches[0])?;
        Ok(format!("Switched to branch '{}'", branches[0]))
    }
}

/// Command to rename current branch
pub struct RenameBranchCommand {
    new_name: String,
}

impl RenameBranchCommand {
    pub fn new(new_name: String) -> Self {
        Self { new_name }
    }
}

impl Command for RenameBranchCommand {
    fn execute(&self) -> Result<String> {
        Validate::branch_name(&self.new_name)?;

        let current_branch = GitOperations::current_branch()?;

        if BranchOperations::exists(&self.new_name)? {
            return Err(GitXError::GitCommand(format!(
                "Branch '{}' already exists",
                self.new_name
            )));
        }

        BranchOperations::rename(&self.new_name)?;
        Ok(format!(
            "✅ Renamed branch '{}' to '{}'",
            current_branch, self.new_name
        ))
    }

    fn name(&self) -> &'static str {
        "rename-branch"
    }

    fn description(&self) -> &'static str {
        "Rename the current branch"
    }
}

impl GitCommand for RenameBranchCommand {}

/// Command to prune (delete) merged local branches
pub struct PruneBranchesCommand {
    dry_run: bool,
}

impl PruneBranchesCommand {
    pub fn new(dry_run: bool) -> Self {
        Self { dry_run }
    }

    fn get_protected_branches() -> Vec<&'static str> {
        vec!["main", "master", "develop"]
    }

    fn is_protected_branch(branch: &str) -> bool {
        Self::get_protected_branches().contains(&branch)
    }
}

impl Command for PruneBranchesCommand {
    fn execute(&self) -> Result<String> {
        let merged_branches = GitOperations::merged_branches()?;
        let current_branch = GitOperations::current_branch()?;

        let branches_to_delete: Vec<String> = merged_branches
            .into_iter()
            .filter(|branch| branch != &current_branch)
            .filter(|branch| !Self::is_protected_branch(branch))
            .collect();

        if branches_to_delete.is_empty() {
            return Ok("✅ No merged branches to prune.".to_string());
        }

        if self.dry_run {
            let mut result = format!(
                "🧪 (dry run) {} branches would be deleted:\n",
                branches_to_delete.len()
            );
            for branch in &branches_to_delete {
                result.push_str(&format!("(dry run) Would delete: {branch}\n"));
            }
            return Ok(result);
        }

        // Confirm deletion
        let details = format!(
            "This will delete {} merged branches: {}",
            branches_to_delete.len(),
            branches_to_delete.join(", ")
        );

        if !Safety::confirm_destructive_operation("Delete merged branches", &details)? {
            return Ok("Operation cancelled by user.".to_string());
        }

        let mut deleted = Vec::new();
        for branch in branches_to_delete {
            if BranchOperations::delete(&branch, false).is_ok() {
                deleted.push(branch);
            }
        }

        Ok(format!(
            "🧹 Deleted {} merged branches:\n{}",
            deleted.len(),
            deleted.join("\n")
        ))
    }

    fn name(&self) -> &'static str {
        "prune-branches"
    }

    fn description(&self) -> &'static str {
        "Delete merged local branches (except protected ones)"
    }
}

impl GitCommand for PruneBranchesCommand {}
impl DryRunnable for PruneBranchesCommand {
    fn execute_dry_run(&self) -> Result<String> {
        PruneBranchesCommand::new(true).execute()
    }

    fn is_dry_run(&self) -> bool {
        self.dry_run
    }
}

impl Destructive for PruneBranchesCommand {
    fn destruction_description(&self) -> String {
        "This will permanently delete merged branches".to_string()
    }
}

/// Command to stash work into a new branch
pub struct StashBranchCommand {
    branch_name: String,
}

impl StashBranchCommand {
    pub fn new(branch_name: String) -> Self {
        Self { branch_name }
    }
}

impl Command for StashBranchCommand {
    fn execute(&self) -> Result<String> {
        Validate::branch_name(&self.branch_name)?;

        if BranchOperations::exists(&self.branch_name)? {
            return Err(GitXError::GitCommand(format!(
                "Branch '{}' already exists",
                self.branch_name
            )));
        }

        // Create branch from current state
        BranchOperations::create(&self.branch_name, None)?;

        // Reset to clean state
        GitOperations::run_status(&["reset", "--hard", "HEAD"])?;

        Ok(format!(
            "✅ Created branch '{}' with current changes and reset working directory",
            self.branch_name
        ))
    }

    fn name(&self) -> &'static str {
        "stash-branch"
    }

    fn description(&self) -> &'static str {
        "Create a branch with current changes and reset working directory"
    }
}

impl GitCommand for StashBranchCommand {}
impl Destructive for StashBranchCommand {
    fn destruction_description(&self) -> String {
        "This will reset your working directory to a clean state".to_string()
    }
}
