mod common;

use common::{basic_repo, repo_with_branch};
use predicates::str::contains;
use git_x::health::*;

#[test]
fn test_health_command_runs_successfully() {
    let repo = basic_repo();

    repo.run_git_x(&["health"])
        .success()
        .stdout(contains("Repository Health Check"))
        .stdout(contains("Health check complete!"));
}

#[test]
fn test_health_shows_clean_working_directory() {
    let repo = basic_repo();

    repo.run_git_x(&["health"])
        .success()
        .stdout(contains("✓ Working directory is clean"));
}

#[test]
fn test_health_shows_dirty_working_directory() {
    let repo = basic_repo();

    // Create an untracked file
    std::fs::write(repo.path().join("untracked.txt"), "new file").unwrap();

    repo.run_git_x(&["health"])
        .success()
        .stdout(contains("! 1 untracked files found"));
}

#[test]
fn test_health_shows_no_untracked_files_when_clean() {
    let repo = basic_repo();

    repo.run_git_x(&["health"])
        .success()
        .stdout(contains("✓ No untracked files"));
}

#[test]
fn test_health_shows_no_staged_changes() {
    let repo = basic_repo();

    repo.run_git_x(&["health"])
        .success()
        .stdout(contains("✓ No staged changes"));
}

#[test]
fn test_health_shows_staged_changes() {
    let repo = basic_repo();

    // Create and stage a file
    std::fs::write(repo.path().join("staged.txt"), "staged content").unwrap();
    std::process::Command::new("git")
        .args(["add", "staged.txt"])
        .current_dir(repo.path())
        .output()
        .expect("Failed to stage file");

    repo.run_git_x(&["health"])
        .success()
        .stdout(contains("! 1 files staged for commit"));
}

#[test]
fn test_health_shows_repository_size() {
    let repo = basic_repo();

    repo.run_git_x(&["health"])
        .success()
        .stdout(contains("Repository size:"));
}

#[test]
fn test_health_shows_no_stale_branches() {
    let repo = repo_with_branch("feature");

    repo.run_git_x(&["health"])
        .success()
        .stdout(contains("✓ No stale branches"));
}

#[test]
fn test_health_fails_outside_git_repo() {
    let temp_dir = tempfile::tempdir().unwrap();

    assert_cmd::Command::cargo_bin("git-x")
        .unwrap()
        .arg("health")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(contains("✗ Not in a Git repository"));
}

// Unit tests for helper functions
#[test]
fn test_is_git_repo_returns_false_for_non_git_dir() {
    let temp_dir = tempfile::tempdir().unwrap();
    assert!(!is_git_repo(temp_dir.path()));
}
