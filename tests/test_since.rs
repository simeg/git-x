mod common;

use common::repo_with_commits;
use predicates::str::contains;
use git_x::since::*;

#[test]
fn test_git_xsince_outputs_commits_since_ref() {
    let repo = repo_with_commits(2);

    repo.run_git_x(&["since", "HEAD~1"])
        .success()
        .stdout(contains("🔍 Commits since HEAD~1:"))
        .stdout(contains("commit 2"));
}

#[test]
fn test_git_xsince_no_new_commits() {
    let repo = repo_with_commits(2);

    repo.run_git_x(&["since", "HEAD"])
        .success()
        .stdout(contains("✅ No new commits since HEAD"));
}

// Unit tests for helper functions
#[test]
fn test_format_git_log_range() {
    assert_eq!(format_git_log_range("main"), "main..HEAD");
    assert_eq!(format_git_log_range("HEAD~1"), "HEAD~1..HEAD");
    assert_eq!(format_git_log_range("origin/main"), "origin/main..HEAD");
}

#[test]
fn test_is_log_empty() {
    assert!(is_log_empty(""));
    assert!(is_log_empty("   \n\n  "));
    assert!(!is_log_empty("some log output"));
    assert!(!is_log_empty("commit abc123"));
}
