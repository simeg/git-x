// Additional tests for sync.rs to increase coverage

use git_x::sync::*;

#[test]
fn test_sync_status_enum_coverage() {
    // Test enum variants for complete coverage
    let up_to_date = SyncStatus::UpToDate;
    let behind = SyncStatus::Behind(5);
    let ahead = SyncStatus::Ahead(3);
    let diverged = SyncStatus::Diverged(2, 4);

    // Test Debug formatting (if derived)
    let _ = format!("{up_to_date:?}");
    let _ = format!("{behind:?}");
    let _ = format!("{ahead:?}");
    let _ = format!("{diverged:?}");

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
}

#[test]
fn test_additional_parse_sync_counts_edge_cases() {
    // Test more edge cases for parse_sync_counts to increase coverage
    assert!(parse_sync_counts("").is_err());
    assert!(parse_sync_counts("invalid").is_err());
    assert!(parse_sync_counts("1").is_err());
    assert!(parse_sync_counts("abc\tdef").is_err());
    assert!(parse_sync_counts("-1\t2").is_err());
    assert!(parse_sync_counts("1\t-2").is_err());
    assert!(parse_sync_counts("999999999999999999999\t1").is_err());

    // Test valid formats
    assert_eq!(parse_sync_counts("0\t0").unwrap(), (0, 0));
    assert_eq!(parse_sync_counts("10\t20").unwrap(), (10, 20));
    assert_eq!(parse_sync_counts("1\t1").unwrap(), (1, 1));
}

#[test]
fn test_format_message_variations() {
    // Test format functions with different inputs for better coverage
    assert_eq!(
        format_behind_message(0),
        "⬇️ Branch is 0 commit(s) behind upstream"
    );
    assert_eq!(
        format_behind_message(1),
        "⬇️ Branch is 1 commit(s) behind upstream"
    );
    assert_eq!(
        format_behind_message(100),
        "⬇️ Branch is 100 commit(s) behind upstream"
    );

    assert_eq!(
        format_ahead_message(0),
        "⬆️ Branch is 0 commit(s) ahead of upstream"
    );
    assert_eq!(
        format_ahead_message(1),
        "⬆️ Branch is 1 commit(s) ahead of upstream"
    );
    assert_eq!(
        format_ahead_message(999),
        "⬆️ Branch is 999 commit(s) ahead of upstream"
    );

    assert_eq!(
        format_diverged_message(0, 0),
        "🔀 Branch has diverged: 0 behind, 0 ahead"
    );
    assert_eq!(
        format_diverged_message(1, 1),
        "🔀 Branch has diverged: 1 behind, 1 ahead"
    );
    assert_eq!(
        format_diverged_message(10, 5),
        "🔀 Branch has diverged: 10 behind, 5 ahead"
    );

    assert_eq!(
        format_sync_success_message(true),
        "✅ Successfully merged upstream changes"
    );
    assert_eq!(
        format_sync_success_message(false),
        "✅ Successfully rebased onto upstream"
    );

    assert!(format_diverged_help_message().contains("handle"));
    assert!(format_up_to_date_message().contains("up to date"));
}

#[test]
fn test_sync_start_message_variations() {
    // Test different branch name combinations
    assert!(format_sync_start_message("main", "origin/main").contains("main"));
    assert!(format_sync_start_message("main", "origin/main").contains("origin/main"));

    assert!(format_sync_start_message("feature", "origin/feature").contains("feature"));
    assert!(format_sync_start_message("", "").contains(""));

    let result = format_sync_start_message("test-branch", "upstream/test-branch");
    assert!(result.contains("test-branch"));
    assert!(result.contains("upstream/test-branch"));
}

#[test]
fn test_error_message_format_coverage() {
    // Test error message formatting with various inputs
    assert_eq!(format_error_message("test error"), "❌ test error");
    assert_eq!(format_error_message(""), "❌ ");
    assert_eq!(
        format_error_message("Network timeout"),
        "❌ Network timeout"
    );
    assert_eq!(
        format_error_message("Git command failed"),
        "❌ Git command failed"
    );
}
