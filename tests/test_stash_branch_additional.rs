// Additional tests for stash_branch.rs to increase coverage

use git_x::stash_branch::*;

#[test]
fn test_extract_branch_from_message_coverage() {
    // Test different message formats
    assert_eq!(extract_branch_from_message("On main: some changes"), "main");
    assert_eq!(
        extract_branch_from_message("On feature/test: work in progress"),
        "feature/test"
    );
    assert_eq!(
        extract_branch_from_message("WIP on develop: quick fix"),
        "develop"
    );
    assert_eq!(
        extract_branch_from_message("WIP on feature/branch-123: updates"),
        "feature/branch-123"
    );

    // Test edge cases
    assert_eq!(extract_branch_from_message(""), "unknown");
    assert_eq!(extract_branch_from_message("No branch info"), "unknown");
    assert_eq!(extract_branch_from_message("On "), "unknown");
    assert_eq!(extract_branch_from_message("WIP on "), "unknown");
    assert_eq!(extract_branch_from_message("On no-colon"), "unknown");
    assert_eq!(extract_branch_from_message("WIP on no-colon"), "unknown");
}

#[test]
fn test_filter_stashes_by_age_coverage() {
    let sample_stashes = vec![
        StashInfo {
            name: "stash@{0}".to_string(),
            message: "Test stash 1".to_string(),
            branch: "main".to_string(),
            timestamp: "2023-01-01".to_string(),
        },
        StashInfo {
            name: "stash@{1}".to_string(),
            message: "Test stash 2".to_string(),
            branch: "develop".to_string(),
            timestamp: "2023-01-02".to_string(),
        },
    ];

    // Test valid age formats
    assert!(filter_stashes_by_age(&sample_stashes, "7d").is_ok());
    assert!(filter_stashes_by_age(&sample_stashes, "2w").is_ok());
    assert!(filter_stashes_by_age(&sample_stashes, "1m").is_ok());
    // Test that strings ending with the characters but being "invalid" are still ok per implementation
    assert!(filter_stashes_by_age(&sample_stashes, "invalidd").is_ok());
    assert!(filter_stashes_by_age(&sample_stashes, "testw").is_ok());
    assert!(filter_stashes_by_age(&sample_stashes, "xm").is_ok());

    // Test invalid age format (anything that doesn't end with d, w, or m)
    assert!(filter_stashes_by_age(&sample_stashes, "invalidx").is_err());
    assert!(filter_stashes_by_age(&sample_stashes, "7").is_err());
    assert!(filter_stashes_by_age(&sample_stashes, "").is_err());

    // Test that valid formats return the input stashes (placeholder implementation)
    let result = filter_stashes_by_age(&sample_stashes, "7d").unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].name, "stash@{0}");
}

#[test]
fn test_format_error_message_coverage() {
    assert_eq!(format_error_message("test error"), "❌ test error");
    assert_eq!(format_error_message(""), "❌ ");
    assert_eq!(
        format_error_message("Branch validation failed"),
        "❌ Branch validation failed"
    );
}

#[test]
fn test_format_branch_exists_message_coverage() {
    assert_eq!(
        format_branch_exists_message("main"),
        "❌ Branch 'main' already exists"
    );
    assert_eq!(
        format_branch_exists_message("feature/test"),
        "❌ Branch 'feature/test' already exists"
    );
    assert_eq!(
        format_branch_exists_message(""),
        "❌ Branch '' already exists"
    );
}

#[test]
fn test_format_creating_branch_message_coverage() {
    assert_eq!(
        format_creating_branch_message("test-branch", "stash@{0}"),
        "🌿 Creating branch 'test-branch' from stash@{0}..."
    );
    assert_eq!(
        format_creating_branch_message("feature/awesome", "stash@{1}"),
        "🌿 Creating branch 'feature/awesome' from stash@{1}..."
    );
    assert_eq!(
        format_creating_branch_message("", ""),
        "🌿 Creating branch '' from ..."
    );
}

#[test]
fn test_format_branch_created_message_coverage() {
    assert_eq!(
        format_branch_created_message("test-branch"),
        "✅ Branch 'test-branch' created and checked out"
    );
    assert_eq!(
        format_branch_created_message("feature/test"),
        "✅ Branch 'feature/test' created and checked out"
    );
    assert_eq!(
        format_branch_created_message(""),
        "✅ Branch '' created and checked out"
    );
}

#[test]
fn test_format_static_messages_coverage() {
    assert_eq!(format_no_stashes_message(), "ℹ️ No stashes found");
    assert_eq!(
        format_no_old_stashes_message(),
        "✅ No old stashes to clean"
    );
}

#[test]
fn test_format_stashes_to_clean_message_coverage() {
    assert_eq!(
        format_stashes_to_clean_message(5, true),
        "🧪 (dry run) Would clean 5 stash(es):"
    );
    assert_eq!(
        format_stashes_to_clean_message(3, false),
        "🧹 Cleaning 3 stash(es):"
    );
    assert_eq!(
        format_stashes_to_clean_message(0, true),
        "🧪 (dry run) Would clean 0 stash(es):"
    );
    assert_eq!(
        format_stashes_to_clean_message(1, false),
        "🧹 Cleaning 1 stash(es):"
    );
}

#[test]
fn test_format_cleanup_complete_message_coverage() {
    assert_eq!(format_cleanup_complete_message(5), "✅ Cleaned 5 stash(es)");
    assert_eq!(format_cleanup_complete_message(0), "✅ Cleaned 0 stash(es)");
    assert_eq!(format_cleanup_complete_message(1), "✅ Cleaned 1 stash(es)");
}

#[test]
fn test_format_branch_specific_messages_coverage() {
    assert_eq!(
        format_no_stashes_for_branch_message("main"),
        "ℹ️ No stashes found for branch 'main'"
    );
    assert_eq!(
        format_no_stashes_for_branch_message("feature/test"),
        "ℹ️ No stashes found for branch 'feature/test'"
    );

    assert_eq!(
        format_stashes_for_branch_header("main", 3),
        "📋 Found 3 stash(es) for branch 'main':"
    );
    assert_eq!(
        format_stashes_for_branch_header("develop", 1),
        "📋 Found 1 stash(es) for branch 'develop':"
    );

    assert_eq!(
        format_applying_stashes_message("main", 2),
        "🔄 Applying 2 stash(es) from branch 'main':"
    );
    assert_eq!(
        format_applying_stashes_message("feature/test", 5),
        "🔄 Applying 5 stash(es) from branch 'feature/test':"
    );
}

#[test]
fn test_format_stash_entry_coverage() {
    assert_eq!(
        format_stash_entry("stash@{0}", "WIP on main: quick fix"),
        "stash@{0}: WIP on main: quick fix"
    );
    assert_eq!(
        format_stash_entry("stash@{1}", "On feature/test: work in progress"),
        "stash@{1}: On feature/test: work in progress"
    );
    assert_eq!(format_stash_entry("", ""), ": ");
}

#[test]
fn test_git_args_functions_coverage() {
    let branch_args = get_git_stash_branch_args();
    assert_eq!(branch_args.len(), 2);
    assert_eq!(branch_args[0], "stash");
    assert_eq!(branch_args[1], "branch");

    let drop_args = get_git_stash_drop_args();
    assert_eq!(drop_args.len(), 2);
    assert_eq!(drop_args[0], "stash");
    assert_eq!(drop_args[1], "drop");
}

#[test]
fn test_stash_info_struct_coverage() {
    // Test StashInfo struct construction and field access
    let stash = StashInfo {
        name: "stash@{0}".to_string(),
        message: "Test message".to_string(),
        branch: "main".to_string(),
        timestamp: "2023-01-01 12:00:00".to_string(),
    };

    assert_eq!(stash.name, "stash@{0}");
    assert_eq!(stash.message, "Test message");
    assert_eq!(stash.branch, "main");
    assert_eq!(stash.timestamp, "2023-01-01 12:00:00");

    // Test with empty values
    let empty_stash = StashInfo {
        name: "".to_string(),
        message: "".to_string(),
        branch: "".to_string(),
        timestamp: "".to_string(),
    };

    assert_eq!(empty_stash.name, "");
    assert_eq!(empty_stash.message, "");
    assert_eq!(empty_stash.branch, "");
    assert_eq!(empty_stash.timestamp, "");
}

#[test]
fn test_message_formatting_consistency() {
    // Test that all format functions return non-empty strings for reasonable inputs
    assert!(!format_error_message("test").is_empty());
    assert!(!format_branch_exists_message("test").is_empty());
    assert!(!format_creating_branch_message("test", "stash@{0}").is_empty());
    assert!(!format_branch_created_message("test").is_empty());
    assert!(!format_cleanup_complete_message(1).is_empty());
    assert!(!format_no_stashes_for_branch_message("test").is_empty());
    assert!(!format_stashes_for_branch_header("test", 1).is_empty());
    assert!(!format_applying_stashes_message("test", 1).is_empty());
    assert!(!format_stash_entry("stash@{0}", "message").is_empty());

    // Test that they include expected emojis or symbols
    assert!(format_error_message("test").contains("❌"));
    assert!(format_branch_exists_message("test").contains("❌"));
    assert!(format_creating_branch_message("test", "stash@{0}").contains("🌿"));
    assert!(format_branch_created_message("test").contains("✅"));
    assert!(format_cleanup_complete_message(1).contains("✅"));
    assert!(format_no_stashes_for_branch_message("test").contains("ℹ️"));
    assert!(format_stashes_for_branch_header("test", 1).contains("📋"));
    assert!(format_applying_stashes_message("test", 1).contains("🔄"));
}
