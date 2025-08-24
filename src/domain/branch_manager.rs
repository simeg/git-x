use crate::core::{git::*, safety::Safety, validation::Validate};
use crate::domain::GitRepository;
use crate::{GitXError, Result};

/// High-level branch management operations
pub struct BranchManager {
    _repository: GitRepository,
}

impl BranchManager {
    /// Create a new branch manager
    pub fn new(repository: GitRepository) -> Self {
        Self {
            _repository: repository,
        }
    }

    /// Create a new branch with validation and safety checks
    pub fn create_branch(&self, request: CreateBranchRequest) -> Result<BranchCreationResult> {
        // Validate inputs
        Validate::branch_name(&request.name)?;

        if let Some(ref base) = request.from
            && !GitOperations::commit_exists(base)?
        {
            return Err(GitXError::GitCommand(format!(
                "Base branch or ref '{base}' does not exist"
            )));
        }

        // Check if branch already exists
        if BranchOperations::exists(&request.name)? {
            return Err(GitXError::GitCommand(format!(
                "Branch '{}' already exists",
                request.name
            )));
        }

        // Create backup if requested
        let backup_branch = if request.create_backup {
            Some(Safety::create_backup_branch(Some("pre-create"))?)
        } else {
            None
        };

        // Create and switch to the branch
        BranchOperations::create(&request.name, request.from.as_deref())?;

        Ok(BranchCreationResult {
            branch_name: request.name,
            base_commit: request.from,
            backup_branch,
            switched: true,
        })
    }

    /// Delete branches with safety checks
    pub fn delete_branches(&self, request: DeleteBranchesRequest) -> Result<BranchDeletionResult> {
        let mut deleted = Vec::new();
        let mut failed = Vec::new();
        let mut protected = Vec::new();

        // Filter out protected branches
        let protected_branches = ["main", "master", "develop"];

        for branch in &request.branches {
            if protected_branches.contains(&branch.as_str()) {
                protected.push(branch.clone());
                continue;
            }

            if request.dry_run {
                deleted.push(branch.clone()); // In dry run, assume all would succeed
            } else {
                match BranchOperations::delete(branch, request.force) {
                    Ok(_) => deleted.push(branch.clone()),
                    Err(_) => failed.push(branch.clone()),
                }
            }
        }

        Ok(BranchDeletionResult {
            deleted,
            failed,
            protected,
            dry_run: request.dry_run,
        })
    }

    /// Get recent branches with filtering
    pub fn get_recent_branches(
        &self,
        request: RecentBranchesRequest,
    ) -> Result<RecentBranchesResult> {
        let all_recent = GitOperations::recent_branches(request.limit)?;

        // Filter out current branch and protected branches if requested
        let current_branch = GitOperations::current_branch()?;
        let protected_branches = ["main", "master", "develop"];

        let filtered_branches = all_recent
            .into_iter()
            .filter(|branch| {
                if request.exclude_current && branch == &current_branch {
                    return false;
                }
                if request.exclude_protected && protected_branches.contains(&branch.as_str()) {
                    return false;
                }
                true
            })
            .collect();

        Ok(RecentBranchesResult {
            branches: filtered_branches,
            current_branch,
        })
    }

    /// Switch to a branch with validation
    pub fn switch_branch(&self, request: SwitchBranchRequest) -> Result<BranchSwitchResult> {
        // Validate branch exists
        if !BranchOperations::exists(&request.branch_name)? {
            return Err(GitXError::GitCommand(format!(
                "Branch '{}' does not exist",
                request.branch_name
            )));
        }

        // Check for uncommitted changes if strict mode
        if request.strict_mode && !GitOperations::is_working_directory_clean()? {
            return Err(GitXError::GitCommand(
                "Working directory has uncommitted changes. Use --force or commit/stash changes."
                    .to_string(),
            ));
        }

        let previous_branch = GitOperations::current_branch()?;

        // Create checkpoint if requested
        let checkpoint = if request.create_checkpoint {
            Some(Safety::create_checkpoint(Some(&format!(
                "Before switching to {}",
                request.branch_name
            )))?)
        } else {
            None
        };

        // Perform the switch
        BranchOperations::switch(&request.branch_name)?;

        Ok(BranchSwitchResult {
            previous_branch,
            new_branch: request.branch_name,
            checkpoint,
        })
    }

    /// Rename current branch
    pub fn rename_branch(&self, request: RenameBranchRequest) -> Result<BranchRenameResult> {
        Validate::branch_name(&request.new_name)?;

        let current_branch = GitOperations::current_branch()?;

        if BranchOperations::exists(&request.new_name)? {
            return Err(GitXError::GitCommand(format!(
                "Branch '{}' already exists",
                request.new_name
            )));
        }

        // Create backup if requested
        let backup_branch = if request.create_backup {
            Some(Safety::create_backup_branch(Some("pre-rename"))?)
        } else {
            None
        };

        BranchOperations::rename(&request.new_name)?;

        Ok(BranchRenameResult {
            old_name: current_branch,
            new_name: request.new_name,
            backup_branch,
        })
    }

    /// Clean merged branches
    pub fn clean_merged_branches(
        &self,
        request: CleanBranchesRequest,
    ) -> Result<CleanBranchesResult> {
        let merged_branches = GitOperations::merged_branches()?;
        let current_branch = GitOperations::current_branch()?;
        let protected_branches = ["main", "master", "develop"];

        let candidates: Vec<String> = merged_branches
            .into_iter()
            .filter(|branch| branch != &current_branch)
            .filter(|branch| !protected_branches.contains(&branch.as_str()))
            .collect();

        if candidates.is_empty() {
            return Ok(CleanBranchesResult {
                candidates: vec![],
                deleted: vec![],
                failed: vec![],
                dry_run: request.dry_run,
            });
        }

        let mut deleted = Vec::new();
        let mut failed = Vec::new();

        if request.dry_run {
            // In dry run, all candidates would be "deleted"
            deleted = candidates.clone();
        } else {
            // Confirm operation if interactive
            if request.confirm_deletion {
                let details = format!(
                    "This will delete {} merged branches: {}",
                    candidates.len(),
                    candidates.join(", ")
                );

                if !Safety::confirm_destructive_operation("Clean merged branches", &details)? {
                    return Ok(CleanBranchesResult {
                        candidates: candidates.clone(),
                        deleted: vec![],
                        failed: vec![],
                        dry_run: false,
                    });
                }
            }

            // Perform deletions
            for branch in &candidates {
                match BranchOperations::delete(branch, false) {
                    Ok(_) => deleted.push(branch.clone()),
                    Err(_) => failed.push(branch.clone()),
                }
            }
        }

        Ok(CleanBranchesResult {
            candidates,
            deleted,
            failed,
            dry_run: request.dry_run,
        })
    }
}

// Request/Response DTOs for better type safety

#[derive(Debug, Clone)]
pub struct CreateBranchRequest {
    pub name: String,
    pub from: Option<String>,
    pub create_backup: bool,
}

#[derive(Debug, Clone)]
pub struct BranchCreationResult {
    pub branch_name: String,
    pub base_commit: Option<String>,
    pub backup_branch: Option<String>,
    pub switched: bool,
}

#[derive(Debug, Clone)]
pub struct DeleteBranchesRequest {
    pub branches: Vec<String>,
    pub force: bool,
    pub dry_run: bool,
}

#[derive(Debug, Clone)]
pub struct BranchDeletionResult {
    pub deleted: Vec<String>,
    pub failed: Vec<String>,
    pub protected: Vec<String>,
    pub dry_run: bool,
}

#[derive(Debug, Clone)]
pub struct RecentBranchesRequest {
    pub limit: Option<usize>,
    pub exclude_current: bool,
    pub exclude_protected: bool,
}

#[derive(Debug, Clone)]
pub struct RecentBranchesResult {
    pub branches: Vec<String>,
    pub current_branch: String,
}

#[derive(Debug, Clone)]
pub struct SwitchBranchRequest {
    pub branch_name: String,
    pub strict_mode: bool,
    pub create_checkpoint: bool,
}

#[derive(Debug, Clone)]
pub struct BranchSwitchResult {
    pub previous_branch: String,
    pub new_branch: String,
    pub checkpoint: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RenameBranchRequest {
    pub new_name: String,
    pub create_backup: bool,
}

#[derive(Debug, Clone)]
pub struct BranchRenameResult {
    pub old_name: String,
    pub new_name: String,
    pub backup_branch: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CleanBranchesRequest {
    pub dry_run: bool,
    pub confirm_deletion: bool,
}

#[derive(Debug, Clone)]
pub struct CleanBranchesResult {
    pub candidates: Vec<String>,
    pub deleted: Vec<String>,
    pub failed: Vec<String>,
    pub dry_run: bool,
}

impl CleanBranchesResult {
    /// Get a summary of the operation
    pub fn summary(&self) -> String {
        if self.dry_run {
            format!("Would delete {} branches", self.candidates.len())
        } else {
            format!(
                "Deleted {} branches, {} failed",
                self.deleted.len(),
                self.failed.len()
            )
        }
    }
}
