mod common;

use common::repo_with_commits;
use git_x::since::*;
use predicates::str::contains;

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

#[test]
fn test_since_run_function() {
    let repo = repo_with_commits(3);

    // Change to repo directory and run the function directly
    std::env::set_current_dir(repo.path()).unwrap();

    // Test that the function returns Ok and git commands work
    let result = git_x::since::run("HEAD~1".to_string());
    assert!(result.is_ok());
    assert!(result.unwrap().contains("🔍 Commits since HEAD~1:"));
}

#[test]
fn test_since_run_function_no_commits() {
    let repo = common::basic_repo();

    // Change to repo directory and run the function directly
    std::env::set_current_dir(repo.path()).unwrap();

    // Test with a reference that should show no commits
    let result = git_x::since::run("HEAD".to_string());
    assert!(result.is_ok());
    assert!(result.unwrap().contains("✅ No new commits since HEAD"));
}

#[test]
fn test_since_run_function_git_error() {
    let temp_dir = tempfile::tempdir().unwrap();

    // Change to non-git directory to trigger error path
    std::env::set_current_dir(temp_dir.path()).unwrap();

    // Test that the function returns an error for non-git directory
    let result = git_x::since::run("HEAD".to_string());
    assert!(result.is_err());
}
