mod common;

use common::repo_with_commits;
use git_x::undo::*;
use predicates::str::contains;
use std::process::Command;

#[test]
fn test_git_xundo_soft_resets_last_commit() {
    let repo = repo_with_commits(2);

    repo.run_git_x(&["undo"])
        .success()
        .stdout(contains("Last commit undone"));

    // Verify that the commit was undone but the file changes remain
    let log = Command::new("git")
        .args(["log", "--oneline"])
        .current_dir(repo.path())
        .output()
        .unwrap();
    let log_output = String::from_utf8_lossy(&log.stdout);
    assert!(log_output.contains("initial"));
    assert!(!log_output.contains("commit 2"));

    let diff = Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .current_dir(repo.path())
        .output()
        .unwrap();
    let diff_output = String::from_utf8_lossy(&diff.stdout);
    assert!(diff_output.contains("file.txt"));
}

// Unit tests for helper functions
#[test]
fn test_get_git_reset_args() {
    assert_eq!(get_git_reset_args(), ["reset", "--soft", "HEAD~1"]);
}

#[test]
fn test_format_success_message() {
    assert_eq!(
        format_success_message(),
        "Last commit undone (soft reset). Changes kept in working directory."
    );
}

#[test]
fn test_format_error_message() {
    assert_eq!(format_error_message(), "❌ Failed to undo last commit.");
}

#[test]
fn test_undo_run_function() {
    let repo = repo_with_commits(3);

    // Change to repo directory and run the function directly
    std::env::set_current_dir(repo.path()).unwrap();

    // Test that the function returns Ok and git commands work
    let result = git_x::undo::run();
    assert!(result.is_ok());
    assert!(result.unwrap().contains("Last commit undone"));
}

#[test]
fn test_undo_run_function_git_error() {
    let temp_dir = tempfile::tempdir().unwrap();

    // Change to non-git directory to trigger error path
    std::env::set_current_dir(temp_dir.path()).unwrap();

    // Test that the function returns an error for non-git directory
    let result = git_x::undo::run();
    assert!(result.is_err());
}
