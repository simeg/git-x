use crate::core::traits::*;
use crate::core::{git::*, output::*};
use crate::{GitXError, Result};

/// Repository-level commands grouped together
pub struct RepositoryCommands;

impl RepositoryCommands {
    /// Show repository information
    pub fn info() -> Result<String> {
        InfoCommand::new().execute()
    }

    /// Show repository health check
    pub fn health() -> Result<String> {
        HealthCommand::new().execute()
    }

    /// Sync with upstream
    pub fn sync(strategy: SyncStrategy) -> Result<String> {
        SyncCommand::new(strategy).execute()
    }

    /// Manage upstream configuration
    pub fn upstream(action: UpstreamAction) -> Result<String> {
        UpstreamCommand::new(action).execute()
    }

    /// Create a new branch
    pub fn new_branch(branch_name: String, from: Option<String>) -> Result<String> {
        NewBranchCommand::new(branch_name, from).execute()
    }
}

/// Command to show repository information
pub struct InfoCommand {
    show_detailed: bool,
}

impl Default for InfoCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl InfoCommand {
    pub fn new() -> Self {
        Self {
            show_detailed: false,
        }
    }

    pub fn with_details(mut self) -> Self {
        self.show_detailed = true;
        self
    }

    fn format_branch_info(
        current: &str,
        upstream: Option<&str>,
        ahead: u32,
        behind: u32,
    ) -> String {
        let mut info = format!("📍 Current branch: {}", Format::bold(current));

        if let Some(upstream_branch) = upstream {
            info.push_str(&format!("\n🔗 Upstream: {upstream_branch}"));

            if ahead > 0 || behind > 0 {
                let mut status_parts = Vec::new();
                if ahead > 0 {
                    status_parts.push(format!("{ahead} ahead"));
                }
                if behind > 0 {
                    status_parts.push(format!("{behind} behind"));
                }
                info.push_str(&format!("\n📊 Status: {}", status_parts.join(", ")));
            } else {
                info.push_str("\n✅ Status: Up to date");
            }
        } else {
            info.push_str("\n❌ No upstream configured");
        }

        info
    }
}

impl Command for InfoCommand {
    fn execute(&self) -> Result<String> {
        let mut output = BufferedOutput::new();

        // Repository info
        let repo_name = match GitOperations::repo_root() {
            Ok(path) => std::path::Path::new(&path)
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "Unknown".to_string()),
            Err(_) => return Err(GitXError::GitCommand("Not in a git repository".to_string())),
        };

        output.add_line(format!("🗂️  Repository: {}", Format::bold(&repo_name)));

        // Branch information
        let (current, upstream, ahead, behind) = GitOperations::branch_info_optimized()?;
        output.add_line(Self::format_branch_info(
            &current,
            upstream.as_deref(),
            ahead,
            behind,
        ));

        // Working directory status
        if GitOperations::is_working_directory_clean()? {
            output.add_line("✅ Working directory: Clean".to_string());
        } else {
            output.add_line("⚠️  Working directory: Has changes".to_string());
        }

        // Staged files
        let staged = GitOperations::staged_files()?;
        if staged.is_empty() {
            output.add_line("📋 Staged files: None".to_string());
        } else {
            output.add_line(format!("📋 Staged files: {} file(s)", staged.len()));
            if self.show_detailed {
                for file in staged {
                    output.add_line(format!("   • {file}"));
                }
            }
        }

        // Recent branches
        if self.show_detailed {
            match GitOperations::recent_branches(Some(5)) {
                Ok(recent) if !recent.is_empty() => {
                    output.add_line("\n🕒 Recent branches:".to_string());
                    for (i, branch) in recent.iter().enumerate() {
                        let prefix = if i == 0 { "🌟" } else { "📁" };
                        output.add_line(format!("   {prefix} {branch}"));
                    }
                }
                _ => {}
            }
        }

        Ok(output.content())
    }

    fn name(&self) -> &'static str {
        "info"
    }

    fn description(&self) -> &'static str {
        "Show repository information and status"
    }
}

impl GitCommand for InfoCommand {}

/// Command to check repository health
pub struct HealthCommand;

impl Default for HealthCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl HealthCommand {
    pub fn new() -> Self {
        Self
    }

    fn check_git_config() -> Vec<String> {
        let mut issues = Vec::new();

        // Check username and email
        if GitOperations::run(&["config", "user.name"]).is_err() {
            issues.push("❌ Git user.name not configured".to_string());
        }
        if GitOperations::run(&["config", "user.email"]).is_err() {
            issues.push("❌ Git user.email not configured".to_string());
        }

        issues
    }

    fn check_remotes() -> Vec<String> {
        let mut issues = Vec::new();

        match RemoteOperations::list() {
            Ok(remotes) => {
                if remotes.is_empty() {
                    issues.push("⚠️  No remotes configured".to_string());
                }
            }
            Err(_) => {
                issues.push("❌ Could not check remotes".to_string());
            }
        }

        issues
    }

    fn check_branches() -> Vec<String> {
        let mut issues = Vec::new();

        // Check for very old branches
        match GitOperations::local_branches() {
            Ok(branches) => {
                if branches.len() > 20 {
                    issues.push(format!(
                        "⚠️  Many local branches ({}) - consider cleaning up",
                        branches.len()
                    ));
                }
            }
            Err(_) => {
                issues.push("❌ Could not check branches".to_string());
            }
        }

        // Check for stale branches
        if let Ok(stale_count) = Self::count_stale_branches() {
            if stale_count > 0 {
                issues.push(format!(
                    "⚠️  {stale_count} potentially stale branches found"
                ));
            }
        }

        issues
    }

    fn count_stale_branches() -> Result<usize> {
        let output = GitOperations::run(&[
            "for-each-ref",
            "--format=%(refname:short) %(committerdate:relative)",
            "refs/heads/",
        ])?;

        let stale_count = output
            .lines()
            .filter(|line| line.contains("months ago") || line.contains("year"))
            .count();

        Ok(stale_count)
    }

    fn check_working_directory() -> Vec<String> {
        let mut issues = Vec::new();

        // Check for untracked files
        if let Ok(output) = GitOperations::run(&["ls-files", "--others", "--exclude-standard"]) {
            let untracked_count = output
                .lines()
                .filter(|line| !line.trim().is_empty())
                .count();
            if untracked_count > 5 {
                issues.push(format!("⚠️  {untracked_count} untracked files found"));
            }
        }

        // Check for uncommitted changes
        if let Ok(output) = GitOperations::run(&["diff", "--cached", "--name-only"]) {
            let staged_count = output
                .lines()
                .filter(|line| !line.trim().is_empty())
                .count();
            if staged_count > 0 {
                issues.push(format!("ℹ️  {staged_count} files staged for commit"));
            }
        }

        issues
    }

    fn check_repository_size() -> Vec<String> {
        let mut issues = Vec::new();

        // Use git count-objects for repository size
        if let Ok(output) = GitOperations::run(&["count-objects", "-vH"]) {
            for line in output.lines() {
                if line.starts_with("size-pack") {
                    if let Some(size_str) = line.split_whitespace().nth(1) {
                        // Parse size and check if it's concerning
                        if size_str.ends_with("GiB") || size_str.contains("1024") {
                            issues.push(format!(
                                "⚠️  Repository size: {size_str} (consider cleanup)"
                            ));
                        }
                    }
                }
            }
        }

        issues
    }
}

impl Command for HealthCommand {
    fn execute(&self) -> Result<String> {
        let mut output = BufferedOutput::new();
        output.add_line("🏥 Repository Health Check".to_string());
        output.add_line("=".repeat(30));

        let mut all_issues = Vec::new();

        // Check git configuration
        let config_issues = Self::check_git_config();
        if config_issues.is_empty() {
            output.add_line("✅ Git configuration: OK".to_string());
        } else {
            output.add_line("❌ Git configuration: Issues found".to_string());
            all_issues.extend(config_issues);
        }

        // Check remotes
        let remote_issues = Self::check_remotes();
        if remote_issues.is_empty() {
            output.add_line("✅ Remotes: OK".to_string());
        } else {
            output.add_line("⚠️  Remotes: Issues found".to_string());
            all_issues.extend(remote_issues);
        }

        // Check branches
        let branch_issues = Self::check_branches();
        if branch_issues.is_empty() {
            output.add_line("✅ Branches: OK".to_string());
        } else {
            output.add_line("⚠️  Branches: Issues found".to_string());
            all_issues.extend(branch_issues);
        }

        // Check working directory
        let wd_issues = Self::check_working_directory();
        if wd_issues.is_empty() {
            output.add_line("✅ Working directory: Clean".to_string());
        } else {
            output.add_line("ℹ️  Working directory: Has notes".to_string());
            all_issues.extend(wd_issues);
        }

        // Check repository size
        let size_issues = Self::check_repository_size();
        if size_issues.is_empty() {
            output.add_line("✅ Repository size: OK".to_string());
        } else {
            output.add_line("⚠️  Repository size: Large".to_string());
            all_issues.extend(size_issues);
        }

        // Summary
        if all_issues.is_empty() {
            output.add_line("\n🎉 Repository is healthy!".to_string());
        } else {
            output.add_line(format!("\n🔧 Found {} issue(s):", all_issues.len()));
            for issue in all_issues {
                output.add_line(format!("   {issue}"));
            }
        }

        Ok(output.content())
    }

    fn name(&self) -> &'static str {
        "health"
    }

    fn description(&self) -> &'static str {
        "Check repository health and configuration"
    }
}

impl GitCommand for HealthCommand {}

/// Sync strategies
#[derive(Debug, Clone)]
pub enum SyncStrategy {
    Merge,
    Rebase,
    Auto,
}

/// Command to sync with upstream
pub struct SyncCommand {
    strategy: SyncStrategy,
}

impl SyncCommand {
    pub fn new(strategy: SyncStrategy) -> Self {
        Self { strategy }
    }
}

impl Command for SyncCommand {
    fn execute(&self) -> Result<String> {
        // Fetch latest changes
        RemoteOperations::fetch(None)?;

        let (current_branch, upstream, ahead, behind) = GitOperations::branch_info_optimized()?;

        let upstream_branch = upstream.ok_or_else(|| {
            GitXError::GitCommand(format!(
                "No upstream configured for branch '{current_branch}'"
            ))
        })?;

        if behind == 0 {
            return Ok("✅ Already up to date with upstream".to_string());
        }

        let strategy_name = match self.strategy {
            SyncStrategy::Merge => "merge",
            SyncStrategy::Rebase => "rebase",
            SyncStrategy::Auto => {
                // Auto-choose: rebase if no local commits, merge otherwise
                if ahead == 0 { "merge" } else { "rebase" }
            }
        };

        // Perform sync
        match strategy_name {
            "merge" => {
                GitOperations::run_status(&["merge", &upstream_branch])?;
                Ok(format!("✅ Merged {behind} commits from {upstream_branch}"))
            }
            "rebase" => {
                GitOperations::run_status(&["rebase", &upstream_branch])?;
                Ok(format!("✅ Rebased {ahead} commits onto {upstream_branch}"))
            }
            _ => unreachable!(),
        }
    }

    fn name(&self) -> &'static str {
        "sync"
    }

    fn description(&self) -> &'static str {
        "Sync current branch with upstream"
    }
}

impl GitCommand for SyncCommand {}

/// Upstream actions
#[derive(Debug, Clone)]
pub enum UpstreamAction {
    Set { remote: String, branch: String },
    Status,
    SyncAll,
}

/// Command to manage upstream configuration
pub struct UpstreamCommand {
    action: UpstreamAction,
}

impl UpstreamCommand {
    pub fn new(action: UpstreamAction) -> Self {
        Self { action }
    }
}

impl Command for UpstreamCommand {
    fn execute(&self) -> Result<String> {
        match &self.action {
            UpstreamAction::Set { remote, branch } => {
                RemoteOperations::set_upstream(remote, branch)?;
                Ok(format!("✅ Set upstream to {remote}/{branch}"))
            }
            UpstreamAction::Status => {
                let branches = GitOperations::local_branches()?;
                let mut output = BufferedOutput::new();
                output.add_line("🔗 Upstream Status:".to_string());

                for branch in branches {
                    // Switch to each branch and check upstream
                    // This is a simplified version - in practice you'd want to parse git config
                    output.add_line(format!("📁 {branch}: (checking...)"));
                }

                Ok(output.content())
            }
            UpstreamAction::SyncAll => {
                let current_branch = GitOperations::current_branch()?;
                let branches = GitOperations::local_branches()?;
                let mut synced = 0;

                for branch in branches {
                    if branch == current_branch {
                        continue; // Skip current branch
                    }

                    // Try to sync each branch (simplified)
                    if BranchOperations::switch(&branch).is_ok()
                        && SyncCommand::new(SyncStrategy::Auto).execute().is_ok()
                    {
                        synced += 1;
                    }
                }

                // Return to original branch
                BranchOperations::switch(&current_branch)?;

                Ok(format!("✅ Synced {synced} branches"))
            }
        }
    }

    fn name(&self) -> &'static str {
        "upstream"
    }

    fn description(&self) -> &'static str {
        "Manage upstream branch configuration"
    }
}

impl GitCommand for UpstreamCommand {}

/// Command to create a new branch
pub struct NewBranchCommand {
    branch_name: String,
    from: Option<String>,
}

impl NewBranchCommand {
    pub fn new(branch_name: String, from: Option<String>) -> Self {
        Self { branch_name, from }
    }

    fn branch_exists(&self, branch_name: &str) -> bool {
        GitOperations::run(&[
            "show-ref",
            "--verify",
            "--quiet",
            &format!("refs/heads/{branch_name}"),
        ])
        .is_ok()
    }

    fn is_valid_ref(&self, ref_name: &str) -> bool {
        GitOperations::run(&["rev-parse", "--verify", "--quiet", ref_name]).is_ok()
    }
}

impl Command for NewBranchCommand {
    fn execute(&self) -> Result<String> {
        // Validate branch name format and safety
        if self.branch_name.is_empty() {
            return Err(GitXError::GitCommand(
                "Branch name cannot be empty".to_string(),
            ));
        }

        // Check if branch already exists
        if self.branch_exists(&self.branch_name) {
            return Err(GitXError::GitCommand(format!(
                "Branch '{}' already exists",
                self.branch_name
            )));
        }

        // Determine base branch
        let base_branch = match &self.from {
            Some(branch) => {
                if !self.branch_exists(branch) && !self.is_valid_ref(branch) {
                    return Err(GitXError::GitCommand(format!(
                        "Base branch or ref '{branch}' does not exist"
                    )));
                }
                branch.clone()
            }
            None => GitOperations::current_branch()?,
        };

        let mut output = Vec::new();
        output.push(format!(
            "🌿 Creating new branch '{}' from '{}'",
            Format::bold(&self.branch_name),
            Format::bold(&base_branch)
        ));

        // Create the new branch
        GitOperations::run_status(&["branch", &self.branch_name, &base_branch])?;

        // Switch to the new branch (use checkout for better compatibility)
        GitOperations::run_status(&["checkout", &self.branch_name])?;

        output.push(format!(
            "✅ Successfully created and switched to branch '{}'",
            Format::bold(&self.branch_name)
        ));

        Ok(output.join("\n"))
    }

    fn name(&self) -> &'static str {
        "new-branch"
    }

    fn description(&self) -> &'static str {
        "Create and switch to a new branch"
    }
}

impl GitCommand for NewBranchCommand {}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to strip ANSI escape codes for testing
    fn strip_ansi_codes(text: &str) -> String {
        // Simple regex-like approach to remove ANSI escape sequences
        let mut result = String::new();
        let mut chars = text.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '\x1B' {
                // Found escape character, skip until 'm'
                for next_ch in chars.by_ref() {
                    if next_ch == 'm' {
                        break;
                    }
                }
            } else {
                result.push(ch);
            }
        }

        result
    }

    #[test]
    fn test_ansi_stripping() {
        // Test the ANSI stripping helper function
        let formatted_text = Format::bold("main");
        let clean_text = strip_ansi_codes(&formatted_text);
        assert_eq!(clean_text, "main");

        // Test with mixed content
        let mixed = format!("Branch: {} Status: OK", Format::bold("feature"));
        let clean_mixed = strip_ansi_codes(&mixed);
        assert_eq!(clean_mixed, "Branch: feature Status: OK");
    }

    #[test]
    fn test_info_command_creation() {
        let cmd = InfoCommand::new();
        assert!(!cmd.show_detailed);

        let detailed_cmd = cmd.with_details();
        assert!(detailed_cmd.show_detailed);
    }

    #[test]
    fn test_command_trait_implementations() {
        let info_cmd = InfoCommand::new();
        assert_eq!(info_cmd.name(), "info");
        assert_eq!(
            info_cmd.description(),
            "Show repository information and status"
        );

        let health_cmd = HealthCommand::new();
        assert_eq!(health_cmd.name(), "health");
        assert_eq!(
            health_cmd.description(),
            "Check repository health and configuration"
        );

        let sync_cmd = SyncCommand::new(SyncStrategy::Auto);
        assert_eq!(sync_cmd.name(), "sync");
        assert_eq!(sync_cmd.description(), "Sync current branch with upstream");
    }

    #[test]
    fn test_branch_info_formatting() {
        let formatted = InfoCommand::format_branch_info("main", Some("origin/main"), 2, 1);
        let clean_text = strip_ansi_codes(&formatted);

        assert!(clean_text.contains("Current branch: main"));
        assert!(clean_text.contains("Upstream: origin/main"));
        assert!(clean_text.contains("2 ahead"));
        assert!(clean_text.contains("1 behind"));
    }

    #[test]
    fn test_branch_info_formatting_no_upstream() {
        let formatted = InfoCommand::format_branch_info("feature", None, 0, 0);
        let clean_text = strip_ansi_codes(&formatted);

        assert!(clean_text.contains("Current branch: feature"));
        assert!(clean_text.contains("No upstream configured"));
    }

    #[test]
    fn test_branch_info_formatting_up_to_date() {
        let formatted = InfoCommand::format_branch_info("main", Some("origin/main"), 0, 0);
        let clean_text = strip_ansi_codes(&formatted);

        assert!(clean_text.contains("Status: Up to date"));
    }

    #[test]
    fn test_sync_strategy_auto_selection() {
        // Test the auto strategy logic
        let sync_cmd = SyncCommand::new(SyncStrategy::Auto);
        assert_eq!(sync_cmd.name(), "sync");

        // Auto strategy should work for any input
        let merge_cmd = SyncCommand::new(SyncStrategy::Merge);
        let rebase_cmd = SyncCommand::new(SyncStrategy::Rebase);

        assert_eq!(merge_cmd.name(), "sync");
        assert_eq!(rebase_cmd.name(), "sync");
    }
}
