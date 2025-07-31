use git_x::adapters::formatters::{
    AnalysisFormatter, BranchFormatter, FormatterUtils, RepositoryFormatter,
};
use git_x::domain::{
    BranchCreationResult, BranchSwitchResult, CleanBranchesResult, HealthLevel, HealthStatus,
    RepositoryInfo,
};

// Helper function to strip ANSI escape codes for testing
fn strip_ansi_codes(text: &str) -> String {
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
fn test_branch_formatter_new() {
    let _formatter = BranchFormatter::new();
    // Just verify it can be created
}

#[test]
fn test_branch_formatter_default() {
    let _formatter = BranchFormatter;
}

#[test]
fn test_branch_formatter_format_creation_result_minimal() {
    let formatter = BranchFormatter::new();
    let result = BranchCreationResult {
        branch_name: "feature-branch".to_string(),
        base_commit: None,
        backup_branch: None,
        switched: true,
    };

    let output = formatter.format_creation_result(&result);
    assert!(output.contains("Created and switched to branch 'feature-branch'"));
    assert!(!output.contains("💾"));
    assert!(!output.contains("📍"));
}

#[test]
fn test_branch_formatter_format_creation_result_with_backup() {
    let formatter = BranchFormatter::new();
    let result = BranchCreationResult {
        branch_name: "feature-branch".to_string(),
        base_commit: None,
        backup_branch: Some("backup-12345".to_string()),
        switched: true,
    };

    let output = formatter.format_creation_result(&result);
    assert!(output.contains("Created and switched to branch 'feature-branch'"));
    assert!(output.contains("💾 Backup created: backup-12345"));
    assert!(!output.contains("📍"));
}

#[test]
fn test_branch_formatter_format_creation_result_with_base_commit() {
    let formatter = BranchFormatter::new();
    let result = BranchCreationResult {
        branch_name: "feature-branch".to_string(),
        base_commit: Some("abc123".to_string()),
        backup_branch: None,
        switched: true,
    };

    let output = formatter.format_creation_result(&result);
    assert!(output.contains("Created and switched to branch 'feature-branch'"));
    assert!(!output.contains("💾"));
    assert!(output.contains("📍 Based on: abc123"));
}

#[test]
fn test_branch_formatter_format_creation_result_complete() {
    let formatter = BranchFormatter::new();
    let result = BranchCreationResult {
        branch_name: "feature-branch".to_string(),
        base_commit: Some("abc123".to_string()),
        backup_branch: Some("backup-12345".to_string()),
        switched: true,
    };

    let output = formatter.format_creation_result(&result);
    assert!(output.contains("Created and switched to branch 'feature-branch'"));
    assert!(output.contains("💾 Backup created: backup-12345"));
    assert!(output.contains("📍 Based on: abc123"));
}

#[test]
fn test_branch_formatter_format_clean_result_no_candidates() {
    let formatter = BranchFormatter::new();
    let result = CleanBranchesResult {
        candidates: vec![],
        deleted: vec![],
        failed: vec![],
        dry_run: false,
    };

    let output = formatter.format_clean_result(&result);
    assert_eq!(output, "No merged branches to delete.");
}

#[test]
fn test_branch_formatter_format_clean_result_dry_run() {
    let formatter = BranchFormatter::new();
    let result = CleanBranchesResult {
        candidates: vec!["branch1".to_string(), "branch2".to_string()],
        deleted: vec![],
        failed: vec![],
        dry_run: true,
    };

    let output = formatter.format_clean_result(&result);
    assert!(output.contains("🧪 (dry run) 2 branches would be deleted:"));
    assert!(output.contains("(dry run) Would delete: branch1"));
    assert!(output.contains("(dry run) Would delete: branch2"));
}

#[test]
fn test_branch_formatter_format_clean_result_successful_deletion() {
    let formatter = BranchFormatter::new();
    let result = CleanBranchesResult {
        candidates: vec!["branch1".to_string(), "branch2".to_string()],
        deleted: vec!["branch1".to_string(), "branch2".to_string()],
        failed: vec![],
        dry_run: false,
    };

    let output = formatter.format_clean_result(&result);
    assert!(output.contains("🧹 Deleted 2 merged branches:"));
    assert!(output.contains("✅ Deleted: branch1"));
    assert!(output.contains("✅ Deleted: branch2"));
    assert!(!output.contains("❌ Failed"));
}

#[test]
fn test_branch_formatter_format_clean_result_with_failures() {
    let formatter = BranchFormatter::new();
    let result = CleanBranchesResult {
        candidates: vec!["branch1".to_string(), "branch2".to_string()],
        deleted: vec!["branch1".to_string()],
        failed: vec!["branch2".to_string()],
        dry_run: false,
    };

    let output = formatter.format_clean_result(&result);
    assert!(output.contains("🧹 Deleted 1 merged branches:"));
    assert!(output.contains("✅ Deleted: branch1"));
    assert!(output.contains("❌ Failed to delete 1 branches:"));
    assert!(output.contains("❌ Failed: branch2"));
}

#[test]
fn test_branch_formatter_format_switch_result_minimal() {
    let formatter = BranchFormatter::new();
    let result = BranchSwitchResult {
        previous_branch: "main".to_string(),
        new_branch: "feature-branch".to_string(),
        checkpoint: None,
    };

    let output = formatter.format_switch_result(&result);
    assert!(output.contains("Switched to branch 'feature-branch'"));
    assert!(!output.contains("💾"));
}

#[test]
fn test_branch_formatter_format_switch_result_with_checkpoint() {
    let formatter = BranchFormatter::new();
    let result = BranchSwitchResult {
        previous_branch: "main".to_string(),
        new_branch: "feature-branch".to_string(),
        checkpoint: Some("checkpoint-12345".to_string()),
    };

    let output = formatter.format_switch_result(&result);
    assert!(output.contains("Switched to branch 'feature-branch'"));
    assert!(output.contains("💾 Checkpoint created: checkpoint-12345"));
}

#[test]
fn test_repository_formatter_new() {
    let _formatter = RepositoryFormatter::new();
}

#[test]
fn test_repository_formatter_default() {
    let _formatter = RepositoryFormatter;
}

#[test]
fn test_repository_formatter_format_repository_info_basic() {
    let formatter = RepositoryFormatter::new();
    let info = RepositoryInfo {
        name: "my-repo".to_string(),
        root_path: "/path/to/repo".to_string(),
        current_branch: "main".to_string(),
        upstream_branch: None,
        ahead_count: 0,
        behind_count: 0,
        is_clean: true,
        staged_files_count: 0,
    };

    let output = formatter.format_repository_info(&info, false);
    let clean_output = strip_ansi_codes(&output);
    assert!(clean_output.contains("🗂️  Repository: my-repo"));
    assert!(clean_output.contains("📍 Current branch: main"));
    assert!(clean_output.contains("❌ No upstream configured"));
    assert!(clean_output.contains("✅ Working directory: Clean"));
    assert!(clean_output.contains("📋 Staged files: None"));

    // Should not contain detailed info
    assert!(!clean_output.contains("📂 Root path:"));
    assert!(!clean_output.contains("📊 Status:"));
}

#[test]
fn test_repository_formatter_format_repository_info_detailed() {
    let formatter = RepositoryFormatter::new();
    let info = RepositoryInfo {
        name: "my-repo".to_string(),
        root_path: "/path/to/repo".to_string(),
        current_branch: "main".to_string(),
        upstream_branch: Some("origin/main".to_string()),
        ahead_count: 2,
        behind_count: 1,
        is_clean: false,
        staged_files_count: 3,
    };

    let output = formatter.format_repository_info(&info, true);
    let clean_output = strip_ansi_codes(&output);
    assert!(clean_output.contains("🗂️  Repository: my-repo"));
    assert!(clean_output.contains("📍 Current branch: main"));
    assert!(clean_output.contains("🔗 Upstream: origin/main (2 ahead, 1 behind)"));
    assert!(clean_output.contains("⚠️  Working directory: Has changes"));
    assert!(clean_output.contains("📋 Staged files: 3 file(s)"));

    // Should contain detailed info
    assert!(clean_output.contains("📂 Root path: /path/to/repo"));
    assert!(clean_output.contains("📊 Status:"));
}

#[test]
fn test_repository_formatter_format_repository_info_in_sync() {
    let formatter = RepositoryFormatter::new();
    let info = RepositoryInfo {
        name: "my-repo".to_string(),
        root_path: "/path/to/repo".to_string(),
        current_branch: "main".to_string(),
        upstream_branch: Some("origin/main".to_string()),
        ahead_count: 0,
        behind_count: 0,
        is_clean: true,
        staged_files_count: 0,
    };

    let output = formatter.format_repository_info(&info, false);
    let clean_output = strip_ansi_codes(&output);
    assert!(clean_output.contains("🔗 Upstream: origin/main (up to date)"));
}

#[test]
fn test_repository_formatter_format_repository_info_only_ahead() {
    let formatter = RepositoryFormatter::new();
    let info = RepositoryInfo {
        name: "my-repo".to_string(),
        root_path: "/path/to/repo".to_string(),
        current_branch: "feature".to_string(),
        upstream_branch: Some("origin/feature".to_string()),
        ahead_count: 3,
        behind_count: 0,
        is_clean: true,
        staged_files_count: 0,
    };

    let output = formatter.format_repository_info(&info, false);
    let clean_output = strip_ansi_codes(&output);
    assert!(clean_output.contains("🔗 Upstream: origin/feature (3 ahead)"));
}

#[test]
fn test_repository_formatter_format_repository_info_only_behind() {
    let formatter = RepositoryFormatter::new();
    let info = RepositoryInfo {
        name: "my-repo".to_string(),
        root_path: "/path/to/repo".to_string(),
        current_branch: "feature".to_string(),
        upstream_branch: Some("origin/feature".to_string()),
        ahead_count: 0,
        behind_count: 2,
        is_clean: true,
        staged_files_count: 0,
    };

    let output = formatter.format_repository_info(&info, false);
    let clean_output = strip_ansi_codes(&output);
    assert!(clean_output.contains("🔗 Upstream: origin/feature (2 behind)"));
}

#[test]
fn test_repository_formatter_format_health_status_healthy() {
    let formatter = RepositoryFormatter::new();
    let health = HealthStatus {
        level: HealthLevel::Healthy,
        issues: vec![],
        warnings: vec![],
    };

    let output = formatter.format_health_status(&health);
    assert!(output.contains("🏥 Repository Health Check"));
    assert!(output.contains("=============================="));
    assert!(output.contains("Repository is healthy!"));
    assert!(output.contains("📋 Summary: Repository is healthy"));
    assert!(!output.contains("🚨 Issues:"));
    assert!(!output.contains("⚠️  Warnings:"));
}

#[test]
fn test_repository_formatter_format_health_status_warning() {
    let formatter = RepositoryFormatter::new();
    let health = HealthStatus {
        level: HealthLevel::Warning,
        issues: vec![],
        warnings: vec![
            "No remotes configured".to_string(),
            "Too many branches".to_string(),
        ],
    };

    let output = formatter.format_health_status(&health);
    assert!(output.contains("🏥 Repository Health Check"));
    assert!(output.contains("Repository has 2 warning(s)"));
    assert!(output.contains("⚠️  Warnings:"));
    assert!(output.contains("⚠️  No remotes configured"));
    assert!(output.contains("⚠️  Too many branches"));
    assert!(!output.contains("🚨 Issues:"));
}

#[test]
fn test_repository_formatter_format_health_status_unhealthy() {
    let formatter = RepositoryFormatter::new();
    let health = HealthStatus {
        level: HealthLevel::Unhealthy,
        issues: vec![
            "Git user.name not configured".to_string(),
            "Git user.email not configured".to_string(),
        ],
        warnings: vec!["No remotes configured".to_string()],
    };

    let output = formatter.format_health_status(&health);
    assert!(output.contains("🏥 Repository Health Check"));
    assert!(output.contains("Repository has 2 issue(s)"));
    assert!(output.contains("🚨 Issues:"));
    assert!(output.contains("❌ Git user.name not configured"));
    assert!(output.contains("❌ Git user.email not configured"));
    assert!(output.contains("⚠️  Warnings:"));
    assert!(output.contains("⚠️  No remotes configured"));
}

#[test]
fn test_analysis_formatter_new() {
    let _formatter = AnalysisFormatter::new();
}

#[test]
fn test_analysis_formatter_default() {
    let _formatter = AnalysisFormatter;
}

#[test]
fn test_analysis_formatter_format_commit_stats() {
    let formatter = AnalysisFormatter::new();
    let output = formatter.format_commit_stats(42, "last 30 days");
    assert!(output.contains("📈 42 commits in last 30 days"));
}

#[test]
fn test_analysis_formatter_format_commit_stats_zero() {
    let formatter = AnalysisFormatter::new();
    let output = formatter.format_commit_stats(0, "last week");
    assert!(output.contains("📈 0 commits in last week"));
}

#[test]
fn test_analysis_formatter_format_contributors_empty() {
    let formatter = AnalysisFormatter::new();
    let contributors = vec![];
    let output = formatter.format_contributors(&contributors);
    assert!(output.contains("👥 Top Contributors:"));
    // Should not contain any contributor entries
    assert!(!output.contains("🥇"));
    assert!(!output.contains("🥈"));
    assert!(!output.contains("🥉"));
    assert!(!output.contains("👤"));
}

#[test]
fn test_analysis_formatter_format_contributors_single() {
    let formatter = AnalysisFormatter::new();
    let contributors = vec![("Alice".to_string(), 10)];
    let output = formatter.format_contributors(&contributors);
    assert!(output.contains("👥 Top Contributors:"));
    assert!(output.contains("🥇 Alice (10 commits)"));
    assert!(!output.contains("🥈"));
    assert!(!output.contains("🥉"));
    assert!(!output.contains("👤"));
}

#[test]
fn test_analysis_formatter_format_contributors_top_three() {
    let formatter = AnalysisFormatter::new();
    let contributors = vec![
        ("Alice".to_string(), 20),
        ("Bob".to_string(), 15),
        ("Charlie".to_string(), 10),
    ];
    let output = formatter.format_contributors(&contributors);
    assert!(output.contains("👥 Top Contributors:"));
    assert!(output.contains("🥇 Alice (20 commits)"));
    assert!(output.contains("🥈 Bob (15 commits)"));
    assert!(output.contains("🥉 Charlie (10 commits)"));
    assert!(!output.contains("👤"));
}

#[test]
fn test_analysis_formatter_format_contributors_many() {
    let formatter = AnalysisFormatter::new();
    let contributors = vec![
        ("Alice".to_string(), 20),
        ("Bob".to_string(), 15),
        ("Charlie".to_string(), 10),
        ("David".to_string(), 8),
        ("Eve".to_string(), 5),
    ];
    let output = formatter.format_contributors(&contributors);
    assert!(output.contains("👥 Top Contributors:"));
    assert!(output.contains("🥇 Alice (20 commits)"));
    assert!(output.contains("🥈 Bob (15 commits)"));
    assert!(output.contains("🥉 Charlie (10 commits)"));
    assert!(output.contains("👤 David (8 commits)"));
    assert!(output.contains("👤 Eve (5 commits)"));
}

#[test]
fn test_formatter_utils_section_header() {
    let output = FormatterUtils::section_header("Test Section");
    assert_eq!(output, "Test Section\n============\n");
}

#[test]
fn test_formatter_utils_section_header_long() {
    let output = FormatterUtils::section_header("This is a very long section header");
    assert_eq!(
        output,
        "This is a very long section header\n==================================\n"
    );
}

#[test]
fn test_formatter_utils_section_header_empty() {
    let output = FormatterUtils::section_header("");
    assert_eq!(output, "\n\n");
}

#[test]
fn test_formatter_utils_subsection_header() {
    let output = FormatterUtils::subsection_header("Subsection");
    assert_eq!(output, "\nSubsection\n----------\n");
}

#[test]
fn test_formatter_utils_subsection_header_long() {
    let output = FormatterUtils::subsection_header("This is a long subsection");
    assert_eq!(
        output,
        "\nThis is a long subsection\n-------------------------\n"
    );
}

#[test]
fn test_formatter_utils_subsection_header_empty() {
    let output = FormatterUtils::subsection_header("");
    assert_eq!(output, "\n\n\n");
}

#[test]
fn test_formatter_utils_bullet_list_empty() {
    let items = vec![];
    let output = FormatterUtils::bullet_list(&items, "•");
    assert_eq!(output, "");
}

#[test]
fn test_formatter_utils_bullet_list_single() {
    let items = vec!["First item".to_string()];
    let output = FormatterUtils::bullet_list(&items, "•");
    assert_eq!(output, "• First item");
}

#[test]
fn test_formatter_utils_bullet_list_multiple() {
    let items = vec![
        "First item".to_string(),
        "Second item".to_string(),
        "Third item".to_string(),
    ];
    let output = FormatterUtils::bullet_list(&items, "•");
    assert_eq!(output, "• First item\n• Second item\n• Third item");
}

#[test]
fn test_formatter_utils_bullet_list_custom_bullet() {
    let items = vec!["Item 1".to_string(), "Item 2".to_string()];
    let output = FormatterUtils::bullet_list(&items, "→");
    assert_eq!(output, "→ Item 1\n→ Item 2");
}

#[test]
fn test_formatter_utils_numbered_list_empty() {
    let items = vec![];
    let output = FormatterUtils::numbered_list(&items);
    assert_eq!(output, "");
}

#[test]
fn test_formatter_utils_numbered_list_single() {
    let items = vec!["First item".to_string()];
    let output = FormatterUtils::numbered_list(&items);
    assert_eq!(output, "1. First item");
}

#[test]
fn test_formatter_utils_numbered_list_multiple() {
    let items = vec![
        "First item".to_string(),
        "Second item".to_string(),
        "Third item".to_string(),
    ];
    let output = FormatterUtils::numbered_list(&items);
    assert_eq!(output, "1. First item\n2. Second item\n3. Third item");
}

#[test]
fn test_formatter_utils_numbered_list_ten_items() {
    let items: Vec<String> = (1..=10).map(|i| format!("Item {i}")).collect();
    let output = FormatterUtils::numbered_list(&items);
    assert!(output.contains("1. Item 1"));
    assert!(output.contains("5. Item 5"));
    assert!(output.contains("10. Item 10"));
}

// Edge case tests

#[test]
fn test_branch_formatter_empty_branch_names() {
    let formatter = BranchFormatter::new();
    let result = BranchCreationResult {
        branch_name: "".to_string(),
        base_commit: None,
        backup_branch: None,
        switched: true,
    };

    let output = formatter.format_creation_result(&result);
    assert!(output.contains("Created and switched to branch ''"));
}

#[test]
fn test_repository_formatter_special_characters_in_repo_name() {
    let formatter = RepositoryFormatter::new();
    let info = RepositoryInfo {
        name: "my-repo_with-special.chars".to_string(),
        root_path: "/path/to/repo".to_string(),
        current_branch: "feature/test-123".to_string(),
        upstream_branch: None,
        ahead_count: 0,
        behind_count: 0,
        is_clean: true,
        staged_files_count: 0,
    };

    let output = formatter.format_repository_info(&info, false);
    let clean_output = strip_ansi_codes(&output);
    assert!(clean_output.contains("my-repo_with-special.chars"));
    assert!(clean_output.contains("feature/test-123"));
}

#[test]
fn test_analysis_formatter_large_commit_counts() {
    let formatter = AnalysisFormatter::new();
    let output = formatter.format_commit_stats(9999, "all time");
    assert!(output.contains("📈 9999 commits in all time"));
}

#[test]
fn test_analysis_formatter_contributors_single_commit() {
    let formatter = AnalysisFormatter::new();
    let contributors = vec![("SingleCommitter".to_string(), 1)];
    let output = formatter.format_contributors(&contributors);
    assert!(output.contains("🥇 SingleCommitter (1 commits)"));
}

#[test]
fn test_formatter_utils_bullet_list_with_empty_strings() {
    let items = vec!["".to_string(), "Non-empty".to_string(), "".to_string()];
    let output = FormatterUtils::bullet_list(&items, "•");
    assert_eq!(output, "• \n• Non-empty\n• ");
}

#[test]
fn test_formatter_utils_numbered_list_with_empty_strings() {
    let items = vec!["".to_string(), "Item".to_string()];
    let output = FormatterUtils::numbered_list(&items);
    assert_eq!(output, "1. \n2. Item");
}

// Test Default trait implementations

#[test]
fn test_all_formatter_defaults() {
    let _branch_formatter: BranchFormatter = Default::default();
    let _repo_formatter: RepositoryFormatter = Default::default();
    let _analysis_formatter: AnalysisFormatter = Default::default();
}
