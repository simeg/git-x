mod common;

use assert_cmd::Command as AssertCmd;
use common::repo_with_branch;
use git_x::rename_branch::*;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

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

// Helper function to create a test git repository
fn create_test_repo() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let repo_path = temp_dir.path().to_path_buf();

    // Initialize git repo
    Command::new("git")
        .args(["init"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to init repo");

    // Configure git
    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to configure user");

    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to configure email");

    // Create initial commit
    fs::write(repo_path.join("README.md"), "Initial commit").expect("Failed to write file");
    Command::new("git")
        .args(["add", "README.md"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to add file");

    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to commit");

    (temp_dir, repo_path)
}

#[test]
fn test_rename_branch_run_outside_git_repo() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let mut cmd = AssertCmd::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["rename-branch", "new-name"])
        .current_dir(temp_dir.path())
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(
            "Failed to get current branch name",
        ));
}

#[test]
fn test_rename_branch_same_name() {
    let (_temp_dir, repo_path) = create_test_repo();

    // Get current branch name (should be main or master)
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to get current branch");

    let current_branch = String::from_utf8_lossy(&output.stdout).trim().to_string();

    let mut cmd = AssertCmd::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["rename-branch", &current_branch])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "Current branch is already named '{current_branch}'. Nothing to do."
        )));
}

#[test]
fn test_rename_branch_local_rename_failure() {
    let (_temp_dir, repo_path) = create_test_repo();

    // Create a branch with an invalid name that would cause rename to fail
    // Use invalid characters that git doesn't allow
    let mut cmd = AssertCmd::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["rename-branch", "branch..with..double..dots"])
        .current_dir(&repo_path)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Failed to rename local branch"));
}

#[test]
fn test_rename_branch_command_help() {
    let mut cmd = AssertCmd::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["rename-branch", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Rename the current branch"));
}

#[test]
fn test_rename_branch_push_failure() {
    let (_temp_dir, repo_path) = create_test_repo();

    // Add a fake remote that doesn't exist to cause push failure
    Command::new("git")
        .args([
            "remote",
            "add",
            "origin",
            "https://github.com/nonexistent/repo.git",
        ])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to add remote");

    let mut cmd = AssertCmd::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["rename-branch", "new-branch-name"])
        .current_dir(&repo_path)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(
            "Failed to push new branch to origin",
        ));
}

#[test]
fn test_rename_branch_edge_cases() {
    // Test various edge cases in formatting and validation

    // Test empty branch name handling
    assert_eq!(
        format_already_named_message(""),
        "Current branch is already named ''. Nothing to do."
    );

    // Test special characters in branch names
    assert_eq!(
        format_rename_start_message("feature/test-123", "hotfix/urgent_fix"),
        "Renaming branch 'feature/test-123' to 'hotfix/urgent_fix'"
    );

    // Test long branch names
    let long_name = "very-long-branch-name-that-exceeds-normal-length";
    assert_eq!(
        format_delete_success_message(long_name),
        format!("Deleted old branch '{long_name}' from origin.")
    );
}

#[test]
fn test_rename_branch_args_completeness() {
    // Ensure all argument generation functions produce valid arrays
    let current_args = get_current_branch_args();
    assert_eq!(current_args.len(), 3);
    assert!(current_args.contains(&"rev-parse"));
    assert!(current_args.contains(&"--abbrev-ref"));
    assert!(current_args.contains(&"HEAD"));

    let local_rename_args = get_local_rename_args("test");
    assert_eq!(local_rename_args.len(), 3);
    assert!(local_rename_args.contains(&"branch".to_string()));
    assert!(local_rename_args.contains(&"-m".to_string()));
    assert!(local_rename_args.contains(&"test".to_string()));

    let push_args = get_push_new_branch_args("test");
    assert_eq!(push_args.len(), 4);
    assert!(push_args.contains(&"push".to_string()));
    assert!(push_args.contains(&"-u".to_string()));
    assert!(push_args.contains(&"origin".to_string()));
    assert!(push_args.contains(&"test".to_string()));

    let delete_args = get_delete_old_branch_args("old");
    assert_eq!(delete_args.len(), 4);
    assert!(delete_args.contains(&"push".to_string()));
    assert!(delete_args.contains(&"origin".to_string()));
    assert!(delete_args.contains(&"--delete".to_string()));
    assert!(delete_args.contains(&"old".to_string()));
}

// Direct run() function tests for maximum coverage

// Note: Direct run() function tests for rename_branch are challenging because
// the function calls exit(1) on certain failures, which terminates the test process.
// The functionality is well covered by CLI integration tests instead.

#[test]
fn test_rename_branch_run_function_successful_case() {
    let repo = repo_with_branch("test-branch");

    std::env::set_current_dir(repo.path()).expect("Failed to change directory");

    // Test the case where branch already has the desired name (returns early, no exit)
    git_x::rename_branch::run("test-branch");

    std::env::set_current_dir("/").expect("Failed to reset directory");
}

#[test]
fn test_rename_branch_run_function_same_name() {
    let repo = repo_with_branch("test-branch");

    std::env::set_current_dir(repo.path()).expect("Failed to change directory");

    // Test run function with same name (should return early)
    git_x::rename_branch::run("test-branch");

    std::env::set_current_dir("/").expect("Failed to reset directory");
}

// Note: test_rename_branch_run_function_outside_git_repo was removed because
// rename_branch::run() calls exit(1) on git command failures, which terminates
// the test process. This behavior is tested via CLI integration tests instead.
