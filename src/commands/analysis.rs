use crate::Result;
use crate::core::traits::*;
use crate::core::{git::*, output::*};
use std::collections::HashMap;

/// Analysis and reporting commands grouped together
pub struct AnalysisCommands;

impl AnalysisCommands {
    /// Generate repository summary
    pub fn summary(since: Option<String>) -> Result<String> {
        SummaryCommand::new(since).execute()
    }

    /// Show repository graph
    pub fn graph(colored: bool) -> Result<String> {
        if colored {
            ColorGraphCommand::new().execute()
        } else {
            GraphCommand::new().execute()
        }
    }

    /// Show contributors statistics
    pub fn contributors(since: Option<String>) -> Result<String> {
        ContributorsCommand::new(since).execute()
    }

    /// Analyze technical debt
    pub fn technical_debt() -> Result<String> {
        TechnicalDebtCommand::new().execute()
    }

    /// Find large files
    pub fn large_files(threshold_mb: Option<f64>, limit: Option<usize>) -> Result<String> {
        LargeFilesCommand::new(threshold_mb, limit).execute()
    }

    /// Show commits since a certain time
    pub fn since(time_spec: String) -> Result<String> {
        SinceCommand::new(time_spec).execute()
    }
}

/// Command to generate repository summary
pub struct SummaryCommand {
    since: Option<String>,
}

impl SummaryCommand {
    pub fn new(since: Option<String>) -> Self {
        Self { since }
    }

    fn get_commit_stats(&self) -> Result<CommitStats> {
        let since_arg = self.since.as_deref().unwrap_or("1 month ago");
        let args = if self.since.is_some() {
            vec!["rev-list", "--count", "--since", since_arg, "HEAD"]
        } else {
            vec!["rev-list", "--count", "HEAD"]
        };

        let count_output = GitOperations::run(&args)?;
        let total_commits: u32 = count_output.trim().parse().unwrap_or(0);

        Ok(CommitStats {
            total_commits,
            period: since_arg.to_string(),
        })
    }

    fn get_author_stats(&self) -> Result<Vec<AuthorStats>> {
        let since_arg = self.since.as_deref().unwrap_or("1 month ago");
        let args = vec!["shortlog", "-sn", "--since", since_arg];

        let output = GitOperations::run(&args)?;
        let mut authors = Vec::new();

        for line in output.lines() {
            if let Some((count_str, name)) = line.trim().split_once('\t') {
                if let Ok(count) = count_str.trim().parse::<u32>() {
                    authors.push(AuthorStats {
                        name: name.to_string(),
                        commits: count,
                    });
                }
            }
        }

        Ok(authors)
    }

    fn get_file_stats(&self) -> Result<FileStats> {
        let output = GitOperations::run(&["ls-files"])?;
        let total_files = output.lines().count();

        // Get lines of code (rough estimate)
        let mut total_lines = 0;
        if let Ok(wc_output) = GitOperations::run(&["ls-files", "-z"]) {
            // This is a simplified version - in practice you'd want better file type detection
            total_lines = wc_output.split('\0').count();
        }

        Ok(FileStats {
            total_files,
            _total_lines: total_lines,
        })
    }
}

impl Command for SummaryCommand {
    fn execute(&self) -> Result<String> {
        let mut output = BufferedOutput::new();

        output.add_line("📊 Repository Summary".to_string());
        output.add_line("=".repeat(50));

        // Repository name
        if let Ok(repo_path) = GitOperations::repo_root() {
            let repo_name = std::path::Path::new(&repo_path)
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "Unknown".to_string());
            output.add_line(format!("🗂️  Repository: {}", Format::bold(&repo_name)));
        }

        // Current branch info
        let (current_branch, upstream, ahead, behind) = GitOperations::branch_info_optimized()?;
        output.add_line(format!(
            "📍 Current branch: {}",
            Format::bold(&current_branch)
        ));

        if let Some(upstream_branch) = upstream {
            if ahead > 0 || behind > 0 {
                output.add_line(format!(
                    "🔗 Upstream: {upstream_branch} ({ahead} ahead, {behind} behind)"
                ));
            } else {
                output.add_line(format!("🔗 Upstream: {upstream_branch} (up to date)"));
            }
        }

        // Commit statistics
        match self.get_commit_stats() {
            Ok(stats) => {
                output.add_line(format!(
                    "📈 Commits ({}): {}",
                    stats.period, stats.total_commits
                ));
            }
            Err(_) => {
                output.add_line("📈 Commits: Unable to retrieve".to_string());
            }
        }

        // Author statistics
        match self.get_author_stats() {
            Ok(authors) => {
                if !authors.is_empty() {
                    output.add_line(format!(
                        "👥 Top contributors ({}): ",
                        self.since.as_deref().unwrap_or("all time")
                    ));
                    for (i, author) in authors.iter().take(5).enumerate() {
                        let prefix = match i {
                            0 => "🥇",
                            1 => "🥈",
                            2 => "🥉",
                            _ => "👤",
                        };
                        output.add_line(format!(
                            "   {} {} ({} commits)",
                            prefix, author.name, author.commits
                        ));
                    }
                }
            }
            Err(_) => {
                output.add_line("👥 Contributors: Unable to retrieve".to_string());
            }
        }

        // File statistics
        match self.get_file_stats() {
            Ok(stats) => {
                output.add_line(format!("📁 Files: {} total", stats.total_files));
            }
            Err(_) => {
                output.add_line("📁 Files: Unable to retrieve".to_string());
            }
        }

        Ok(output.content())
    }

    fn name(&self) -> &'static str {
        "summary"
    }

    fn description(&self) -> &'static str {
        "Generate a summary of repository activity"
    }
}

impl GitCommand for SummaryCommand {}

/// Command to show colored commit graph
pub struct ColorGraphCommand;

impl Default for ColorGraphCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl ColorGraphCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for ColorGraphCommand {
    fn execute(&self) -> Result<String> {
        GitOperations::run(&[
            "log",
            "--graph",
            "--pretty=format:%C(auto)%h%d %s %C(black)%C(bold)%cr",
            "--abbrev-commit",
            "--all",
            "-20", // Limit to recent commits for better performance
        ])
    }

    fn name(&self) -> &'static str {
        "color-graph"
    }

    fn description(&self) -> &'static str {
        "Show a colored commit graph"
    }
}

impl GitCommand for ColorGraphCommand {}

/// Command to show simple commit graph
pub struct GraphCommand;

impl Default for GraphCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for GraphCommand {
    fn execute(&self) -> Result<String> {
        GitOperations::run(&["log", "--graph", "--oneline", "--all", "-20"])
    }

    fn name(&self) -> &'static str {
        "graph"
    }

    fn description(&self) -> &'static str {
        "Show a simple commit graph"
    }
}

impl GitCommand for GraphCommand {}

/// Command to show contributors
pub struct ContributorsCommand {
    since: Option<String>,
}

impl ContributorsCommand {
    pub fn new(since: Option<String>) -> Self {
        Self { since }
    }
}

impl Command for ContributorsCommand {
    fn execute(&self) -> Result<String> {
        let mut args = vec!["shortlog", "-sn"];
        if let Some(ref since) = self.since {
            args.extend_from_slice(&["--since", since]);
        }

        let output = GitOperations::run(&args)?;
        let mut result = String::new();

        result.push_str("👥 Contributors:\n");
        result.push_str(&"=".repeat(30));
        result.push('\n');

        for (i, line) in output.lines().enumerate() {
            if let Some((count, name)) = line.trim().split_once('\t') {
                let prefix = match i {
                    0 => "🥇",
                    1 => "🥈",
                    2 => "🥉",
                    _ => "👤",
                };
                result.push_str(&format!("{prefix} {name} ({count} commits)\n"));
            }
        }

        Ok(result)
    }

    fn name(&self) -> &'static str {
        "contributors"
    }

    fn description(&self) -> &'static str {
        "Show contributor statistics"
    }
}

impl GitCommand for ContributorsCommand {}

/// Command to analyze technical debt
pub struct TechnicalDebtCommand;

impl Default for TechnicalDebtCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl TechnicalDebtCommand {
    pub fn new() -> Self {
        Self
    }

    fn analyze_file_churn(&self) -> Result<Vec<FileChurn>> {
        let output = GitOperations::run(&[
            "log",
            "--name-only",
            "--pretty=format:",
            "--since=3 months ago",
        ])?;

        let mut file_changes: HashMap<String, u32> = HashMap::new();

        for line in output.lines() {
            let line = line.trim();
            if !line.is_empty() && !line.starts_with("commit") {
                *file_changes.entry(line.to_string()).or_insert(0) += 1;
            }
        }

        let mut churns: Vec<FileChurn> = file_changes
            .into_iter()
            .map(|(file, changes)| FileChurn { file, changes })
            .collect();

        churns.sort_by(|a, b| b.changes.cmp(&a.changes));
        churns.truncate(10); // Top 10 most changed files

        Ok(churns)
    }

    fn find_large_files(&self) -> Result<Vec<LargeFile>> {
        // This is a simplified version - you'd want to use git-sizer or similar tools
        let output = GitOperations::run(&["ls-files"])?;
        let mut large_files = Vec::new();

        for file in output.lines() {
            if let Ok(metadata) = std::fs::metadata(file) {
                let size_mb = metadata.len() as f64 / 1024.0 / 1024.0;
                if size_mb > 1.0 {
                    // Files larger than 1MB
                    large_files.push(LargeFile {
                        path: file.to_string(),
                        size_mb,
                    });
                }
            }
        }

        large_files.sort_by(|a, b| b.size_mb.partial_cmp(&a.size_mb).unwrap());
        large_files.truncate(10);

        Ok(large_files)
    }
}

impl Command for TechnicalDebtCommand {
    fn execute(&self) -> Result<String> {
        let mut output = BufferedOutput::new();

        output.add_line("🔧 Technical Debt Analysis".to_string());
        output.add_line("=".repeat(50));

        // File churn analysis
        match self.analyze_file_churn() {
            Ok(churns) if !churns.is_empty() => {
                output.add_line("📊 Most frequently changed files (last 3 months):".to_string());
                for churn in churns {
                    output.add_line(format!("   📁 {} ({} changes)", churn.file, churn.changes));
                }
                output.add_line("".to_string());
            }
            _ => {
                output.add_line("📊 File churn: No data available".to_string());
            }
        }

        // Large files analysis
        match self.find_large_files() {
            Ok(large_files) if !large_files.is_empty() => {
                output.add_line("📦 Large files (>1MB):".to_string());
                for file in large_files {
                    output.add_line(format!("   🗃️  {} ({:.2} MB)", file.path, file.size_mb));
                }
            }
            _ => {
                output.add_line("📦 Large files: None found".to_string());
            }
        }

        Ok(output.content())
    }

    fn name(&self) -> &'static str {
        "technical-debt"
    }

    fn description(&self) -> &'static str {
        "Analyze technical debt indicators"
    }
}

impl GitCommand for TechnicalDebtCommand {}

/// Command to find large files
pub struct LargeFilesCommand {
    threshold_mb: Option<f64>,
    limit: Option<usize>,
}

impl LargeFilesCommand {
    pub fn new(threshold_mb: Option<f64>, limit: Option<usize>) -> Self {
        Self {
            threshold_mb,
            limit,
        }
    }
}

impl Command for LargeFilesCommand {
    fn execute(&self) -> Result<String> {
        let threshold = self.threshold_mb.unwrap_or(1.0);
        let limit = self.limit.unwrap_or(10);

        let output = GitOperations::run(&["ls-files"])?;
        let mut large_files = Vec::new();

        for file in output.lines() {
            if let Ok(metadata) = std::fs::metadata(file) {
                let size_mb = metadata.len() as f64 / 1024.0 / 1024.0;
                if size_mb >= threshold {
                    large_files.push(LargeFile {
                        path: file.to_string(),
                        size_mb,
                    });
                }
            }
        }

        large_files.sort_by(|a, b| b.size_mb.partial_cmp(&a.size_mb).unwrap());
        large_files.truncate(limit);

        if large_files.is_empty() {
            return Ok(format!("No files larger than {threshold:.1}MB found"));
        }

        let mut result = format!("📦 Files larger than {threshold:.1}MB:\n");
        result.push_str(&"=".repeat(40));
        result.push('\n');

        for file in large_files {
            result.push_str(&format!("🗃️  {} ({:.2} MB)\n", file.path, file.size_mb));
        }

        Ok(result)
    }

    fn name(&self) -> &'static str {
        "large-files"
    }

    fn description(&self) -> &'static str {
        "Find large files in the repository"
    }
}

impl GitCommand for LargeFilesCommand {}

/// Command to show commits since a certain time
pub struct SinceCommand {
    time_spec: String,
}

impl SinceCommand {
    pub fn new(time_spec: String) -> Self {
        Self { time_spec }
    }
}

impl Command for SinceCommand {
    fn execute(&self) -> Result<String> {
        let output = GitOperations::run(&["log", "--oneline", "--since", &self.time_spec])?;

        if output.trim().is_empty() {
            return Ok(format!("No commits found since '{}'", self.time_spec));
        }

        let mut result = format!("📅 Commits since '{}':\n", self.time_spec);
        result.push_str(&"=".repeat(40));
        result.push('\n');

        for line in output.lines() {
            result.push_str(&format!("• {line}\n"));
        }

        Ok(result)
    }

    fn name(&self) -> &'static str {
        "since"
    }

    fn description(&self) -> &'static str {
        "Show commits since a specific time"
    }
}

impl GitCommand for SinceCommand {}

// Supporting data structures
#[derive(Debug)]
struct CommitStats {
    total_commits: u32,
    period: String,
}

#[derive(Debug)]
struct AuthorStats {
    name: String,
    commits: u32,
}

#[derive(Debug)]
struct FileStats {
    total_files: usize,
    _total_lines: usize,
}

#[derive(Debug)]
struct FileChurn {
    file: String,
    changes: u32,
}

#[derive(Debug)]
struct LargeFile {
    path: String,
    size_mb: f64,
}
