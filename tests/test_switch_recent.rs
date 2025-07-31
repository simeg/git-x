use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::process::Command as StdCommand;
use tempfile::TempDir;

mod common;

#[test]
fn test_switch_recent_in_non_git_repo() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.current_dir(temp_dir.path())
        .arg("switch-recent")
        .assert()
        .success()
        .stderr(predicate::str::contains("Git command failed"));
}

#[test]
fn test_switch_recent_in_empty_git_repo() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Initialize git repo
    StdCommand::new("git")
        .args(["init"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to init git repo");

    // Configure git user for commits
    StdCommand::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to set git user name");

    StdCommand::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to set git user email");

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.current_dir(temp_dir.path())
        .arg("switch-recent")
        .assert()
        .success()
        .stderr(predicate::str::contains("No recent branches found"));
}

#[test]
fn test_switch_recent_with_branches() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Initialize git repo
    StdCommand::new("git")
        .args(["init"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to init git repo");

    // Configure git user for commits
    StdCommand::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to set git user name");

    StdCommand::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to set git user email");

    // Create initial commit
    fs::write(temp_dir.path().join("README.md"), "# Test Repo").expect("Failed to write file");
    StdCommand::new("git")
        .args(["add", "README.md"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to add file");

    StdCommand::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to commit");

    // Create and switch to feature branch
    StdCommand::new("git")
        .args(["checkout", "-b", "feature/test"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to create feature branch");

    // Switch back to main
    StdCommand::new("git")
        .args(["checkout", "master"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to switch back to master");

    // Test switch-recent command (it should find feature/test branch)
    // In non-interactive mode, it should automatically switch to the most recent branch
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.current_dir(temp_dir.path())
        .arg("switch-recent")
        .env("GIT_X_NON_INTERACTIVE", "1") // Explicitly set non-interactive mode
        .assert()
        .success()
        .stdout(predicate::str::contains("Switched to branch"));
}

#[test]
fn test_switch_recent_command_available() {
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("switch-recent"));
}

#[test]
fn test_switch_recent_command_direct() {
    use git_x::commands::branch::SwitchRecentCommand;
    use git_x::core::traits::Command;

    let repo = common::basic_repo();
    let original_dir = std::env::current_dir().unwrap();

    std::env::set_current_dir(repo.path()).unwrap();

    let cmd = SwitchRecentCommand::new();
    let result = cmd.execute();

    // The switch recent command may fail if no recent branches are found
    // This is acceptable since it's testing error handling
    match &result {
        Ok(_output) => {
            // Command succeeded - this is fine
        }
        Err(e) => {
            // Git command failures are acceptable in this test scenario
            assert!(e.to_string().contains("No recent branches found"));
        }
    }

    // Restore original directory
    let _ = std::env::set_current_dir(&original_dir);
}
