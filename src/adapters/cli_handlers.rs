use crate::adapters::formatters::*;
use crate::core::interactive::Interactive;
use crate::domain::{
    BranchManager, CleanBranchesRequest, CreateBranchRequest, GitRepository, RecentBranchesRequest,
    SwitchBranchRequest,
};
use crate::{GitXError, Result};

/// CLI handler for branch operations
pub struct BranchCliHandler {
    branch_manager: BranchManager,
    formatter: BranchFormatter,
}

impl BranchCliHandler {
    /// Create a new CLI handler
    pub fn new() -> Result<Self> {
        let repository = GitRepository::open()?;
        let branch_manager = BranchManager::new(repository);
        let formatter = BranchFormatter::new();

        Ok(Self {
            branch_manager,
            formatter,
        })
    }

    /// Handle new branch creation command
    pub fn handle_new_branch(&self, name: String, from: Option<String>) -> Result<String> {
        let request = CreateBranchRequest {
            name,
            from,
            create_backup: false, // CLI doesn't expose this option by default
        };

        let result = self.branch_manager.create_branch(request)?;
        Ok(self.formatter.format_creation_result(&result))
    }

    /// Handle clean branches command
    pub fn handle_clean_branches(&self, dry_run: bool) -> Result<String> {
        let request = CleanBranchesRequest {
            dry_run,
            confirm_deletion: !dry_run, // Only confirm for actual deletions
        };

        let result = self.branch_manager.clean_merged_branches(request)?;
        Ok(self.formatter.format_clean_result(&result))
    }

    /// Handle switch recent command
    pub fn handle_switch_recent(&self) -> Result<String> {
        let recent_request = RecentBranchesRequest {
            limit: Some(10),
            exclude_current: true,
            exclude_protected: false,
        };

        let recent_result = self.branch_manager.get_recent_branches(recent_request)?;

        if recent_result.branches.is_empty() {
            return Err(GitXError::GitCommand(
                "No recent branches found".to_string(),
            ));
        }

        let selected_branch = if Interactive::is_interactive() {
            Interactive::branch_picker(
                &recent_result.branches,
                Some("Select a recent branch to switch to"),
            )?
        } else {
            // In non-interactive mode, just switch to the most recent branch
            recent_result.branches[0].clone()
        };

        let switch_request = SwitchBranchRequest {
            branch_name: selected_branch.clone(),
            strict_mode: false,
            create_checkpoint: false,
        };

        let switch_result = self.branch_manager.switch_branch(switch_request)?;
        Ok(self.formatter.format_switch_result(&switch_result))
    }
}

/// CLI handler for repository operations
pub struct RepositoryCliHandler {
    repository: GitRepository,
    formatter: RepositoryFormatter,
}

impl RepositoryCliHandler {
    /// Create a new CLI handler
    pub fn new() -> Result<Self> {
        let repository = GitRepository::open()?;
        let formatter = RepositoryFormatter::new();

        Ok(Self {
            repository,
            formatter,
        })
    }

    /// Handle info command
    pub fn handle_info(&self, detailed: bool) -> Result<String> {
        let info = self.repository.info()?;
        Ok(self.formatter.format_repository_info(&info, detailed))
    }

    /// Handle health command
    pub fn handle_health(&self) -> Result<String> {
        let health = self.repository.health()?;
        Ok(self.formatter.format_health_status(&health))
    }
}

/// Factory for creating CLI handlers
pub struct CliHandlerFactory;

impl CliHandlerFactory {
    /// Create a branch CLI handler
    pub fn create_branch_handler() -> Result<BranchCliHandler> {
        BranchCliHandler::new()
    }

    /// Create a repository CLI handler
    pub fn create_repository_handler() -> Result<RepositoryCliHandler> {
        RepositoryCliHandler::new()
    }
}
