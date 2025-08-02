use crate::Result;
use crate::core::git::AsyncGitOperations;
use crate::core::traits::*;
use crate::core::{git::*, output::*};
use chrono::{NaiveDate, Utc};
use std::collections::{BTreeMap, HashMap};

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

    /// Analyze what changed between branches
    pub fn what(target: Option<String>) -> Result<String> {
        WhatCommand::new(target).execute()
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

    fn get_detailed_commit_summary(&self) -> Result<String> {
        let since_arg = self.since.as_deref().unwrap_or("1 month ago");
        let git_log_output = GitOperations::run(&[
            "log",
            "--since",
            since_arg,
            "--pretty=format:%h|%ad|%s|%an|%cr",
            "--date=short",
        ])?;

        if git_log_output.trim().is_empty() {
            return Ok(format!("📅 No commits found since {since_arg}"));
        }

        let grouped = self.parse_git_log_output(&git_log_output);
        Ok(self.format_commit_summary(since_arg, &grouped))
    }

    fn parse_git_log_output(&self, stdout: &str) -> BTreeMap<NaiveDate, Vec<String>> {
        let mut grouped: BTreeMap<NaiveDate, Vec<String>> = BTreeMap::new();

        for line in stdout.lines() {
            if let Some((date, formatted_commit)) = self.parse_commit_line(line) {
                grouped.entry(date).or_default().push(formatted_commit);
            }
        }

        grouped
    }

    fn parse_commit_line(&self, line: &str) -> Option<(NaiveDate, String)> {
        let parts: Vec<&str> = line.splitn(5, '|').collect();
        if parts.len() != 5 {
            return None;
        }

        let date = self.parse_commit_date(parts[1])?;
        let message = parts[2];
        let entry = format!(" - {} {}", self.get_commit_emoji(message), message.trim());
        let author = parts[3];
        let time = parts[4];
        let meta = format!("(by {author}, {time})");
        Some((date, format!("{entry} {meta}")))
    }

    fn parse_commit_date(&self, date_str: &str) -> Option<NaiveDate> {
        NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .ok()
            .or_else(|| Some(Utc::now().date_naive()))
    }

    fn format_commit_summary(
        &self,
        since: &str,
        grouped: &BTreeMap<NaiveDate, Vec<String>>,
    ) -> String {
        let mut result = format!("📅 Commit Summary since {since}:\n");
        result.push_str(&"=".repeat(50));
        result.push('\n');

        for (date, commits) in grouped.iter().rev() {
            result.push_str(&format!("\n📆 {date}\n"));
            for commit in commits {
                result.push_str(commit);
                result.push('\n');
            }
        }

        result
    }

    fn get_commit_emoji(&self, message: &str) -> &'static str {
        // Use case-insensitive matching without allocation
        let msg_bytes = message.as_bytes();
        if msg_bytes.windows(3).any(|w| w.eq_ignore_ascii_case(b"fix"))
            || msg_bytes.windows(3).any(|w| w.eq_ignore_ascii_case(b"bug"))
        {
            "🐛"
        } else if msg_bytes
            .windows(4)
            .any(|w| w.eq_ignore_ascii_case(b"feat"))
            || msg_bytes.windows(3).any(|w| w.eq_ignore_ascii_case(b"add"))
        {
            "✨"
        } else if msg_bytes
            .windows(6)
            .any(|w| w.eq_ignore_ascii_case(b"remove"))
            || msg_bytes
                .windows(6)
                .any(|w| w.eq_ignore_ascii_case(b"delete"))
        {
            "🔥"
        } else if msg_bytes
            .windows(8)
            .any(|w| w.eq_ignore_ascii_case(b"refactor"))
        {
            "🛠"
        } else {
            "🔹"
        }
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
        // If a specific since parameter is provided, show detailed commit summary
        if self.since.is_some() {
            return self.get_detailed_commit_summary();
        }

        // Otherwise show repository summary
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

/// Async parallel version of SummaryCommand
pub struct AsyncSummaryCommand {
    since: Option<String>,
}

impl AsyncSummaryCommand {
    pub fn new(since: Option<String>) -> Self {
        Self { since }
    }

    pub async fn execute_parallel(&self) -> Result<String> {
        // If a specific since parameter is provided, show detailed commit summary
        if self.since.is_some() {
            return self.get_detailed_commit_summary_async().await;
        }

        // Execute all operations in parallel
        let (
            repo_root_result,
            branch_info_result,
            commit_stats_result,
            author_stats_result,
            file_stats_result,
        ) = tokio::try_join!(
            AsyncGitOperations::repo_root(),
            AsyncGitOperations::branch_info_parallel(),
            self.get_commit_stats_async(),
            self.get_author_stats_async(),
            self.get_file_stats_async(),
        )?;

        let mut output = BufferedOutput::new();

        output.add_line("📊 Repository Summary".to_string());
        output.add_line("=".repeat(50));

        // Repository name
        let repo_name = std::path::Path::new(&repo_root_result)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown".to_string());
        output.add_line(format!("🗂️  Repository: {}", Format::bold(&repo_name)));

        // Current branch info
        let (current_branch, upstream, ahead, behind) = branch_info_result;
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
        output.add_line(format!(
            "📈 Commits ({}): {}",
            commit_stats_result.period, commit_stats_result.total_commits
        ));

        // Author statistics
        if !author_stats_result.is_empty() {
            output.add_line(format!(
                "👥 Top contributors ({}): ",
                self.since.as_deref().unwrap_or("all time")
            ));
            for (i, author) in author_stats_result.iter().take(5).enumerate() {
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

        // File statistics
        output.add_line(format!("📁 Files: {} total", file_stats_result.total_files));

        Ok(output.content())
    }

    async fn get_detailed_commit_summary_async(&self) -> Result<String> {
        let since_arg = self.since.as_deref().unwrap_or("1 month ago");
        let git_log_output = AsyncGitOperations::run(&[
            "log",
            "--since",
            since_arg,
            "--pretty=format:%h|%ad|%s|%an|%cr",
            "--date=short",
        ])
        .await?;

        if git_log_output.trim().is_empty() {
            return Ok(format!("📅 No commits found since {since_arg}"));
        }

        let grouped = self.parse_git_log_output(&git_log_output);
        Ok(self.format_commit_summary(since_arg, &grouped))
    }

    async fn get_commit_stats_async(&self) -> Result<CommitStats> {
        let since_arg = self.since.as_deref().unwrap_or("1 month ago");
        let args = if self.since.is_some() {
            vec!["rev-list", "--count", "--since", since_arg, "HEAD"]
        } else {
            vec!["rev-list", "--count", "HEAD"]
        };

        let count_output = AsyncGitOperations::run(&args).await?;
        let total_commits: u32 = count_output.trim().parse().unwrap_or(0);

        Ok(CommitStats {
            total_commits,
            period: since_arg.to_string(),
        })
    }

    async fn get_author_stats_async(&self) -> Result<Vec<AuthorStats>> {
        let since_arg = self.since.as_deref().unwrap_or("1 month ago");
        let args = vec!["shortlog", "-sn", "--since", since_arg];

        let output = AsyncGitOperations::run(&args).await?;
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

    async fn get_file_stats_async(&self) -> Result<FileStats> {
        let (output, wc_output) = tokio::try_join!(
            AsyncGitOperations::run(&["ls-files"]),
            AsyncGitOperations::run(&["ls-files", "-z"])
        )?;

        let total_files = output.lines().count();
        let total_lines = wc_output.split('\0').count();

        Ok(FileStats {
            total_files,
            _total_lines: total_lines,
        })
    }

    // Reuse existing helper methods
    fn parse_git_log_output(
        &self,
        stdout: &str,
    ) -> std::collections::BTreeMap<chrono::NaiveDate, Vec<String>> {
        use std::collections::BTreeMap;
        let mut grouped: BTreeMap<chrono::NaiveDate, Vec<String>> = BTreeMap::new();

        for line in stdout.lines() {
            if let Some((date, formatted_commit)) = self.parse_commit_line(line) {
                grouped.entry(date).or_default().push(formatted_commit);
            }
        }

        grouped
    }

    fn parse_commit_line(&self, line: &str) -> Option<(chrono::NaiveDate, String)> {
        let parts: Vec<&str> = line.splitn(5, '|').collect();
        if parts.len() != 5 {
            return None;
        }

        let date = self.parse_commit_date(parts[1])?;
        let message = parts[2];
        let entry = format!(" - {} {}", self.get_commit_emoji(message), message.trim());
        let author = parts[3];
        let time = parts[4];
        let meta = format!("(by {author}, {time})");
        Some((date, format!("{entry} {meta}")))
    }

    fn parse_commit_date(&self, date_str: &str) -> Option<chrono::NaiveDate> {
        use chrono::{NaiveDate, Utc};
        NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .ok()
            .or_else(|| Some(Utc::now().date_naive()))
    }

    fn format_commit_summary(
        &self,
        since: &str,
        grouped: &std::collections::BTreeMap<chrono::NaiveDate, Vec<String>>,
    ) -> String {
        let mut result = format!("📅 Commit Summary since {since}:\n");
        result.push_str(&"=".repeat(50));
        result.push('\n');

        for (date, commits) in grouped.iter().rev() {
            result.push_str(&format!("\n📆 {date}\n"));
            for commit in commits {
                result.push_str(commit);
                result.push('\n');
            }
        }

        result
    }

    fn get_commit_emoji(&self, message: &str) -> &'static str {
        // Use case-insensitive matching without allocation
        let msg_bytes = message.as_bytes();
        if msg_bytes.windows(3).any(|w| w.eq_ignore_ascii_case(b"fix"))
            || msg_bytes.windows(3).any(|w| w.eq_ignore_ascii_case(b"bug"))
        {
            "🐛"
        } else if msg_bytes
            .windows(4)
            .any(|w| w.eq_ignore_ascii_case(b"feat"))
            || msg_bytes.windows(3).any(|w| w.eq_ignore_ascii_case(b"add"))
        {
            "✨"
        } else if msg_bytes
            .windows(6)
            .any(|w| w.eq_ignore_ascii_case(b"remove"))
            || msg_bytes
                .windows(6)
                .any(|w| w.eq_ignore_ascii_case(b"delete"))
        {
            "🔥"
        } else if msg_bytes
            .windows(8)
            .any(|w| w.eq_ignore_ascii_case(b"refactor"))
        {
            "🛠"
        } else {
            "🔹"
        }
    }
}

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

    fn get_detailed_contributors(&self) -> Result<Vec<ContributorStats>> {
        let args = if let Some(ref since) = self.since {
            vec![
                "log",
                "--all",
                "--format=%ae|%an|%ad",
                "--date=short",
                "--since",
                since,
            ]
        } else {
            vec!["log", "--all", "--format=%ae|%an|%ad", "--date=short"]
        };

        let output = GitOperations::run(&args)?;

        if output.trim().is_empty() {
            return Ok(Vec::new());
        }

        let mut contributors: HashMap<String, ContributorStats> = HashMap::new();

        for line in output.lines() {
            let parts: Vec<&str> = line.splitn(3, '|').collect();
            if parts.len() != 3 {
                continue;
            }

            let email = parts[0].trim().to_string();
            let name = parts[1].trim().to_string();
            let date = parts[2].trim().to_string();

            contributors
                .entry(email.clone())
                .and_modify(|stats| {
                    stats.commit_count += 1;
                    if date < stats.first_commit {
                        stats.first_commit = date.clone();
                    }
                    if date > stats.last_commit {
                        stats.last_commit = date.clone();
                    }
                })
                .or_insert(ContributorStats {
                    name: name.clone(),
                    email: email.clone(),
                    commit_count: 1,
                    first_commit: date.clone(),
                    last_commit: date,
                });
        }

        let mut sorted_contributors: Vec<ContributorStats> = contributors.into_values().collect();
        sorted_contributors.sort_by(|a, b| b.commit_count.cmp(&a.commit_count));

        Ok(sorted_contributors)
    }
}

impl Command for ContributorsCommand {
    fn execute(&self) -> Result<String> {
        let contributors = self.get_detailed_contributors()?;

        if contributors.is_empty() {
            return Ok("📊 No contributors found in this repository".to_string());
        }

        let total_commits: usize = contributors.iter().map(|c| c.commit_count).sum();
        let mut result = String::new();

        let time_period = self.since.as_deref().unwrap_or("all time");
        result.push_str(&format!(
            "📊 Repository Contributors ({total_commits} total commits, {time_period}):\n"
        ));
        result.push_str(&"=".repeat(60));
        result.push('\n');

        for (index, contributor) in contributors.iter().enumerate() {
            let rank_icon = match index {
                0 => "🥇",
                1 => "🥈",
                2 => "🥉",
                _ => "👤",
            };

            let percentage = (contributor.commit_count as f64 / total_commits as f64) * 100.0;

            result.push_str(&format!(
                "{} {} {} commits ({:.1}%)\n",
                rank_icon, contributor.name, contributor.commit_count, percentage
            ));

            result.push_str(&format!(
                "   📧 {} | 📅 {} to {}\n",
                contributor.email, contributor.first_commit, contributor.last_commit
            ));

            if index < contributors.len() - 1 {
                result.push('\n');
            }
        }

        Ok(result)
    }

    fn name(&self) -> &'static str {
        "contributors"
    }

    fn description(&self) -> &'static str {
        "Show repository contributors and their commit statistics"
    }
}

impl GitCommand for ContributorsCommand {}

/// Parallel version of ContributorsCommand using multi-threading
pub struct ParallelContributorsCommand {
    since: Option<String>,
}

impl ParallelContributorsCommand {
    pub fn new(since: Option<String>) -> Self {
        Self { since }
    }

    pub fn execute_parallel(&self) -> Result<String> {
        use rayon::prelude::*;
        use std::collections::HashMap;

        let args = if let Some(ref since) = self.since {
            vec![
                "log",
                "--all",
                "--format=%ae|%an|%ad",
                "--date=short",
                "--since",
                since,
            ]
        } else {
            vec!["log", "--all", "--format=%ae|%an|%ad", "--date=short"]
        };

        let output = GitOperations::run(&args)?;

        if output.trim().is_empty() {
            return Ok("No commits found".to_string());
        }

        // Split lines and process in parallel
        let lines: Vec<&str> = output.lines().collect();

        // Use parallel processing for line parsing and aggregation
        let contributors: HashMap<String, ContributorStats> = lines
            .par_iter()
            .filter_map(|&line| {
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() == 3 {
                    let email = parts[0].trim().to_string();
                    let name = parts[1].trim().to_string();
                    let date = parts[2].trim().to_string();

                    Some((
                        email.clone(),
                        ContributorStats {
                            name,
                            email,
                            commit_count: 1,
                            first_commit: date.clone(),
                            last_commit: date,
                        },
                    ))
                } else {
                    None
                }
            })
            .fold(
                HashMap::new,
                |mut acc: HashMap<String, ContributorStats>, (email, stats)| {
                    acc.entry(email)
                        .and_modify(|existing| {
                            existing.commit_count += 1;
                            if stats.first_commit < existing.first_commit {
                                existing.first_commit = stats.first_commit.clone();
                            }
                            if stats.last_commit > existing.last_commit {
                                existing.last_commit = stats.last_commit.clone();
                            }
                        })
                        .or_insert(stats);
                    acc
                },
            )
            .reduce(HashMap::new, |mut acc, map| {
                for (email, stats) in map {
                    acc.entry(email)
                        .and_modify(|existing| {
                            existing.commit_count += stats.commit_count;
                            if stats.first_commit < existing.first_commit {
                                existing.first_commit = stats.first_commit.clone();
                            }
                            if stats.last_commit > existing.last_commit {
                                existing.last_commit = stats.last_commit.clone();
                            }
                        })
                        .or_insert(stats);
                }
                acc
            });

        // Sort by commit count
        let mut sorted_contributors: Vec<_> = contributors.into_values().collect();
        sorted_contributors.sort_by(|a, b| b.commit_count.cmp(&a.commit_count));

        // Format output
        let mut output = BufferedOutput::new();
        let period = self.since.as_deref().unwrap_or("all time");
        output.add_line(format!("👥 Contributors ({period})"));
        output.add_line("=".repeat(50));

        for (i, contributor) in sorted_contributors.iter().take(20).enumerate() {
            let rank = match i {
                0 => "🥇",
                1 => "🥈",
                2 => "🥉",
                _ => "👤",
            };

            output.add_line(format!(
                "{} {} {} commits",
                rank, contributor.name, contributor.commit_count
            ));

            output.add_line(format!(
                "   📧 {} | 📅 {} to {}",
                contributor.email, contributor.first_commit, contributor.last_commit
            ));
        }

        Ok(output.content())
    }
}

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

/// Parallel version of TechnicalDebtCommand using multi-threading
pub struct ParallelTechnicalDebtCommand;

impl Default for ParallelTechnicalDebtCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl ParallelTechnicalDebtCommand {
    pub fn new() -> Self {
        Self
    }

    pub fn execute_parallel(&self) -> Result<String> {
        // Run multiple analysis types in parallel
        let ((file_churn_result, large_files_result), old_files_result) = rayon::join(
            || {
                rayon::join(
                    || self.analyze_file_churn_parallel(),
                    || self.analyze_large_files_parallel(),
                )
            },
            || self.analyze_old_files_parallel(),
        );

        let file_churn = file_churn_result?;
        let large_files = large_files_result?;
        let old_files = old_files_result?;

        let mut output = BufferedOutput::new();

        output.add_line("🔧 Technical Debt Analysis".to_string());
        output.add_line("=".repeat(40));

        // File churn analysis
        if !file_churn.is_empty() {
            output.add_line("\n📈 High-churn files (frequently modified):".to_string());
            for churn in file_churn.iter().take(10) {
                output.add_line(format!("   🔄 {} ({} changes)", churn.file, churn.changes));
            }
        }

        // Large files analysis
        if !large_files.is_empty() {
            output.add_line("\n📦 Large files:".to_string());
            for file in large_files.iter().take(10) {
                output.add_line(format!("   📁 {} ({:.1} MB)", file.path, file.size_mb));
            }
        }

        // Old files analysis
        if !old_files.is_empty() {
            output.add_line("\n⏰ Potentially stale files (not modified recently):".to_string());
            for file in old_files.iter().take(10) {
                output.add_line(format!("   📅 {file}"));
            }
        }

        if file_churn.is_empty() && large_files.is_empty() && old_files.is_empty() {
            output.add_line("✅ No significant technical debt detected".to_string());
        }

        Ok(output.content())
    }

    fn analyze_file_churn_parallel(&self) -> Result<Vec<FileChurn>> {
        use rayon::prelude::*;
        use std::collections::HashMap;

        let output = GitOperations::run(&[
            "log",
            "--name-only",
            "--pretty=format:",
            "--since=3 months ago",
        ])?;

        let lines: Vec<&str> = output.lines().collect();

        // Parallel counting of file changes
        let file_changes: HashMap<String, u32> = lines
            .par_iter()
            .filter_map(|&line| {
                let line = line.trim();
                if !line.is_empty() && !line.starts_with("commit") {
                    Some((line.to_string(), 1u32))
                } else {
                    None
                }
            })
            .fold(
                HashMap::new,
                |mut acc: HashMap<String, u32>, (file, count)| {
                    *acc.entry(file).or_insert(0) += count;
                    acc
                },
            )
            .reduce(HashMap::new, |mut acc, map| {
                for (file, count) in map {
                    *acc.entry(file).or_insert(0) += count;
                }
                acc
            });

        let mut churns: Vec<FileChurn> = file_changes
            .into_iter()
            .map(|(file, changes)| FileChurn { file, changes })
            .collect();

        churns.sort_by(|a, b| b.changes.cmp(&a.changes));
        churns.retain(|churn| churn.changes > 5); // Only show files with significant churn

        Ok(churns)
    }

    fn analyze_large_files_parallel(&self) -> Result<Vec<LargeFile>> {
        use rayon::prelude::*;

        let output = GitOperations::run(&["ls-files"])?;
        let files: Vec<&str> = output.lines().collect();

        let large_files: Vec<LargeFile> = files
            .par_iter()
            .filter_map(|&file| {
                if let Ok(metadata) = std::fs::metadata(file) {
                    let size_mb = metadata.len() as f64 / 1024.0 / 1024.0;
                    if size_mb >= 0.5 {
                        // Files larger than 500KB
                        Some(LargeFile {
                            path: file.to_string(),
                            size_mb,
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        let mut sorted_files = large_files;
        sorted_files.sort_by(|a, b| b.size_mb.partial_cmp(&a.size_mb).unwrap());

        Ok(sorted_files)
    }

    fn analyze_old_files_parallel(&self) -> Result<Vec<String>> {
        use rayon::prelude::*;

        let output = GitOperations::run(&["ls-files"])?;
        let files: Vec<&str> = output.lines().collect();

        // Get files not modified in last 6 months
        let old_files: Vec<String> = files
            .par_iter()
            .filter_map(|&file| {
                // Check last modification in git
                if let Ok(log_output) =
                    GitOperations::run(&["log", "-1", "--pretty=format:%cr", "--", file])
                {
                    if log_output.contains("months ago") || log_output.contains("year") {
                        Some(format!("{} (last modified: {})", file, log_output.trim()))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        Ok(old_files)
    }
}

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

/// Parallel version of LargeFilesCommand using multi-threading
pub struct ParallelLargeFilesCommand {
    threshold_mb: Option<f64>,
    limit: Option<usize>,
}

impl ParallelLargeFilesCommand {
    pub fn new(threshold_mb: Option<f64>, limit: Option<usize>) -> Self {
        Self {
            threshold_mb,
            limit,
        }
    }

    pub fn execute_parallel(&self) -> Result<String> {
        use rayon::prelude::*;
        let threshold = self.threshold_mb.unwrap_or(1.0);
        let limit = self.limit.unwrap_or(10);

        let output = GitOperations::run(&["ls-files"])?;
        let files: Vec<&str> = output.lines().collect();

        // Process files in parallel using rayon
        let large_files: Vec<LargeFile> = files
            .par_iter()
            .filter_map(|&file| {
                if let Ok(metadata) = std::fs::metadata(file) {
                    let size_mb = metadata.len() as f64 / 1024.0 / 1024.0;
                    if size_mb >= threshold {
                        Some(LargeFile {
                            path: file.to_string(),
                            size_mb,
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        // Sort and limit results
        let mut sorted_files = large_files;
        sorted_files.sort_by(|a, b| b.size_mb.partial_cmp(&a.size_mb).unwrap());
        sorted_files.truncate(limit);

        if sorted_files.is_empty() {
            return Ok(format!("No files larger than {threshold:.1}MB found"));
        }

        let mut result = format!("📦 Files larger than {threshold:.1}MB:\n");
        result.push_str(&"=".repeat(40));
        result.push('\n');

        for file in sorted_files {
            result.push_str(&format!("🗃️  {} ({:.2} MB)\n", file.path, file.size_mb));
        }

        Ok(result)
    }
}

/// Command to show commits since a certain time or reference
pub struct SinceCommand {
    reference: String,
}

impl SinceCommand {
    pub fn new(reference: String) -> Self {
        Self { reference }
    }
}

impl Command for SinceCommand {
    fn execute(&self) -> Result<String> {
        // First try as a git reference (commit hash, branch, tag)
        let log_range = format!("{}..HEAD", self.reference);
        if let Ok(output) = GitOperations::run(&["log", &log_range, "--pretty=format:- %h %s"]) {
            if !output.trim().is_empty() {
                return Ok(format!("🔍 Commits since {}:\n{}", self.reference, output));
            } else {
                return Ok(format!("✅ No new commits since {}", self.reference));
            }
        }

        // If that fails, try as a time specification
        let output = GitOperations::run(&["log", "--oneline", "--since", &self.reference])?;

        if output.trim().is_empty() {
            return Ok(format!("✅ No commits found since '{}'", self.reference));
        }

        let mut result = format!("📅 Commits since '{}':\n", self.reference);
        result.push_str(&"=".repeat(50));
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
        "Show commits since a reference (e.g., cb676ec, origin/main) or time"
    }
}

impl GitCommand for SinceCommand {}

/// Command to analyze what changed between branches
pub struct WhatCommand {
    target: Option<String>,
}

impl WhatCommand {
    pub fn new(target: Option<String>) -> Self {
        Self { target }
    }

    fn get_default_target(&self) -> String {
        "main".to_string()
    }

    fn format_branch_comparison(&self, current: &str, target: &str) -> String {
        format!(
            "🔍 Branch: {} vs {}",
            Format::bold(current),
            Format::bold(target)
        )
    }

    fn parse_commit_counts(&self, output: &str) -> (String, String) {
        let mut counts = output.split_whitespace();
        let behind = counts.next().unwrap_or("0").to_string();
        let ahead = counts.next().unwrap_or("0").to_string();
        (ahead, behind)
    }

    fn format_commit_counts(&self, ahead: &str, behind: &str) -> (String, String) {
        (
            format!("📈 {ahead} commits ahead"),
            format!("📉 {behind} commits behind"),
        )
    }

    fn format_rev_list_range(&self, target: &str, current: &str) -> String {
        format!("{target}...{current}")
    }

    fn git_status_to_symbol(&self, status: &str) -> &'static str {
        match status {
            "A" => "➕",
            "M" => "🔄",
            "D" => "➖",
            _ => "❓",
        }
    }

    fn format_diff_line(&self, line: &str) -> Option<String> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let symbol = self.git_status_to_symbol(parts[0]);
            Some(format!(" {} {}", symbol, parts[1]))
        } else {
            None
        }
    }
}

impl Command for WhatCommand {
    fn execute(&self) -> Result<String> {
        let target_branch = self
            .target
            .clone()
            .unwrap_or_else(|| self.get_default_target());

        // Get current branch name
        let current_branch = GitOperations::current_branch()?;

        let mut output = Vec::new();
        output.push(self.format_branch_comparison(&current_branch, &target_branch));

        // Get ahead/behind commit counts
        let rev_list_output = GitOperations::run(&[
            "rev-list",
            "--left-right",
            "--count",
            &self.format_rev_list_range(&target_branch, &current_branch),
        ])?;

        let (ahead, behind) = self.parse_commit_counts(&rev_list_output);
        let (ahead_msg, behind_msg) = self.format_commit_counts(&ahead, &behind);
        output.push(ahead_msg);
        output.push(behind_msg);

        // Get diff summary
        let diff_output = GitOperations::run(&[
            "diff",
            "--name-status",
            &self.format_rev_list_range(&target_branch, &current_branch),
        ])?;

        if !diff_output.trim().is_empty() {
            output.push("📝 Changes:".to_string());
            for line in diff_output.lines() {
                if let Some(formatted_line) = self.format_diff_line(line) {
                    output.push(formatted_line);
                }
            }
        } else {
            output.push("✅ No file changes".to_string());
        }

        Ok(output.join("\n"))
    }

    fn name(&self) -> &'static str {
        "what"
    }

    fn description(&self) -> &'static str {
        "Analyze what changed between current branch and target"
    }
}

impl GitCommand for WhatCommand {}

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

#[derive(Debug, Clone)]
struct ContributorStats {
    name: String,
    email: String,
    commit_count: usize,
    first_commit: String,
    last_commit: String,
}
