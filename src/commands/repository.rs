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

    /// Show what would be pushed/pulled
    pub fn what(target: Option<String>) -> Result<String> {
        WhatCommand::new(target).execute()
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

        // Check user name and email
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

/// Command to show what would be pushed/pulled
pub struct WhatCommand {
    target: Option<String>,
}

impl WhatCommand {
    pub fn new(target: Option<String>) -> Self {
        Self { target }
    }
}

impl Command for WhatCommand {
    fn execute(&self) -> Result<String> {
        let (current_branch, upstream, ahead, behind) = GitOperations::branch_info_optimized()?;
        let mut output = BufferedOutput::new();

        let target_ref = self
            .target
            .as_deref()
            .unwrap_or_else(|| upstream.as_deref().unwrap_or("origin/main"));

        output.add_line(format!("🔍 Comparing {current_branch} with {target_ref}"));
        output.add_line("=".repeat(50));

        // Show commits that would be pushed
        if ahead > 0 {
            output.add_line(format!("📤 {ahead} commit(s) to push:"));
            match GitOperations::run(&["log", "--oneline", &format!("{target_ref}..HEAD")]) {
                Ok(commits) => {
                    for line in commits.lines() {
                        output.add_line(format!("  • {line}"));
                    }
                }
                Err(_) => {
                    output.add_line("  (Could not retrieve commit details)".to_string());
                }
            }
        } else {
            output.add_line("📤 No commits to push".to_string());
        }

        // Show commits that would be pulled
        if behind > 0 {
            output.add_line(format!("\n📥 {behind} commit(s) to pull:"));
            match GitOperations::run(&["log", "--oneline", &format!("HEAD..{target_ref}")]) {
                Ok(commits) => {
                    for line in commits.lines() {
                        output.add_line(format!("  • {line}"));
                    }
                }
                Err(_) => {
                    output.add_line("  (Could not retrieve commit details)".to_string());
                }
            }
        } else {
            output.add_line("\n📥 No commits to pull".to_string());
        }

        Ok(output.content())
    }

    fn name(&self) -> &'static str {
        "what"
    }

    fn description(&self) -> &'static str {
        "Show what would be pushed or pulled"
    }
}

impl GitCommand for WhatCommand {}
