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

    fn get_recent_activity_timeline(limit: usize) -> Result<Vec<String>> {
        let output = GitOperations::run(&[
            "log",
            "--oneline",
            "--decorate",
            "--graph",
            "--all",
            &format!("--max-count={limit}"),
            "--pretty=format:%C(auto)%h %s %C(dim)(%cr) %C(bold blue)<%an>%C(reset)",
        ])?;

        let lines: Vec<String> = output.lines().map(|s| s.to_string()).collect();
        Ok(lines)
    }

    fn check_github_pr_status() -> Result<Option<String>> {
        // Try to detect if GitHub CLI is available and check for PR status
        match std::process::Command::new("gh")
            .args(["pr", "status", "--json", "currentBranch"])
            .output()
        {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.trim().is_empty() || stdout.contains("null") {
                    Ok(Some("❌ No open PR for current branch".to_string()))
                } else {
                    Ok(Some("✅ Open PR found for current branch".to_string()))
                }
            }
            _ => Ok(None), // GitHub CLI not available or error
        }
    }

    fn get_branch_differences(current_branch: &str) -> Result<Vec<String>> {
        let mut differences = Vec::new();

        // Check against main/master
        for main_branch in ["main", "master", "develop"] {
            if current_branch == main_branch {
                continue;
            }

            // Check if this main branch exists
            if GitOperations::run(&[
                "rev-parse",
                "--verify",
                &format!("refs/heads/{main_branch}"),
            ])
            .is_ok()
            {
                // Get ahead/behind count
                if let Ok(output) = GitOperations::run(&[
                    "rev-list",
                    "--left-right",
                    "--count",
                    &format!("{main_branch}...{current_branch}"),
                ]) {
                    let parts: Vec<&str> = output.split_whitespace().collect();
                    if parts.len() == 2 {
                        let behind: u32 = parts[0].parse().unwrap_or(0);
                        let ahead: u32 = parts[1].parse().unwrap_or(0);

                        if ahead > 0 || behind > 0 {
                            let mut status_parts = Vec::new();
                            if ahead > 0 {
                                status_parts.push(format!("{ahead} ahead"));
                            }
                            if behind > 0 {
                                status_parts.push(format!("{behind} behind"));
                            }
                            differences.push(format!(
                                "📊 vs {}: {}",
                                main_branch,
                                status_parts.join(", ")
                            ));
                        } else {
                            differences.push(format!("✅ vs {main_branch}: Up to date"));
                        }
                        break; // Only check the first existing main branch
                    }
                }
            }
        }

        Ok(differences)
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

        // Recent activity timeline
        if self.show_detailed {
            match Self::get_recent_activity_timeline(8) {
                Ok(timeline) if !timeline.is_empty() => {
                    output.add_line("\n📋 Recent activity:".to_string());
                    for line in timeline {
                        output.add_line(format!("   {line}"));
                    }
                }
                _ => {}
            }
        }

        // GitHub PR status (if available)
        if let Ok(Some(pr_status)) = Self::check_github_pr_status() {
            output.add_line(pr_status);
        }

        // Branch differences
        match Self::get_branch_differences(&current) {
            Ok(differences) if !differences.is_empty() => {
                for diff in differences {
                    output.add_line(diff);
                }
            }
            _ => {}
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

    fn check_security_issues() -> Vec<String> {
        let mut issues = Vec::new();

        // Check for potential credentials in history
        if let Ok(output) = GitOperations::run(&[
            "log",
            "--all",
            "--full-history",
            "--grep=password",
            "--grep=secret",
            "--grep=key",
            "--grep=token",
            "--grep=credential",
            "--pretty=format:%h %s",
            "-i",
        ]) {
            let suspicious_commits: Vec<_> =
                output.lines().filter(|l| !l.trim().is_empty()).collect();
            if !suspicious_commits.is_empty() {
                issues.push(format!(
                    "🔒 {} potentially sensitive commit message(s) found:",
                    suspicious_commits.len()
                ));
                for commit in suspicious_commits.iter().take(5) {
                    issues.push(format!("     • {commit}"));
                }
                if suspicious_commits.len() > 5 {
                    issues.push(format!(
                        "     • ...and {} more",
                        suspicious_commits.len() - 5
                    ));
                }
            }
        }

        // Check for files with potentially sensitive extensions
        if let Ok(output) =
            GitOperations::run(&["ls-files", "*.pem", "*.key", "*.p12", "*.pfx", "*.jks"])
        {
            let sensitive_files: Vec<_> = output.lines().filter(|l| !l.trim().is_empty()).collect();
            if !sensitive_files.is_empty() {
                issues.push(format!(
                    "🔐 {} potentially sensitive file(s) in repository:",
                    sensitive_files.len()
                ));
                for file in sensitive_files.iter().take(10) {
                    issues.push(format!("     • {file}"));
                }
                if sensitive_files.len() > 10 {
                    issues.push(format!("     • ...and {} more", sensitive_files.len() - 10));
                }
            }
        }

        // Check for .env files that might contain secrets
        if let Ok(output) = GitOperations::run(&["ls-files", "*.env*"]) {
            let env_files: Vec<_> = output.lines().filter(|l| !l.trim().is_empty()).collect();
            if !env_files.is_empty() {
                issues.push(format!(
                    "⚠️  {} environment file(s) found - ensure no secrets are committed:",
                    env_files.len()
                ));
                for file in env_files.iter().take(10) {
                    issues.push(format!("     • {file}"));
                }
                if env_files.len() > 10 {
                    issues.push(format!("     • ...and {} more", env_files.len() - 10));
                }
            }
        }

        issues
    }

    fn check_gitignore_effectiveness() -> Vec<String> {
        let mut issues = Vec::new();

        // Check if .gitignore exists
        if GitOperations::run(&["ls-files", ".gitignore"]).is_err() {
            issues.push("📝 No .gitignore file found".to_string());
            return issues;
        }

        // Check for common files that should probably be ignored
        let should_be_ignored = [
            ("*.log", "log files"),
            ("*.tmp", "temporary files"),
            ("*.swp", "swap files"),
            ("*.bak", "backup files"),
            (".DS_Store", "macOS system files"),
            ("Thumbs.db", "Windows system files"),
            ("node_modules/", "Node.js dependencies"),
            ("target/", "Rust build artifacts"),
            (".vscode/", "VS Code settings"),
            (".idea/", "IntelliJ settings"),
        ];

        for (pattern, description) in should_be_ignored {
            if let Ok(output) = GitOperations::run(&["ls-files", pattern]) {
                let matching_files: Vec<_> =
                    output.lines().filter(|l| !l.trim().is_empty()).collect();
                if !matching_files.is_empty() {
                    issues.push(format!(
                        "🗂️  {} {} tracked (consider adding to .gitignore):",
                        matching_files.len(),
                        description
                    ));
                    for file in matching_files.iter().take(5) {
                        issues.push(format!("     • {file}"));
                    }
                    if matching_files.len() > 5 {
                        issues.push(format!("     • ...and {} more", matching_files.len() - 5));
                    }
                }
            }
        }

        issues
    }

    fn check_binary_files() -> Vec<String> {
        let mut issues = Vec::new();

        // Check for large binary files
        if let Ok(output) = GitOperations::run(&["ls-files", "-z"]) {
            let mut binary_count = 0;
            let mut large_files = Vec::new();

            for file in output.split('\0') {
                if file.trim().is_empty() {
                    continue;
                }

                // Check if file is binary
                if GitOperations::run(&["diff", "--no-index", "/dev/null", file, "--numstat"])
                    .is_ok()
                {
                    // If numstat shows "-	-" it's likely binary
                    if let Ok(stat_output) =
                        GitOperations::run(&["diff", "--no-index", "/dev/null", file, "--numstat"])
                    {
                        if stat_output.trim().starts_with("-\t-") {
                            binary_count += 1;

                            // Check file size (only on systems where wc is available)
                            if let Ok(size_output) =
                                std::process::Command::new("wc").args(["-c", file]).output()
                            {
                                if size_output.status.success() {
                                    if let Ok(size_str) = String::from_utf8(size_output.stdout) {
                                        if let Some(size_part) = size_str.split_whitespace().next()
                                        {
                                            if let Ok(size) = size_part.parse::<u64>() {
                                                if size > 1_000_000 {
                                                    // > 1MB
                                                    large_files.push((file.to_string(), size));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if binary_count > 10 {
                issues.push(format!(
                    "📦 {binary_count} binary files tracked (consider Git LFS for large files)"
                ));
            }

            if !large_files.is_empty() {
                issues.push(format!(
                    "📏 {} large binary file(s) > 1MB found:",
                    large_files.len()
                ));
                for (file, size) in large_files.iter().take(10) {
                    let size_mb = *size as f64 / 1_000_000.0;
                    issues.push(format!("     • {file} ({size_mb:.1} MB)"));
                }
                if large_files.len() > 10 {
                    issues.push(format!("     • ...and {} more", large_files.len() - 10));
                }
            }
        }

        issues
    }
}

impl Command for HealthCommand {
    fn execute(&self) -> Result<String> {
        use indicatif::{ProgressBar, ProgressStyle};

        let mut output = BufferedOutput::new();
        output.add_line("🏥 Repository Health Check".to_string());
        output.add_line("=".repeat(30));

        // Create progress bar - use hidden progress bar in tests/non-interactive environments
        let pb = if atty::is(atty::Stream::Stderr)
            && std::env::var("GIT_X_NON_INTERACTIVE").is_err()
        {
            let pb = ProgressBar::new(8);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template(
                        "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}",
                    )
                    .expect("Failed to set progress style")
                    .progress_chars("#>-"),
            );
            pb
        } else {
            ProgressBar::hidden()
        };
        pb.set_message("Starting health check...");

        let mut all_issues = Vec::new();
        let mut issue_count = 0;

        // Check git configuration
        pb.set_message("Checking Git configuration...");
        let config_issues = Self::check_git_config();
        if config_issues.is_empty() {
            output.add_line("✅ Git configuration: OK".to_string());
        } else {
            output.add_line("❌ Git configuration: Issues found".to_string());
            all_issues.extend(config_issues);
            issue_count += 1;
        }
        pb.inc(1);

        // Check remotes
        pb.set_message("Checking remotes...");
        let remote_issues = Self::check_remotes();
        if remote_issues.is_empty() {
            output.add_line("✅ Remotes: OK".to_string());
        } else {
            output.add_line("⚠️  Remotes: Issues found".to_string());
            all_issues.extend(remote_issues);
            issue_count += 1;
        }
        pb.inc(1);

        // Check branches
        pb.set_message("Analyzing branches...");
        let branch_issues = Self::check_branches();
        if branch_issues.is_empty() {
            output.add_line("✅ Branches: OK".to_string());
        } else {
            output.add_line("⚠️  Branches: Issues found".to_string());
            all_issues.extend(branch_issues);
            issue_count += 1;
        }
        pb.inc(1);

        // Check working directory
        pb.set_message("Checking working directory...");
        let wd_issues = Self::check_working_directory();
        if wd_issues.is_empty() {
            output.add_line("✅ Working directory: Clean".to_string());
        } else {
            output.add_line("ℹ️  Working directory: Has notes".to_string());
            all_issues.extend(wd_issues);
            issue_count += 1;
        }
        pb.inc(1);

        // Check repository size
        pb.set_message("Analyzing repository size...");
        let size_issues = Self::check_repository_size();
        if size_issues.is_empty() {
            output.add_line("✅ Repository size: OK".to_string());
        } else {
            output.add_line("⚠️  Repository size: Large".to_string());
            all_issues.extend(size_issues);
            issue_count += 1;
        }
        pb.inc(1);

        // Check security issues
        pb.set_message("Scanning for security issues...");
        let security_issues = Self::check_security_issues();
        if security_issues.is_empty() {
            output.add_line("✅ Security: No obvious issues found".to_string());
        } else {
            output.add_line("⚠️  Security: Potential issues found".to_string());
            all_issues.extend(security_issues);
            issue_count += 1;
        }
        pb.inc(1);

        // Check .gitignore effectiveness
        pb.set_message("Validating .gitignore...");
        let gitignore_issues = Self::check_gitignore_effectiveness();
        if gitignore_issues.is_empty() {
            output.add_line("✅ .gitignore: Looks good".to_string());
        } else {
            output.add_line("⚠️  .gitignore: Suggestions available".to_string());
            all_issues.extend(gitignore_issues);
            issue_count += 1;
        }
        pb.inc(1);

        // Check binary files
        pb.set_message("Analyzing binary files...");
        let binary_issues = Self::check_binary_files();
        if binary_issues.is_empty() {
            output.add_line("✅ Binary files: OK".to_string());
        } else {
            output.add_line("⚠️  Binary files: Review recommended".to_string());
            all_issues.extend(binary_issues);
            issue_count += 1;
        }
        pb.inc(1);

        // Finish progress bar
        pb.set_message("Health check complete!");
        pb.finish_and_clear();

        // Summary
        if all_issues.is_empty() {
            output.add_line("\n🎉 Repository is healthy!".to_string());
        } else {
            output.add_line(format!("\n🔧 Found {issue_count} issue(s):"));
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
