mod common;

use common::repo_with_branch;
use git_x::rename_branch::*;
use std::process::Command;

#[test]
fn test_rename_branch_in_isolated_repo() {
    let repo = repo_with_branch("test-branch");

    // Rename the branch from test-branch to renamed-branch
    let status = Command::new("git")
        .args(["branch", "-m", "renamed-branch"])
        .current_dir(repo.path())
        .status()
        .expect("Failed to rename branch");
    assert!(status.success());

    // Verify the current branch name
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(repo.path())
        .output()
        .expect("Failed to get current branch");

    assert!(output.status.success());
    let current_branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(current_branch, "renamed-branch");
}

// Unit tests for helper functions
#[test]
fn test_get_current_branch_args() {
    assert_eq!(
        get_current_branch_args(),
        ["rev-parse", "--abbrev-ref", "HEAD"]
    );
}

#[test]
fn test_is_branch_already_named() {
    assert!(is_branch_already_named("main", "main"));
    assert!(is_branch_already_named("feature", "feature"));
    assert!(!is_branch_already_named("main", "develop"));
    assert!(!is_branch_already_named("feature/test", "hotfix/test"));
}

#[test]
fn test_get_local_rename_args() {
    assert_eq!(
        get_local_rename_args("new-branch"),
        vec![
            "branch".to_string(),
            "-m".to_string(),
            "new-branch".to_string()
        ]
    );
    assert_eq!(
        get_local_rename_args("feature/awesome"),
        vec![
            "branch".to_string(),
            "-m".to_string(),
            "feature/awesome".to_string()
        ]
    );
}

#[test]
fn test_get_push_new_branch_args() {
    assert_eq!(
        get_push_new_branch_args("new-branch"),
        vec![
            "push".to_string(),
            "-u".to_string(),
            "origin".to_string(),
            "new-branch".to_string()
        ]
    );
}

#[test]
fn test_get_delete_old_branch_args() {
    assert_eq!(
        get_delete_old_branch_args("old-branch"),
        vec![
            "push".to_string(),
            "origin".to_string(),
            "--delete".to_string(),
            "old-branch".to_string()
        ]
    );
}

#[test]
fn test_format_already_named_message() {
    assert_eq!(
        format_already_named_message("main"),
        "Current branch is already named 'main'. Nothing to do."
    );
    assert_eq!(
        format_already_named_message("feature/test"),
        "Current branch is already named 'feature/test'. Nothing to do."
    );
}

#[test]
fn test_format_rename_start_message() {
    assert_eq!(
        format_rename_start_message("old-branch", "new-branch"),
        "Renaming branch 'old-branch' to 'new-branch'"
    );
    assert_eq!(
        format_rename_start_message("feature/old", "feature/new"),
        "Renaming branch 'feature/old' to 'feature/new'"
    );
}

#[test]
fn test_format_delete_failed_message() {
    assert_eq!(
        format_delete_failed_message("old-branch"),
        "Warning: Failed to delete old branch 'old-branch' from origin."
    );
}

#[test]
fn test_format_delete_success_message() {
    assert_eq!(
        format_delete_success_message("old-branch"),
        "Deleted old branch 'old-branch' from origin."
    );
}

#[test]
fn test_format_rename_success_message() {
    assert_eq!(
        format_rename_success_message(),
        "Branch renamed successfully."
    );
}
