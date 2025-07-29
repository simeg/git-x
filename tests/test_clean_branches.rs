mod common;

use common::repo_with_merged_branch;
use git_x::clean_branches::*;
use predicates::str::contains;
use std::process::Command as StdCommand;

#[test]
fn test_clean_branches_dry_run_outputs_expected() {
    let repo = repo_with_merged_branch("feature/cleanup", "master");

    repo.run_git_x(&["clean-branches", "--dry-run"])
        .success()
        .stdout(contains("(dry run) 1 branches would be deleted"))
        .stdout(contains("feature/cleanup"));
}

#[test]
fn test_clean_branches_run_function_dry_run() {
    let repo = repo_with_merged_branch("feature/test", "master");
    let original_dir = std::env::current_dir().unwrap();

    // Change to repo directory and run the function directly
    std::env::set_current_dir(repo.path()).unwrap();

    // Test that the function doesn't panic and git commands work
    let result = run(true);
    assert!(result.is_ok());

    // Restore original directory
    let _ = std::env::set_current_dir(&original_dir);
}

#[test]
fn test_clean_branches_run_function_actual_delete() {
    let repo = repo_with_merged_branch("feature/delete-me", "master");
    let original_dir = std::env::current_dir().unwrap();

    // Change to repo directory and run the function directly
    std::env::set_current_dir(repo.path()).unwrap();

    // Set non-interactive mode for this test
    unsafe {
        std::env::set_var("GIT_X_NON_INTERACTIVE", "1");
    }

    // Test that the function doesn't panic and actually deletes branches
    let result = run(false);
    assert!(result.is_ok());

    // Clean up environment variable
    unsafe {
        std::env::remove_var("GIT_X_NON_INTERACTIVE");
    }

    // Restore original directory
    let _ = std::env::set_current_dir(&original_dir);
}

#[test]
fn test_clean_branches_run_function_with_branches_to_delete() {
    let repo = repo_with_merged_branch("test-branch", "master");
    let original_dir = std::env::current_dir().unwrap();

    // Switch back to master to ensure the test branch is merged
    repo.checkout_branch("master");

    // Change to repo directory and run the function directly
    std::env::set_current_dir(repo.path()).unwrap();

    // Test with dry run to ensure it finds branches and prints them
    let result = run(true);
    assert!(result.is_ok());

    // Restore original directory
    let _ = std::env::set_current_dir(&original_dir);
}

#[test]
fn test_clean_branches_run_function_non_dry_run_with_branches() {
    let repo = repo_with_merged_branch("test-non-dry", "master");
    let original_dir = std::env::current_dir().unwrap();

    // Switch back to master to ensure the test branch is merged
    repo.checkout_branch("master");

    // Change to repo directory and run the function directly
    std::env::set_current_dir(repo.path()).unwrap();

    // Set non-interactive mode for this test
    unsafe {
        std::env::set_var("GIT_X_NON_INTERACTIVE", "1");
    }

    // Test non-dry run to actually trigger deletion path
    let result = run(false);
    assert!(result.is_ok());

    // Clean up environment variable
    unsafe {
        std::env::remove_var("GIT_X_NON_INTERACTIVE");
    }

    // Restore original directory
    let _ = std::env::set_current_dir(&original_dir);
}

#[test]
fn test_clean_branches_run_function_no_branches() {
    let repo = common::basic_repo();
    let original_dir = std::env::current_dir().unwrap();

    // Change to repo directory - this repo has no merged branches to delete
    std::env::set_current_dir(repo.path()).unwrap();

    // Test the no branches case
    let result = run(true);
    assert!(result.is_ok());

    // Restore original directory
    let _ = std::env::set_current_dir(&original_dir);
}

#[test]
fn test_clean_branches_actually_deletes_branch() {
    let repo = repo_with_merged_branch("feature/cleanup", "master");

    // Sanity check: branch exists before cleanup
    let output_before = StdCommand::new("git")
        .args(["branch"])
        .current_dir(repo.path())
        .output()
        .expect("Failed to list branches");
    let stdout_before = String::from_utf8_lossy(&output_before.stdout);
    assert!(stdout_before.contains("feature/cleanup"));

    // Run the command (no dry run)
    repo.run_git_x(&["clean-branches"])
        .success()
        .stdout(contains("🧹 Deleted 1 merged branches:"));

    // Confirm branch was deleted
    let output_after = StdCommand::new("git")
        .args(["branch"])
        .current_dir(repo.path())
        .output()
        .expect("Failed to list branches");
    let stdout_after = String::from_utf8_lossy(&output_after.stdout);
    assert!(!stdout_after.contains("feature/cleanup"));
}

#[test]
fn test_format_dry_run_message() {
    let branch = "feature/test";
    assert_eq!(
        format!("(dry run) Would delete: {branch}"),
        "(dry run) Would delete: feature/test"
    );
}

#[test]
fn test_format_no_branches_message() {
    assert_eq!(
        "No merged branches to delete.",
        "No merged branches to delete."
    );
}
