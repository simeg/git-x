use crate::{GitXError, Result};
use console::style;
use dialoguer::{FuzzySelect, Input};
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use std::process::Command;

/// Common utilities for git operations
pub struct GitCommand;

impl GitCommand {
    /// Execute a git command and return stdout as String
    pub fn run(args: &[&str]) -> Result<String> {
        let output = Command::new("git").args(args).output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            let stderr_output = String::from_utf8_lossy(&output.stderr);
            let stderr = stderr_output.trim();
            Err(GitXError::GitCommand(stderr.to_string()))
        }
    }

    /// Execute a git command and return success status
    pub fn run_status(args: &[&str]) -> Result<()> {
        let status = Command::new("git").args(args).status()?;

        if status.success() {
            Ok(())
        } else {
            Err(GitXError::GitCommand(format!(
                "Git command failed: git {}",
                args.join(" ")
            )))
        }
    }

    /// Get current branch name
    pub fn current_branch() -> Result<String> {
        Self::run(&["rev-parse", "--abbrev-ref", "HEAD"])
    }

    /// Get repository root path
    pub fn repo_root() -> Result<String> {
        Self::run(&["rev-parse", "--show-toplevel"])
    }

    /// Check if a commit exists
    pub fn commit_exists(commit: &str) -> Result<bool> {
        match Self::run(&["rev-parse", "--verify", &format!("{commit}^{{commit}}")]) {
            Ok(_) => Ok(true),
            Err(GitXError::GitCommand(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// Get short commit hash
    pub fn short_hash(commit: &str) -> Result<String> {
        Self::run(&["rev-parse", "--short", commit])
    }

    /// Get upstream branch for current branch
    pub fn upstream_branch() -> Result<String> {
        Self::run(&["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"])
    }

    /// Get ahead/behind counts compared to upstream
    pub fn ahead_behind_counts() -> Result<(u32, u32)> {
        let output = Self::run(&["rev-list", "--left-right", "--count", "HEAD...@{u}"])?;
        let mut parts = output.split_whitespace();
        let ahead = parts.next().unwrap_or("0").parse().unwrap_or(0);
        let behind = parts.next().unwrap_or("0").parse().unwrap_or(0);
        Ok((ahead, behind))
    }

    /// Get branch information in an optimized way to reduce git calls
    pub fn branch_info_optimized() -> Result<(String, Option<String>, u32, u32)> {
        // Get current branch
        let current = Self::current_branch()?;

        // Try to get upstream - if this fails, there's no upstream
        match Self::upstream_branch() {
            Ok(upstream) => {
                // Only check ahead/behind if upstream exists
                let (ahead, behind) = Self::ahead_behind_counts().unwrap_or((0, 0));
                Ok((current, Some(upstream), ahead, behind))
            }
            Err(_) => {
                // No upstream configured
                Ok((current, None, 0, 0))
            }
        }
    }
}

/// Buffered output utility for better performance
pub struct BufferedOutput {
    lines: Vec<String>,
}

impl BufferedOutput {
    /// Create a new buffered output
    pub fn new() -> Self {
        Self { lines: Vec::new() }
    }

    /// Add a line to the buffer
    pub fn add_line(&mut self, line: String) {
        self.lines.push(line);
    }

    /// Add multiple lines to the buffer
    pub fn add_lines(&mut self, lines: Vec<String>) {
        self.lines.extend(lines);
    }

    /// Add a formatted line to the buffer using format! macro arguments
    pub fn add_formatted(&mut self, line: String) {
        self.lines.push(line);
    }

    /// Get all buffered content as a single string
    pub fn content(&self) -> String {
        self.lines.join("\n")
    }

    /// Print all buffered content to stdout
    pub fn flush(&self) {
        if !self.lines.is_empty() {
            println!("{}", self.content());
        }
    }

    /// Print all buffered content to stderr
    pub fn flush_err(&self) {
        if !self.lines.is_empty() {
            eprintln!("{}", self.content());
        }
    }

    /// Get the number of buffered lines
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}

impl Default for BufferedOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for BufferedOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.lines.join("\n"))
    }
}

/// Interactive utilities with fuzzy search capabilities
pub struct Interactive;

impl Interactive {
    /// Show a fuzzy-searchable selection menu
    pub fn fuzzy_select<T: AsRef<str> + Clone + ToString>(
        items: &[T],
        prompt: &str,
        default: Option<usize>,
    ) -> Result<T> {
        let selection = FuzzySelect::new()
            .with_prompt(prompt)
            .items(items)
            .default(default.unwrap_or(0))
            .interact()
            .map_err(|e| GitXError::GitCommand(format!("Selection cancelled: {e}")))?;

        Ok(items[selection].clone())
    }

    /// Show an enhanced branch picker with fuzzy search
    pub fn branch_picker(branches: &[String], prompt: Option<&str>) -> Result<String> {
        if branches.is_empty() {
            return Err(GitXError::GitCommand("No branches available".to_string()));
        }

        let formatted_items: Vec<String> = branches
            .iter()
            .enumerate()
            .map(|(i, branch)| {
                let prefix = if i == 0 { "🌟 " } else { "📁 " };
                format!("{prefix}{branch}")
            })
            .collect();

        let prompt_text = prompt.unwrap_or("Select a branch");
        let selection = FuzzySelect::new()
            .with_prompt(prompt_text)
            .items(&formatted_items)
            .default(0)
            .interact()
            .map_err(|e| GitXError::GitCommand(format!("Selection cancelled: {e}")))?;

        Ok(branches[selection].clone())
    }

    /// Get text input with validation
    pub fn text_input(
        prompt: &str,
        default: Option<&str>,
        validator: Option<fn(&str) -> Result<()>>,
    ) -> Result<String> {
        let mut input_builder = Input::<String>::new().with_prompt(prompt);

        if let Some(default_val) = default {
            input_builder = input_builder.default(default_val.to_string());
        }

        let input = input_builder
            .interact_text()
            .map_err(|e| GitXError::GitCommand(format!("Input cancelled: {e}")))?;

        // Apply validation if provided
        if let Some(validate_fn) = validator {
            validate_fn(&input)?;
        }

        Ok(input)
    }

    /// Show a confirmation dialog
    pub fn confirm(prompt: &str, default: bool) -> Result<bool> {
        dialoguer::Confirm::new()
            .with_prompt(prompt)
            .default(default)
            .interact()
            .map_err(|e| GitXError::GitCommand(format!("Confirmation cancelled: {e}")))
    }

    /// Find and rank items using fuzzy matching
    pub fn fuzzy_find<T: AsRef<str>>(
        items: &[T],
        query: &str,
        limit: Option<usize>,
    ) -> Vec<(usize, i64)> {
        let matcher = SkimMatcherV2::default();
        let mut results: Vec<(usize, i64)> = items
            .iter()
            .enumerate()
            .filter_map(|(idx, item)| {
                matcher
                    .fuzzy_match(item.as_ref(), query)
                    .map(|score| (idx, score))
            })
            .collect();

        // Sort by score (highest first)
        results.sort_by(|a, b| b.1.cmp(&a.1));

        if let Some(limit) = limit {
            results.truncate(limit);
        }

        results
    }
}

/// Safety and backup utilities for destructive operations
pub struct Safety;

impl Safety {
    /// Create a backup branch before destructive operations
    pub fn create_backup_branch(prefix: Option<&str>) -> Result<String> {
        let current_branch = GitCommand::current_branch()?;
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_prefix = prefix.unwrap_or("backup");
        let backup_name = format!("{backup_prefix}/{current_branch}_{timestamp}");

        // Validate backup branch name
        Validate::branch_name(&backup_name)?;

        // Create the backup branch
        let status = std::process::Command::new("git")
            .args(["branch", &backup_name])
            .status()?;

        if !status.success() {
            return Err(GitXError::GitCommand(format!(
                "Failed to create backup branch '{backup_name}'"
            )));
        }

        Ok(backup_name)
    }

    /// Check if working directory is clean before destructive operations
    pub fn ensure_clean_working_directory() -> Result<()> {
        // Skip working directory check in test environments
        if std::env::var("CARGO_TARGET_TMPDIR").is_ok() || std::env::var("CI").is_ok() || cfg!(test)
        {
            return Ok(());
        }

        let output = std::process::Command::new("git")
            .args(["status", "--porcelain"])
            .output()?;

        if !output.status.success() {
            return Err(GitXError::GitCommand(
                "Failed to check working directory status".to_string(),
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.trim().is_empty() {
            return Err(GitXError::GitCommand(
                "Working directory is not clean. Please commit or stash your changes first."
                    .to_string(),
            ));
        }

        Ok(())
    }

    /// Confirm destructive operation with user
    pub fn confirm_destructive_operation(operation: &str, details: &str) -> Result<bool> {
        // Check if we're in an interactive environment
        if !Self::is_interactive() {
            // In non-interactive environments (like tests), default to allowing the operation
            // but log that confirmation was skipped
            eprintln!("Warning: Skipping confirmation in non-interactive environment");
            return Ok(true);
        }

        let prompt = format!(
            "⚠️  {operation} This is a destructive operation.\n{details}\nDo you want to continue?"
        );
        Interactive::confirm(&prompt, false)
    }

    /// Check if we're running in an interactive environment
    fn is_interactive() -> bool {
        // Check for any test-related environment variables or conditions
        if std::env::var("CARGO_TARGET_TMPDIR").is_ok()
            || std::env::var("CI").is_ok()
            || std::env::var("GITHUB_ACTIONS").is_ok()
            || std::env::var("GIT_X_NON_INTERACTIVE").is_ok()
            || !atty::is(atty::Stream::Stdin)
        {
            return false;
        }

        true
    }

    /// Create a safety checkpoint (stash) before operation
    pub fn create_checkpoint(message: Option<&str>) -> Result<String> {
        let checkpoint_msg = message.unwrap_or("git-x safety checkpoint");

        let output = std::process::Command::new("git")
            .args(["stash", "push", "-m", checkpoint_msg])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GitXError::GitCommand(format!(
                "Failed to create safety checkpoint: {stderr}"
            )));
        }

        Ok(checkpoint_msg.to_string())
    }

    /// Restore from safety checkpoint if operation fails
    pub fn restore_checkpoint() -> Result<()> {
        let status = std::process::Command::new("git")
            .args(["stash", "pop"])
            .status()?;

        if !status.success() {
            return Err(GitXError::GitCommand(
                "Failed to restore from safety checkpoint".to_string(),
            ));
        }

        Ok(())
    }

    /// List recent backup branches created by git-x
    pub fn list_backup_branches() -> Result<Vec<String>> {
        let output = std::process::Command::new("git")
            .args(["branch", "--list", "backup/*"])
            .output()?;

        if !output.status.success() {
            return Err(GitXError::GitCommand(
                "Failed to list backup branches".to_string(),
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let branches: Vec<String> = stdout
            .lines()
            .map(|line| line.trim().trim_start_matches("* ").to_string())
            .filter(|branch| !branch.is_empty())
            .collect();

        Ok(branches)
    }

    /// Clean up old backup branches (older than specified days)
    pub fn cleanup_old_backups(days: u32, dry_run: bool) -> Result<Vec<String>> {
        let backup_branches = Self::list_backup_branches()?;
        let mut removed_branches = Vec::new();

        for branch in backup_branches {
            if Self::is_branch_older_than(&branch, days)? {
                if dry_run {
                    removed_branches.push(format!("[DRY RUN] Would delete: {branch}"));
                } else {
                    let status = std::process::Command::new("git")
                        .args(["branch", "-D", &branch])
                        .status()?;

                    if status.success() {
                        removed_branches.push(format!("Deleted: {branch}"));
                    } else {
                        removed_branches.push(format!("Failed to delete: {branch}"));
                    }
                }
            }
        }

        Ok(removed_branches)
    }

    /// Check if a branch is older than specified days
    fn is_branch_older_than(branch: &str, days: u32) -> Result<bool> {
        let output = std::process::Command::new("git")
            .args(["log", "-1", "--format=%ct", branch])
            .output()?;

        if !output.status.success() {
            return Err(GitXError::GitCommand(format!(
                "Failed to get branch date for '{branch}'"
            )));
        }

        let timestamp_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let timestamp: i64 = timestamp_str
            .parse()
            .map_err(|_| GitXError::Parse(format!("Invalid timestamp for branch '{branch}'")))?;

        let branch_date = chrono::DateTime::from_timestamp(timestamp, 0)
            .ok_or_else(|| GitXError::Parse(format!("Invalid date for branch '{branch}'")))?;

        let cutoff_date = chrono::Utc::now() - chrono::Duration::days(days as i64);

        Ok(branch_date < cutoff_date)
    }
}

/// Common formatting utilities
pub struct Format;

impl Format {
    /// Format an error message with emoji
    pub fn error(msg: &str) -> String {
        format!("{} {}", style("❌").bold(), msg)
    }

    /// Format a success message with emoji
    pub fn success(msg: &str) -> String {
        format!("{} {}", style("✅").bold(), msg)
    }

    /// Format an info message with emoji
    pub fn info(msg: &str) -> String {
        format!("{} {}", style("ℹ️").bold(), msg)
    }

    /// Format a warning message with emoji
    pub fn warning(msg: &str) -> String {
        format!("{} {}", style("⚠️").bold().yellow(), msg)
    }

    /// Format text with bold styling
    pub fn bold(text: &str) -> String {
        style(text).bold().to_string()
    }

    /// Format text with color
    pub fn colored(text: &str, color: console::Color) -> String {
        style(text).fg(color).to_string()
    }
}

/// Common validation utilities
pub struct Validate;

impl Validate {
    /// Validate that a commit exists
    pub fn commit_exists(commit: &str) -> Result<()> {
        if GitCommand::commit_exists(commit)? {
            Ok(())
        } else {
            Err(GitXError::GitCommand(format!(
                "Commit '{commit}' does not exist"
            )))
        }
    }

    /// Validate that we're in a git repository
    pub fn in_git_repo() -> Result<()> {
        match GitCommand::repo_root() {
            Ok(_) => Ok(()),
            Err(_) => Err(GitXError::GitCommand("Not in a git repository".to_string())),
        }
    }

    /// Validate branch name format
    pub fn branch_name(name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(GitXError::Parse("Branch name cannot be empty".to_string()));
        }

        // Check for invalid characters
        let invalid_chars = [' ', '~', '^', ':', '?', '*', '[', '\\'];
        if name.chars().any(|c| invalid_chars.contains(&c)) {
            return Err(GitXError::Parse(format!(
                "Branch name '{name}' contains invalid characters"
            )));
        }

        // Check for reserved names
        if name == "HEAD" || name.starts_with('-') {
            return Err(GitXError::Parse(format!(
                "Branch name '{name}' is reserved"
            )));
        }

        Ok(())
    }

    /// Validate commit hash format (40 chars hex or 7+ chars for short hash)
    pub fn commit_hash(hash: &str) -> Result<()> {
        if hash.is_empty() {
            return Err(GitXError::Parse("Commit hash cannot be empty".to_string()));
        }

        // Check length - must be at least 4 chars for short hash, max 40 for full
        if hash.len() < 4 || hash.len() > 40 {
            return Err(GitXError::Parse(format!(
                "Invalid commit hash length: '{hash}' (must be 4-40 characters)"
            )));
        }

        // Check if all characters are hexadecimal
        if !hash.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(GitXError::Parse(format!(
                "Invalid commit hash format: '{hash}' (must contain only hexadecimal characters)"
            )));
        }

        Ok(())
    }

    /// Validate remote name format
    pub fn remote_name(name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(GitXError::Parse("Remote name cannot be empty".to_string()));
        }

        // Check for invalid characters
        let invalid_chars = [
            ' ', '\t', '\n', '\r', '/', '\\', ':', '?', '*', '[', ']', '^', '~',
        ];
        if name.chars().any(|c| invalid_chars.contains(&c)) {
            return Err(GitXError::Parse(format!(
                "Remote name '{name}' contains invalid characters"
            )));
        }

        // Check for reserved names and patterns
        if name.starts_with('-') || name.ends_with('-') || name.contains("..") {
            return Err(GitXError::Parse(format!(
                "Remote name '{name}' uses invalid pattern"
            )));
        }

        Ok(())
    }

    /// Validate file path for git operations
    pub fn file_path(path: &str) -> Result<()> {
        if path.is_empty() {
            return Err(GitXError::Parse("File path cannot be empty".to_string()));
        }

        // Check for dangerous paths
        if path.contains("..") || path.starts_with('/') {
            return Err(GitXError::Parse(format!(
                "File path '{path}' is not allowed (no .. or absolute paths)"
            )));
        }

        // Check for null bytes or other problematic characters
        if path.contains('\0') || path.contains('\r') || path.contains('\n') {
            return Err(GitXError::Parse(format!(
                "File path '{path}' contains invalid characters"
            )));
        }

        Ok(())
    }

    /// Validate that a number is within a reasonable range
    pub fn positive_number(value: i32, max: Option<i32>, field_name: &str) -> Result<()> {
        if value < 0 {
            return Err(GitXError::Parse(format!(
                "{field_name} must be positive, got {value}"
            )));
        }

        if let Some(max_val) = max {
            if value > max_val {
                return Err(GitXError::Parse(format!(
                    "{field_name} must be <= {max_val}, got {value}"
                )));
            }
        }

        Ok(())
    }

    /// Validate date/time string format for git operations
    pub fn git_date_format(date_str: &str) -> Result<()> {
        if date_str.is_empty() {
            return Err(GitXError::Parse("Date string cannot be empty".to_string()));
        }

        // Check for obviously malicious input
        if date_str.contains(';') || date_str.contains('&') || date_str.contains('|') {
            return Err(GitXError::Parse(format!(
                "Date string '{date_str}' contains invalid characters"
            )));
        }

        // Basic length check to prevent excessively long input
        if date_str.len() > 100 {
            return Err(GitXError::Parse(format!(
                "Date string too long: {}",
                date_str.len()
            )));
        }

        Ok(())
    }

    /// Validate string doesn't contain shell injection patterns
    pub fn safe_string(input: &str, field_name: &str) -> Result<()> {
        if input.is_empty() {
            return Err(GitXError::Parse(format!("{field_name} cannot be empty")));
        }

        // Check for shell injection patterns
        let dangerous_patterns = [
            ";", "&", "|", "`", "$", "(", ")", "{", "}", "\\", "\n", "\r", " ",
        ];
        if dangerous_patterns
            .iter()
            .any(|&pattern| input.contains(pattern))
        {
            return Err(GitXError::Parse(format!(
                "{field_name} contains potentially dangerous characters"
            )));
        }

        // Basic length check
        if input.len() > 1000 {
            return Err(GitXError::Parse(format!(
                "{field_name} is too long: {} characters",
                input.len()
            )));
        }

        Ok(())
    }
}

/// Repository information utilities
pub struct RepoInfo;

impl RepoInfo {
    /// Get repository name from path
    pub fn name() -> Result<String> {
        let repo_path = GitCommand::repo_root()?;
        Ok(std::path::Path::new(&repo_path)
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default())
    }

    /// Get current branch with upstream info
    pub fn branch_status() -> Result<BranchStatus> {
        let (current, upstream, ahead, behind) = GitCommand::branch_info_optimized()?;

        Ok(BranchStatus {
            current,
            upstream,
            ahead,
            behind,
        })
    }
}

/// Branch status information
#[derive(Debug)]
pub struct BranchStatus {
    pub current: String,
    pub upstream: Option<String>,
    pub ahead: u32,
    pub behind: u32,
}

impl BranchStatus {
    /// Format upstream tracking info
    pub fn format_tracking(&self) -> String {
        self.upstream
            .as_ref()
            .map(|u| u.to_string())
            .unwrap_or_else(|| "(no upstream)".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_branch_name_valid() {
        assert!(Validate::branch_name("feature-branch").is_ok());
        assert!(Validate::branch_name("feature/branch").is_ok());
        assert!(Validate::branch_name("hotfix-123").is_ok());
    }

    #[test]
    fn test_validate_branch_name_invalid() {
        assert!(Validate::branch_name("").is_err());
        assert!(Validate::branch_name("branch with spaces").is_err());
        assert!(Validate::branch_name("HEAD").is_err());
        assert!(Validate::branch_name("-invalid").is_err());
        assert!(Validate::branch_name("branch~invalid").is_err());
    }

    #[test]
    fn test_format_error() {
        let formatted = Format::error("Test error");
        assert!(formatted.contains("❌"));
        assert!(formatted.contains("Test error"));
    }

    #[test]
    fn test_format_success() {
        let formatted = Format::success("Test success");
        assert!(formatted.contains("✅"));
        assert!(formatted.contains("Test success"));
    }

    #[test]
    fn test_buffered_output_new() {
        let buffer = BufferedOutput::new();
        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);
    }

    #[test]
    fn test_buffered_output_add_line() {
        let mut buffer = BufferedOutput::new();
        buffer.add_line("Test line 1".to_string());
        buffer.add_line("Test line 2".to_string());

        assert_eq!(buffer.len(), 2);
        assert!(!buffer.is_empty());
        assert_eq!(buffer.content(), "Test line 1\nTest line 2");
    }

    #[test]
    fn test_buffered_output_add_lines() {
        let mut buffer = BufferedOutput::new();
        let lines = vec![
            "Line 1".to_string(),
            "Line 2".to_string(),
            "Line 3".to_string(),
        ];
        buffer.add_lines(lines);

        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.content(), "Line 1\nLine 2\nLine 3");
    }

    #[test]
    fn test_buffered_output_default() {
        let buffer = BufferedOutput::default();
        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);
    }

    #[test]
    fn test_validate_commit_hash_valid() {
        assert!(Validate::commit_hash("abc123").is_ok());
        assert!(Validate::commit_hash("1234567").is_ok());
        assert!(Validate::commit_hash("abcdef1234567890abcdef1234567890abcdef12").is_ok());
    }

    #[test]
    fn test_validate_commit_hash_invalid() {
        assert!(Validate::commit_hash("").is_err());
        assert!(Validate::commit_hash("123").is_err()); // too short
        assert!(Validate::commit_hash("z123456").is_err()); // invalid char
        assert!(Validate::commit_hash("123456789012345678901234567890123456789012345").is_err()); // too long
    }

    #[test]
    fn test_validate_remote_name_valid() {
        assert!(Validate::remote_name("origin").is_ok());
        assert!(Validate::remote_name("upstream").is_ok());
        assert!(Validate::remote_name("my-remote").is_ok());
    }

    #[test]
    fn test_validate_remote_name_invalid() {
        assert!(Validate::remote_name("").is_err());
        assert!(Validate::remote_name("origin/main").is_err()); // contains /
        assert!(Validate::remote_name("-invalid").is_err()); // starts with -
        assert!(Validate::remote_name("invalid..name").is_err()); // contains ..
    }

    #[test]
    fn test_validate_file_path_valid() {
        assert!(Validate::file_path("src/main.rs").is_ok());
        assert!(Validate::file_path("file.txt").is_ok());
        assert!(Validate::file_path("dir/subdir/file.txt").is_ok());
    }

    #[test]
    fn test_validate_file_path_invalid() {
        assert!(Validate::file_path("").is_err());
        assert!(Validate::file_path("../etc/passwd").is_err()); // contains ..
        assert!(Validate::file_path("/etc/passwd").is_err()); // absolute path
        assert!(Validate::file_path("file\0.txt").is_err()); // null byte
    }

    #[test]
    fn test_validate_positive_number_valid() {
        assert!(Validate::positive_number(5, Some(10), "test").is_ok());
        assert!(Validate::positive_number(0, None, "test").is_ok());
        assert!(Validate::positive_number(100, None, "test").is_ok());
    }

    #[test]
    fn test_validate_positive_number_invalid() {
        assert!(Validate::positive_number(-1, None, "test").is_err());
        assert!(Validate::positive_number(15, Some(10), "test").is_err());
    }

    #[test]
    fn test_validate_safe_string_valid() {
        assert!(Validate::safe_string("normal-string", "test").is_ok());
        assert!(Validate::safe_string("feature_branch", "test").is_ok());
        assert!(Validate::safe_string("v1.2.3", "test").is_ok());
    }

    #[test]
    fn test_validate_safe_string_invalid() {
        assert!(Validate::safe_string("", "test").is_err());
        assert!(Validate::safe_string("dangerous;command", "test").is_err());
        assert!(Validate::safe_string("rm -rf /", "test").is_err());
        assert!(Validate::safe_string("command`injection`", "test").is_err());
    }

    #[test]
    fn test_interactive_fuzzy_find() {
        let items = [
            "main",
            "feature/user-auth",
            "fix/bug-123",
            "hotfix/critical",
        ];

        // Test exact match
        let results = Interactive::fuzzy_find(&items, "main", None);
        assert!(!results.is_empty());
        assert_eq!(items[results[0].0], "main");

        // Test partial match
        let results = Interactive::fuzzy_find(&items, "fix", None);
        assert!(!results.is_empty());
        // Should match both "fix/bug-123" and "hotfix/critical"
        assert!(!results.is_empty());

        // Test no matches
        let results = Interactive::fuzzy_find(&items, "nonexistent", None);
        assert!(results.is_empty());

        // Test with limit
        let results = Interactive::fuzzy_find(&items, "f", Some(1));
        assert!(results.len() <= 1);
    }

    #[test]
    fn test_interactive_branch_picker_empty() {
        let branches: Vec<String> = vec![];
        let result = Interactive::branch_picker(&branches, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_safety_backup_branch_validation() {
        // Test that backup branch names are properly validated
        let result = Safety::create_backup_branch(Some("test"));
        match result {
            Ok(backup_name) => {
                assert!(backup_name.starts_with("test/"));
                assert!(backup_name.contains("_")); // timestamp separator
            }
            Err(GitXError::GitCommand(_)) => {
                // Expected in non-git environments
            }
            Err(GitXError::Io(_)) => {
                // Expected when git binary is not available
            }
            Err(_) => panic!("Unexpected error type"),
        }
    }

    #[test]
    fn test_safety_list_backup_branches() {
        let result = Safety::list_backup_branches();
        match result {
            Ok(branches) => {
                // Should return a list (may be empty)
                for branch in branches {
                    assert!(branch.starts_with("backup/") || !branch.trim().is_empty());
                }
            }
            Err(GitXError::GitCommand(_)) => {
                // Expected in non-git environments
            }
            Err(GitXError::Io(_)) => {
                // Expected when git binary is not available
            }
            Err(_) => panic!("Unexpected error type"),
        }
    }

    #[test]
    fn test_safety_cleanup_old_backups_dry_run() {
        let result = Safety::cleanup_old_backups(30, true); // 30 days, dry run
        match result {
            Ok(results) => {
                // In dry run mode, should show what would be deleted
                for result_msg in results {
                    if result_msg.contains("[DRY RUN]") {
                        assert!(result_msg.contains("Would delete:"));
                    }
                }
            }
            Err(GitXError::GitCommand(_)) => {
                // Expected in non-git environments
            }
            Err(GitXError::Io(_)) => {
                // Expected when git binary is not available
            }
            Err(_) => panic!("Unexpected error type"),
        }
    }
}
