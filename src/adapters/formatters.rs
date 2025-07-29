use crate::core::output::Format;
use crate::domain::{
    BranchCreationResult, BranchSwitchResult, CleanBranchesResult, HealthLevel, HealthStatus,
    RepositoryInfo,
};

/// Formatter for branch operations
pub struct BranchFormatter;

impl BranchFormatter {
    pub fn new() -> Self {
        Self
    }

    /// Format branch creation result
    pub fn format_creation_result(&self, result: &BranchCreationResult) -> String {
        let mut output = Format::success(&format!(
            "Created and switched to branch '{}'",
            result.branch_name
        ));

        if let Some(ref backup) = result.backup_branch {
            output.push_str(&format!("\n💾 Backup created: {backup}"));
        }

        if let Some(ref base) = result.base_commit {
            output.push_str(&format!("\n📍 Based on: {base}"));
        }

        output
    }

    /// Format clean branches result
    pub fn format_clean_result(&self, result: &CleanBranchesResult) -> String {
        if result.candidates.is_empty() {
            return "No merged branches to delete.".to_string();
        }

        if result.dry_run {
            let mut output = format!(
                "🧪 (dry run) {} branches would be deleted:\n",
                result.candidates.len()
            );
            for branch in &result.candidates {
                output.push_str(&format!("(dry run) Would delete: {branch}\n"));
            }
            output
        } else {
            let mut output = format!("🧹 Deleted {} merged branches:\n", result.deleted.len());
            for branch in &result.deleted {
                output.push_str(&format!("✅ Deleted: {branch}\n"));
            }

            if !result.failed.is_empty() {
                output.push_str(&format!(
                    "\n❌ Failed to delete {} branches:\n",
                    result.failed.len()
                ));
                for branch in &result.failed {
                    output.push_str(&format!("❌ Failed: {branch}\n"));
                }
            }

            output
        }
    }

    /// Format branch switch result
    pub fn format_switch_result(&self, result: &BranchSwitchResult) -> String {
        let mut output = Format::success(&format!("Switched to branch '{}'", result.new_branch));

        if let Some(ref checkpoint) = result.checkpoint {
            output.push_str(&format!("\n💾 Checkpoint created: {checkpoint}"));
        }

        output
    }
}

/// Formatter for repository operations
pub struct RepositoryFormatter;

impl RepositoryFormatter {
    pub fn new() -> Self {
        Self
    }

    /// Format repository information
    pub fn format_repository_info(&self, info: &RepositoryInfo, detailed: bool) -> String {
        let mut output = String::new();

        output.push_str(&format!("🗂️  Repository: {}\n", Format::bold(&info.name)));
        output.push_str(&format!(
            "📍 Current branch: {}\n",
            Format::bold(&info.current_branch)
        ));

        // Upstream information
        if let Some(ref upstream) = info.upstream_branch {
            if info.is_in_sync() {
                output.push_str(&format!("🔗 Upstream: {upstream} (up to date)\n"));
            } else {
                let mut status_parts = Vec::new();
                if info.ahead_count > 0 {
                    status_parts.push(format!("{} ahead", info.ahead_count));
                }
                if info.behind_count > 0 {
                    status_parts.push(format!("{} behind", info.behind_count));
                }
                output.push_str(&format!(
                    "🔗 Upstream: {} ({})\n",
                    upstream,
                    status_parts.join(", ")
                ));
            }
        } else {
            output.push_str("❌ No upstream configured\n");
        }

        // Working directory status
        if info.is_clean {
            output.push_str("✅ Working directory: Clean\n");
        } else {
            output.push_str("⚠️  Working directory: Has changes\n");
        }

        // Staged files
        if info.staged_files_count == 0 {
            output.push_str("📋 Staged files: None\n");
        } else {
            output.push_str(&format!(
                "📋 Staged files: {} file(s)\n",
                info.staged_files_count
            ));
        }

        // Additional details if requested
        if detailed {
            output.push_str(&format!("\n📂 Root path: {}\n", info.root_path));
            output.push_str(&format!("📊 Status: {}\n", info.status_description()));
        }

        output
    }

    /// Format health status
    pub fn format_health_status(&self, health: &HealthStatus) -> String {
        let mut output = String::new();

        output.push_str("🏥 Repository Health Check\n");
        output.push_str(&"=".repeat(30));
        output.push('\n');

        // Overall status
        match health.level {
            HealthLevel::Healthy => {
                output.push_str(&Format::success("Repository is healthy!"));
            }
            HealthLevel::Warning => {
                output.push_str(&Format::warning(&format!(
                    "Repository has {} warning(s)",
                    health.warnings.len()
                )));
            }
            HealthLevel::Unhealthy => {
                output.push_str(&Format::error(&format!(
                    "Repository has {} issue(s)",
                    health.issues.len()
                )));
            }
        }

        output.push('\n');

        // List issues
        if !health.issues.is_empty() {
            output.push_str("\n🚨 Issues:\n");
            for issue in &health.issues {
                output.push_str(&format!("   ❌ {issue}\n"));
            }
        }

        // List warnings
        if !health.warnings.is_empty() {
            output.push_str("\n⚠️  Warnings:\n");
            for warning in &health.warnings {
                output.push_str(&format!("   ⚠️  {warning}\n"));
            }
        }

        // Summary
        output.push_str(&format!("\n📋 Summary: {}\n", health.summary()));

        output
    }
}

/// Formatter for analysis operations
pub struct AnalysisFormatter;

impl AnalysisFormatter {
    pub fn new() -> Self {
        Self
    }

    /// Format commit statistics
    pub fn format_commit_stats(&self, total_commits: u32, period: &str) -> String {
        Format::info(&format!("📈 {total_commits} commits in {period}"))
    }

    /// Format contributor information
    pub fn format_contributors(&self, contributors: &[(String, u32)]) -> String {
        let mut output = String::new();
        output.push_str("👥 Top Contributors:\n");

        for (i, (name, count)) in contributors.iter().enumerate() {
            let prefix = match i {
                0 => "🥇",
                1 => "🥈",
                2 => "🥉",
                _ => "👤",
            };
            output.push_str(&format!("   {prefix} {name} ({count} commits)\n"));
        }

        output
    }
}

/// Generic formatter utilities
pub struct FormatterUtils;

impl FormatterUtils {
    /// Create a section header
    pub fn section_header(title: &str) -> String {
        format!("{}\n{}\n", title, "=".repeat(title.len()))
    }

    /// Create a subsection header
    pub fn subsection_header(title: &str) -> String {
        format!("\n{}\n{}\n", title, "-".repeat(title.len()))
    }

    /// Format a list with bullets
    pub fn bullet_list(items: &[String], bullet: &str) -> String {
        items
            .iter()
            .map(|item| format!("{bullet} {item}"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Format a numbered list
    pub fn numbered_list(items: &[String]) -> String {
        items
            .iter()
            .enumerate()
            .map(|(i, item)| format!("{}. {}", i + 1, item))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

// Implement Default for formatters
impl Default for BranchFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for RepositoryFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for AnalysisFormatter {
    fn default() -> Self {
        Self::new()
    }
}
