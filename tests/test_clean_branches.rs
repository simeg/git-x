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
        .stdout(contains("(dry run) Would delete: feature/cleanup"));
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

// Unit tests for helper functions
#[test]
fn test_get_git_branch_args() {
    assert_eq!(get_git_branch_args(), ["branch", "--merged"]);
}

#[test]
fn test_get_protected_branches() {
    let protected = get_protected_branches();
    assert_eq!(protected, vec!["main", "master", "develop"]);
}

#[test]
fn test_clean_branch_name() {
    assert_eq!(clean_branch_name("  feature/test  "), "feature/test");
    assert_eq!(clean_branch_name("* main"), "main");
    assert_eq!(clean_branch_name("  * develop  "), "develop");
    assert_eq!(clean_branch_name("bugfix/123"), "bugfix/123");
}

#[test]
fn test_is_protected_branch() {
    assert!(is_protected_branch("main"));
    assert!(is_protected_branch("master"));
    assert!(is_protected_branch("develop"));
    assert!(!is_protected_branch("feature/test"));
    assert!(!is_protected_branch("hotfix/123"));
}

#[test]
fn test_get_git_delete_args() {
    assert_eq!(
        get_git_delete_args("feature"),
        vec![
            "branch".to_string(),
            "-d".to_string(),
            "feature".to_string()
        ]
    );
}

#[test]
fn test_format_dry_run_message() {
    assert_eq!(
        format_dry_run_message("feature/test"),
        "(dry run) Would delete: feature/test"
    );
}

#[test]
fn test_format_no_branches_message() {
    assert_eq!(
        format_no_branches_message(),
        "No merged branches to delete."
    );
}

#[test]
fn test_format_deletion_summary() {
    assert_eq!(
        format_deletion_summary(3, true),
        "🧪 (dry run) 3 branches would be deleted:"
    );
    assert_eq!(
        format_deletion_summary(2, false),
        "🧹 Deleted 2 merged branches:"
    );
}
