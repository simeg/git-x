// Additional tests for upstream.rs to increase coverage

use git_x::upstream::*;

#[test]
fn test_format_error_message_coverage() {
    assert_eq!(format_error_message("test error"), "❌ test error");
    assert_eq!(format_error_message(""), "❌ ");
    assert_eq!(
        format_error_message("Upstream validation failed"),
        "❌ Upstream validation failed"
    );
}

#[test]
fn test_format_setting_upstream_message_coverage() {
    assert_eq!(
        format_setting_upstream_message("main", "origin/main"),
        "🔗 Setting upstream for 'main' to 'origin/main'..."
    );
    assert_eq!(
        format_setting_upstream_message("feature/test", "upstream/feature/test"),
        "🔗 Setting upstream for 'feature/test' to 'upstream/feature/test'..."
    );
    assert_eq!(
        format_setting_upstream_message("", ""),
        "🔗 Setting upstream for '' to ''..."
    );
}

#[test]
fn test_format_upstream_set_message_coverage() {
    assert_eq!(
        format_upstream_set_message("main", "origin/main"),
        "✅ Upstream for 'main' set to 'origin/main'"
    );
    assert_eq!(
        format_upstream_set_message("develop", "origin/develop"),
        "✅ Upstream for 'develop' set to 'origin/develop'"
    );
    assert_eq!(
        format_upstream_set_message("", ""),
        "✅ Upstream for '' set to ''"
    );
}

#[test]
fn test_format_static_messages_coverage() {
    assert_eq!(format_no_branches_message(), "ℹ️ No local branches found");
    assert_eq!(
        format_upstream_status_header(),
        "🔗 Upstream status for all branches:\n"
    );
    assert_eq!(
        format_no_upstream_branches_message(),
        "ℹ️ No branches with upstream configuration found"
    );
    assert_eq!(format_sync_results_header(), "\n📊 Sync results:");
}

#[test]
fn test_format_branch_with_upstream_coverage() {
    // Test all sync status variants
    assert_eq!(
        format_branch_with_upstream("main", "origin/main", &SyncStatus::UpToDate, true),
        "* main -> origin/main (✅ up-to-date)"
    );
    assert_eq!(
        format_branch_with_upstream("feature", "origin/feature", &SyncStatus::Behind(3), false),
        "  feature -> origin/feature (⬇️ 3 behind)"
    );
    assert_eq!(
        format_branch_with_upstream("develop", "origin/develop", &SyncStatus::Ahead(2), true),
        "* develop -> origin/develop (⬆️ 2 ahead)"
    );
    assert_eq!(
        format_branch_with_upstream("test", "origin/test", &SyncStatus::Diverged(1, 4), false),
        "  test -> origin/test (🔀 1 behind, 4 ahead)"
    );
    assert_eq!(
        format_branch_with_upstream("branch", "origin/branch", &SyncStatus::Unknown, true),
        "* branch -> origin/branch (❓ unknown)"
    );
}

#[test]
fn test_format_branch_without_upstream_coverage() {
    assert_eq!(
        format_branch_without_upstream("main", true),
        "* main -> (no upstream)"
    );
    assert_eq!(
        format_branch_without_upstream("feature", false),
        "  feature -> (no upstream)"
    );
    assert_eq!(
        format_branch_without_upstream("", true),
        "*  -> (no upstream)"
    );
}

#[test]
fn test_format_sync_all_start_message_coverage() {
    // Test with merge and dry run combinations
    assert_eq!(
        format_sync_all_start_message(3, true, true),
        "🧪 (dry run) Would sync 3 branch(es) with upstream using merge:"
    );
    assert_eq!(
        format_sync_all_start_message(5, false, true),
        "🔄 Syncing 5 branch(es) with upstream using merge:"
    );
    assert_eq!(
        format_sync_all_start_message(2, true, false),
        "🧪 (dry run) Would sync 2 branch(es) with upstream using rebase:"
    );
    assert_eq!(
        format_sync_all_start_message(1, false, false),
        "🔄 Syncing 1 branch(es) with upstream using rebase:"
    );
    assert_eq!(
        format_sync_all_start_message(0, true, true),
        "🧪 (dry run) Would sync 0 branch(es) with upstream using merge:"
    );
}

#[test]
fn test_format_sync_result_line_coverage() {
    // Test all SyncResult variants
    assert_eq!(
        format_sync_result_line("main", &SyncResult::UpToDate),
        "  ✅ main: already up-to-date"
    );
    assert_eq!(
        format_sync_result_line("feature", &SyncResult::Synced),
        "  ✅ feature: synced successfully"
    );
    assert_eq!(
        format_sync_result_line("develop", &SyncResult::WouldSync),
        "  🔄 develop: would be synced"
    );
    assert_eq!(
        format_sync_result_line("test", &SyncResult::Ahead),
        "  ⬆️ test: ahead of upstream (skipped)"
    );
    assert_eq!(
        format_sync_result_line("broken", &SyncResult::Error("merge conflict".to_string())),
        "  ❌ broken: merge conflict"
    );
    assert_eq!(
        format_sync_result_line("", &SyncResult::Error("".to_string())),
        "  ❌ : "
    );
}

#[test]
fn test_format_sync_summary_coverage() {
    assert_eq!(
        format_sync_summary(5, true),
        "\n💡 Would sync 5 branch(es). Run without --dry-run to apply changes."
    );
    assert_eq!(
        format_sync_summary(3, false),
        "\n✅ Synced 3 branch(es) successfully."
    );
    assert_eq!(
        format_sync_summary(0, true),
        "\n💡 Would sync 0 branch(es). Run without --dry-run to apply changes."
    );
    assert_eq!(
        format_sync_summary(1, false),
        "\n✅ Synced 1 branch(es) successfully."
    );
}

#[test]
fn test_git_args_functions_coverage() {
    let args = get_git_branch_set_upstream_args();
    assert_eq!(args.len(), 2);
    assert_eq!(args[0], "branch");
    assert_eq!(args[1], "--set-upstream-to");
}

#[test]
fn test_sync_status_enum_coverage() {
    // Test SyncStatus enum variants for Debug formatting
    let up_to_date = SyncStatus::UpToDate;
    let behind = SyncStatus::Behind(5);
    let ahead = SyncStatus::Ahead(3);
    let diverged = SyncStatus::Diverged(2, 4);
    let unknown = SyncStatus::Unknown;

    // Test Debug formatting (if derived)
    let _ = format!("{up_to_date:?}");
    let _ = format!("{behind:?}");
    let _ = format!("{ahead:?}");
    let _ = format!("{diverged:?}");
    let _ = format!("{unknown:?}");

    // Test pattern matching coverage
    match up_to_date {
        SyncStatus::UpToDate => {}
        _ => panic!("Should be UpToDate"),
    }

    match behind {
        SyncStatus::Behind(n) => assert_eq!(n, 5),
        _ => panic!("Should be Behind"),
    }

    match ahead {
        SyncStatus::Ahead(n) => assert_eq!(n, 3),
        _ => panic!("Should be Ahead"),
    }

    match diverged {
        SyncStatus::Diverged(b, a) => {
            assert_eq!(b, 2);
            assert_eq!(a, 4);
        }
        _ => panic!("Should be Diverged"),
    }

    match unknown {
        SyncStatus::Unknown => {}
        _ => panic!("Should be Unknown"),
    }
}

#[test]
fn test_sync_result_enum_coverage() {
    // Test SyncResult enum variants for Debug formatting
    let up_to_date = SyncResult::UpToDate;
    let synced = SyncResult::Synced;
    let would_sync = SyncResult::WouldSync;
    let ahead = SyncResult::Ahead;
    let error = SyncResult::Error("test error".to_string());

    // Test Debug formatting (if derived)
    let _ = format!("{up_to_date:?}");
    let _ = format!("{synced:?}");
    let _ = format!("{would_sync:?}");
    let _ = format!("{ahead:?}");
    let _ = format!("{error:?}");

    // Test pattern matching coverage
    match up_to_date {
        SyncResult::UpToDate => {}
        _ => panic!("Should be UpToDate"),
    }

    match synced {
        SyncResult::Synced => {}
        _ => panic!("Should be Synced"),
    }

    match would_sync {
        SyncResult::WouldSync => {}
        _ => panic!("Should be WouldSync"),
    }

    match ahead {
        SyncResult::Ahead => {}
        _ => panic!("Should be Ahead"),
    }

    match error {
        SyncResult::Error(msg) => assert_eq!(msg, "test error"),
        _ => panic!("Should be Error"),
    }
}

#[test]
fn test_message_formatting_consistency() {
    // Test that all format functions return non-empty strings for reasonable inputs
    assert!(!format_error_message("test").is_empty());
    assert!(!format_setting_upstream_message("test", "origin/test").is_empty());
    assert!(!format_upstream_set_message("test", "origin/test").is_empty());
    assert!(
        !format_branch_with_upstream("test", "origin/test", &SyncStatus::UpToDate, false)
            .is_empty()
    );
    assert!(!format_branch_without_upstream("test", false).is_empty());
    assert!(!format_sync_all_start_message(1, false, false).is_empty());
    assert!(!format_sync_result_line("test", &SyncResult::Synced).is_empty());
    assert!(!format_sync_summary(1, false).is_empty());

    // Test that they include expected emojis or symbols
    assert!(format_error_message("test").contains("❌"));
    assert!(format_setting_upstream_message("test", "origin/test").contains("🔗"));
    assert!(format_upstream_set_message("test", "origin/test").contains("✅"));
    assert!(
        format_branch_with_upstream("test", "origin/test", &SyncStatus::UpToDate, false)
            .contains("✅")
    );
    assert!(format_sync_all_start_message(1, false, false).contains("🔄"));
    assert!(format_sync_result_line("test", &SyncResult::Synced).contains("✅"));
    assert!(format_sync_summary(1, false).contains("✅"));
}

#[test]
fn test_format_edge_cases() {
    // Test with special characters and edge cases
    assert!(
        format_setting_upstream_message("feature/branch-123", "origin/feature/branch-123")
            .contains("feature/branch-123")
    );
    assert!(
        format_upstream_set_message("hotfix/urgent", "upstream/hotfix/urgent")
            .contains("hotfix/urgent")
    );

    let result = format_branch_with_upstream(
        "test-branch",
        "upstream/test-branch",
        &SyncStatus::Ahead(10),
        true,
    );
    assert!(result.contains("test-branch"));
    assert!(result.contains("upstream/test-branch"));
    assert!(result.contains("* "));
    assert!(result.contains("10 ahead"));

    let result = format_sync_all_start_message(999, true, true);
    assert!(result.contains("999"));
    assert!(result.contains("dry run"));
    assert!(result.contains("merge"));
}
