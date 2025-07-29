use crate::core::{git::*, validation::Validate};
use crate::{GitXError, Result};

/// High-level git repository abstraction
pub struct GitRepository {
    root_path: String,
}

impl GitRepository {
    /// Create a new repository instance
    pub fn open() -> Result<Self> {
        let root_path = GitOperations::repo_root()?;
        Ok(Self { root_path })
    }

    /// Get repository information
    pub fn info(&self) -> Result<RepositoryInfo> {
        let name = std::path::Path::new(&self.root_path)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        let (current_branch, upstream, ahead, behind) = GitOperations::branch_info_optimized()?;
        let is_clean = GitOperations::is_working_directory_clean()?;
        let staged_files = GitOperations::staged_files()?;

        Ok(RepositoryInfo {
            name,
            root_path: self.root_path.clone(),
            current_branch,
            upstream_branch: upstream,
            ahead_count: ahead,
            behind_count: behind,
            is_clean,
            staged_files_count: staged_files.len(),
        })
    }

    /// Get repository health status
    pub fn health(&self) -> Result<HealthStatus> {
        let mut issues = Vec::new();
        let mut warnings = Vec::new();

        // Check git configuration
        if GitOperations::run(&["config", "user.name"]).is_err() {
            issues.push("Git user.name not configured".to_string());
        }
        if GitOperations::run(&["config", "user.email"]).is_err() {
            issues.push("Git user.email not configured".to_string());
        }

        // Check remotes
        match RemoteOperations::list() {
            Ok(remotes) => {
                if remotes.is_empty() {
                    warnings.push("No remotes configured".to_string());
                }
            }
            Err(_) => {
                issues.push("Could not check remotes".to_string());
            }
        }

        // Check for too many branches
        match GitOperations::local_branches() {
            Ok(branches) => {
                if branches.len() > 20 {
                    warnings.push(format!(
                        "Many local branches ({}) - consider cleaning up",
                        branches.len()
                    ));
                }
            }
            Err(_) => {
                issues.push("Could not check branches".to_string());
            }
        }

        let status = if issues.is_empty() && warnings.is_empty() {
            HealthLevel::Healthy
        } else if issues.is_empty() {
            HealthLevel::Warning
        } else {
            HealthLevel::Unhealthy
        };

        Ok(HealthStatus {
            level: status,
            issues,
            warnings,
        })
    }

    /// Validate repository state for operations
    pub fn validate_for_operation(&self, operation: &str) -> Result<()> {
        Validate::in_git_repo()?;

        match operation {
            "destructive" => {
                if !GitOperations::is_working_directory_clean()? {
                    return Err(GitXError::GitCommand(
                        "Working directory must be clean for destructive operations".to_string(),
                    ));
                }
            }
            "commit" => {
                let staged = GitOperations::staged_files()?;
                if staged.is_empty() {
                    return Err(GitXError::GitCommand("No staged changes found".to_string()));
                }
            }
            _ => {} // No specific validation needed
        }

        Ok(())
    }

    /// Get the repository root path
    pub fn root_path(&self) -> &str {
        &self.root_path
    }
}

/// Repository information structure
#[derive(Debug, Clone)]
pub struct RepositoryInfo {
    pub name: String,
    pub root_path: String,
    pub current_branch: String,
    pub upstream_branch: Option<String>,
    pub ahead_count: u32,
    pub behind_count: u32,
    pub is_clean: bool,
    pub staged_files_count: usize,
}

impl RepositoryInfo {
    /// Check if the repository is in sync with upstream
    pub fn is_in_sync(&self) -> bool {
        self.ahead_count == 0 && self.behind_count == 0
    }

    /// Check if there are local changes
    pub fn has_local_changes(&self) -> bool {
        !self.is_clean || self.staged_files_count > 0
    }

    /// Get a human-readable status description
    pub fn status_description(&self) -> String {
        let mut parts = Vec::new();

        if let Some(ref upstream) = self.upstream_branch {
            if self.is_in_sync() {
                parts.push(format!("up to date with {upstream}"));
            } else {
                if self.ahead_count > 0 {
                    parts.push(format!("{} ahead", self.ahead_count));
                }
                if self.behind_count > 0 {
                    parts.push(format!("{} behind", self.behind_count));
                }
            }
        } else {
            parts.push("no upstream configured".to_string());
        }

        if self.has_local_changes() {
            parts.push("has local changes".to_string());
        } else {
            parts.push("clean".to_string());
        }

        parts.join(", ")
    }
}

/// Repository health status
#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub level: HealthLevel,
    pub issues: Vec<String>,
    pub warnings: Vec<String>,
}

impl HealthStatus {
    /// Check if the repository is healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self.level, HealthLevel::Healthy)
    }

    /// Get a summary message
    pub fn summary(&self) -> String {
        match self.level {
            HealthLevel::Healthy => "Repository is healthy".to_string(),
            HealthLevel::Warning => format!("Repository has {} warning(s)", self.warnings.len()),
            HealthLevel::Unhealthy => format!("Repository has {} issue(s)", self.issues.len()),
        }
    }

    /// Get all problems (issues + warnings)
    pub fn all_problems(&self) -> Vec<String> {
        let mut problems = self.issues.clone();
        problems.extend(self.warnings.clone());
        problems
    }
}

/// Health level enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum HealthLevel {
    Healthy,
    Warning,
    Unhealthy,
}
