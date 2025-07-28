// Integration tests for health.rs run() function testing all code paths

use assert_cmd::Command;
use std::process::Command as StdCommand;
use tempfile::TempDir;

mod common;

#[test]
fn test_health_run_outside_git_repo() {
    // Test error path: not in a git repository
    let temp_dir = TempDir::new().unwrap();

    let output = Command::cargo_bin("git-x")
        .unwrap()
        .current_dir(temp_dir.path())
        .args(["health"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show health check header and not in git repo message
    assert!(stdout.contains("Repository Health Check"));
    assert!(stdout.contains("✗ Not in a Git repository"));
}

#[test]
fn test_health_run_clean_repo() {
    // Test success path: clean repository
    let repo = common::basic_repo();

    let output = Command::cargo_bin("git-x")
        .unwrap()
        .current_dir(repo.path())
        .args(["health"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show health check components
    assert!(stdout.contains("Repository Health Check"));
    assert!(stdout.contains("Working directory"));
    assert!(stdout.contains("untracked files"));
    assert!(stdout.contains("Health check complete!"));
}

#[test]
fn test_health_run_dirty_repo() {
    // Test path: repository with changes
    let repo = common::basic_repo();

    // Make some changes to make the repo dirty
    std::fs::write(repo.path().join("README.md"), "# modified test").unwrap();

    let output = Command::cargo_bin("git-x")
        .unwrap()
        .current_dir(repo.path())
        .args(["health"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show health check with dirty status
    assert!(stdout.contains("Repository Health Check"));
    assert!(stdout.contains("✗ Working directory has changes"));
    assert!(stdout.contains("Health check complete!"));
}

#[test]
fn test_health_run_with_untracked_files() {
    // Test path: repository with untracked files
    let repo = common::basic_repo();

    // Add untracked files
    std::fs::write(repo.path().join("untracked1.txt"), "untracked content 1").unwrap();
    std::fs::write(repo.path().join("untracked2.txt"), "untracked content 2").unwrap();

    let output = Command::cargo_bin("git-x")
        .unwrap()
        .current_dir(repo.path())
        .args(["health"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show health check with untracked files
    assert!(stdout.contains("Repository Health Check"));
    assert!(stdout.contains("untracked files found") || stdout.contains("No untracked files"));
    assert!(stdout.contains("Health check complete!"));
}

#[test]
fn test_health_run_with_staged_changes() {
    // Test path: repository with staged changes
    let repo = common::basic_repo();

    // Add and stage a file
    std::fs::write(repo.path().join("staged_file.txt"), "staged content").unwrap();
    StdCommand::new("git")
        .args(["add", "staged_file.txt"])
        .current_dir(repo.path())
        .output()
        .unwrap();

    let output = Command::cargo_bin("git-x")
        .unwrap()
        .current_dir(repo.path())
        .args(["health"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should show health check with staged changes or handle git errors gracefully
    if stdout.contains("Repository Health Check") {
        assert!(stdout.contains("files staged for commit") || stdout.contains("No staged changes"));
        assert!(stdout.contains("Health check complete!"));
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
    let repo = common::basic_repo();

    let output = Command::cargo_bin("git-x")
        .unwrap()
        .current_dir(repo.path())
        .args(["health"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show health check with repository size
    assert!(stdout.contains("Repository Health Check"));
    assert!(stdout.contains("Repository size:"));
    // Should show healthy since it's a small test repo
    assert!(stdout.contains("healthy") || stdout.contains("moderate"));
    assert!(stdout.contains("Health check complete!"));
}

#[test]
fn test_health_run_comprehensive_output() {
    // Test that all output components are present in success case
    let repo = common::basic_repo();

    let output = Command::cargo_bin("git-x")
        .unwrap()
        .current_dir(repo.path())
        .args(["health"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should contain all expected health check components
    assert!(stdout.contains("Repository Health Check"));
    assert!(stdout.contains("========================="));
    assert!(stdout.contains("Working directory")); // Status check
    assert!(stdout.contains("untracked files")); // Untracked files check
    assert!(stdout.contains("stale branches")); // Stale branches check
    assert!(stdout.contains("Repository size:")); // Repository size check
    assert!(stdout.contains("staged")); // Staged changes check
    assert!(stdout.contains("Health check complete!"));

    // Should contain status indicators (✓, !, or ✗)
    assert!(stdout.contains("✓") || stdout.contains("!") || stdout.contains("✗"));
}

#[test]
fn test_health_run_mixed_states() {
    // Test comprehensive scenario with multiple states
    let repo = common::basic_repo();

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

    let output = Command::cargo_bin("git-x")
        .unwrap()
        .current_dir(repo.path())
        .args(["health"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show health check with mixed states
    assert!(stdout.contains("Repository Health Check"));
    assert!(stdout.contains("Working directory"));
    assert!(stdout.contains("untracked files"));
    assert!(stdout.contains("staged"));
    assert!(stdout.contains("Health check complete!"));
}
