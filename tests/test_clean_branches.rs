mod common;

use common::repo_with_merged_branch;
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
