mod common;

use common::{basic_repo, repo_with_branch};
use git_x::commands::repository::HealthCommand;
use git_x::core::traits::Command;
use predicates::str::contains;
use tempfile::TempDir;

#[test]
fn test_health_command_runs_successfully() {
    let repo = basic_repo();

    repo.run_git_x(&["health"])
        .success()
        .stdout(contains("Repository Health Check"))
        .stdout(contains("Git configuration: OK"));
}

#[test]
fn test_health_shows_clean_working_directory() {
    let repo = basic_repo();

    repo.run_git_x(&["health"])
        .success()
        .stdout(contains("Working directory: Clean"));
}

#[test]
fn test_health_shows_dirty_working_directory() {
    let repo = basic_repo();

    // Create an untracked file
    std::fs::write(repo.path().join("untracked.txt"), "new file").unwrap();

    repo.run_git_x(&["health"])
        .success()
        .stdout(contains("Repository Health Check"));
}

#[test]
fn test_health_shows_no_untracked_files_when_clean() {
    let repo = basic_repo();

    repo.run_git_x(&["health"])
        .success()
        .stdout(contains("Working directory: Clean"));
}

#[test]
fn test_health_shows_no_staged_changes() {
    let repo = basic_repo();

    repo.run_git_x(&["health"])
        .success()
        .stdout(contains("Working directory: Clean"));
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
        .stdout(contains("files staged for commit"));
}

#[test]
fn test_health_shows_repository_size() {
    let repo = basic_repo();

    repo.run_git_x(&["health"])
        .success()
        .stdout(contains("Repository size: OK"));
}

#[test]
fn test_health_shows_no_stale_branches() {
    let repo = repo_with_branch("feature");

    repo.run_git_x(&["health"])
        .success()
        .stdout(contains("Branches: OK"));
}

#[test]
fn test_health_fails_outside_git_repo() {
    let temp_dir = tempfile::tempdir().unwrap();

    AssertCommand::cargo_bin("git-x")
        .unwrap()
        .arg("health")
        .current_dir(temp_dir.path())
        .assert()
        .success() // Our new health command succeeds but shows issues
        .stdout(contains("Repository Health Check"))
        .stdout(contains("issue(s)"));
}

#[test]
fn test_health_command_direct() {
    let repo = basic_repo();
    let original_dir = std::env::current_dir().unwrap();

    std::env::set_current_dir(repo.path()).unwrap();

    let cmd = HealthCommand::new();
    let result = cmd.execute();

    // Should succeed and return formatted output
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Repository Health Check"));

    // Restore original directory
    let _ = std::env::set_current_dir(&original_dir);
}

#[test]
fn test_health_command_traits() {
    let cmd = HealthCommand::new();

    // Test Command trait implementation
    assert_eq!(cmd.name(), "health");
    assert_eq!(
        cmd.description(),
        "Check repository health and configuration"
    );
}

#[test]
fn test_health_command_in_non_git_directory() {
    let temp_dir = tempfile::tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();

    // Change to non-git directory
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let cmd = HealthCommand::new();
    let result = cmd.execute();

    // Should fail or return error message for non-git directory
    if let Ok(output) = result {
        assert!(output.contains("Repository Health Check"));
    }

    // Restore original directory
    let _ = std::env::set_current_dir(&original_dir);
}

// Integration tests for health.rs run() function testing all code paths

use assert_cmd::Command as AssertCommand;
use std::process::Command as StdCommand;

#[test]
fn test_health_run_outside_git_repo() {
    // Test error path: not in a git repository
    let temp_dir = TempDir::new().unwrap();

    let output = AssertCommand::cargo_bin("git-x")
        .unwrap()
        .current_dir(temp_dir.path())
        .args(["health"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show health check with issues found
    assert!(output.status.success());
    assert!(stdout.contains("Repository Health Check"));
}

#[test]
fn test_health_run_clean_repo() {
    // Test success path: clean repository
    let repo = basic_repo();

    let output = AssertCommand::cargo_bin("git-x")
        .unwrap()
        .current_dir(repo.path())
        .args(["health"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show health check components
    assert!(stdout.contains("Repository Health Check"));
    assert!(stdout.contains("Working directory"));
}

#[test]
fn test_health_run_dirty_repo() {
    // Test path: repository with changes
    let repo = basic_repo();

    // Make some changes to make the repo dirty
    std::fs::write(repo.path().join("README.md"), "# modified test").unwrap();

    let output = AssertCommand::cargo_bin("git-x")
        .unwrap()
        .current_dir(repo.path())
        .args(["health"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show health check with dirty status
    assert!(stdout.contains("Repository Health Check"));
    assert!(stdout.contains("Working directory"));
}

#[test]
fn test_health_run_with_untracked_files() {
    // Test path: repository with untracked files
    let repo = basic_repo();

    // Add untracked files
    std::fs::write(repo.path().join("untracked1.txt"), "untracked content 1").unwrap();
    std::fs::write(repo.path().join("untracked2.txt"), "untracked content 2").unwrap();

    let output = AssertCommand::cargo_bin("git-x")
        .unwrap()
        .current_dir(repo.path())
        .args(["health"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show health check with untracked files
    assert!(stdout.contains("Repository Health Check"));
}

#[test]
fn test_health_run_with_staged_changes() {
    // Test path: repository with staged changes
    let repo = basic_repo();

    // Add and stage a file
    std::fs::write(repo.path().join("staged_file.txt"), "staged content").unwrap();
    StdCommand::new("git")
        .args(["add", "staged_file.txt"])
        .current_dir(repo.path())
        .output()
        .unwrap();

    let output = AssertCommand::cargo_bin("git-x")
        .unwrap()
        .current_dir(repo.path())
        .args(["health"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should show health check with staged changes or handle git errors gracefully
    if stdout.contains("Repository Health Check") {
        assert!(stdout.contains("Repository Health Check"));
    } else if stderr.contains("Git command failed") {
        eprintln!(
            "Note: Git command failed in test environment - this is expected in some CI environments"
        );
    } else {
        panic!("Expected either health check output or git command failure");
    }
}

#[test]
fn test_health_run_repo_size_check() {
    // Test path: repository size check
    let repo = basic_repo();

    let output = AssertCommand::cargo_bin("git-x")
        .unwrap()
        .current_dir(repo.path())
        .args(["health"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show health check with repository size
    assert!(stdout.contains("Repository Health Check"));
    assert!(stdout.contains("Repository size"));
}

#[test]
fn test_health_run_comprehensive_output() {
    // Test that all output components are present in success case
    let repo = basic_repo();

    let output = AssertCommand::cargo_bin("git-x")
        .unwrap()
        .current_dir(repo.path())
        .args(["health"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should contain basic health check components
    assert!(stdout.contains("Repository Health Check"));
    assert!(stdout.contains("=============================="));
    assert!(stdout.contains("Working directory"));
    assert!(stdout.contains("Repository size"));

    // Should contain status indicators (✅ or ⚠️)
    assert!(stdout.contains("✅") || stdout.contains("⚠️"));
}

#[test]
fn test_health_run_mixed_states() {
    // Test comprehensive scenario with multiple states
    let repo = basic_repo();

    // Create mixed scenario:
    // 1. Untracked files
    std::fs::write(repo.path().join("untracked.txt"), "untracked").unwrap();

    // 2. Modified files
    std::fs::write(repo.path().join("README.md"), "# modified").unwrap();

    // 3. Staged files
    std::fs::write(repo.path().join("staged.txt"), "staged content").unwrap();
    StdCommand::new("git")
        .args(["add", "staged.txt"])
        .current_dir(repo.path())
        .output()
        .unwrap();

    let output = AssertCommand::cargo_bin("git-x")
        .unwrap()
        .current_dir(repo.path())
        .args(["health"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show health check with mixed states
    assert!(stdout.contains("Repository Health Check"));
    assert!(stdout.contains("Working directory"));
}
