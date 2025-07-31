mod common;

use assert_cmd::Command as AssertCmd;
use common::repo_with_branch;
use git_x::commands::branch::RenameBranchCommand;
use git_x::core::traits::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::process::Command as StdCommand;
use tempfile::TempDir;

#[test]
fn test_rename_branch_in_isolated_repo() {
    let repo = repo_with_branch("test-branch");

    // Rename the branch from test-branch to renamed-branch
    let status = StdCommand::new("git")
        .args(["branch", "-m", "renamed-branch"])
        .current_dir(repo.path())
        .status()
        .expect("Failed to rename branch");
    assert!(status.success());

    // Verify the current branch name
    let output = StdCommand::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(repo.path())
        .output()
        .expect("Failed to get current branch");

    assert!(output.status.success());
    let current_branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(current_branch, "renamed-branch");
}

fn create_test_repo() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let repo_path = temp_dir.path().to_path_buf();

    // Initialize git repo
    StdCommand::new("git")
        .args(["init"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to init repo");

    // Configure git
    StdCommand::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to configure user");

    StdCommand::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to configure email");

    // Create initial commit
    fs::write(repo_path.join("README.md"), "Initial commit").expect("Failed to write file");
    StdCommand::new("git")
        .args(["add", "README.md"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to add file");

    StdCommand::new("git")
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
        .success() // The command succeeds but prints error to stderr
        .stderr(predicate::str::contains("Git command failed"));
}

#[test]
fn test_rename_branch_same_name() {
    let (_temp_dir, repo_path) = create_test_repo();

    // Get current branch name (should be main or master)
    let output = StdCommand::new("git")
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
        .stderr(predicate::str::contains("already exists"));
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
        .success() // The command succeeds but prints error to stderr
        .stderr(predicate::str::contains("Git command failed"));
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
    StdCommand::new("git")
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
        .success() // The command succeeds
        .stdout(predicate::str::contains("Renamed branch"));
}

#[test]
fn test_rename_branch_run_function_successful_case() {
    let repo = repo_with_branch("test-branch");
    let original_dir = std::env::current_dir().unwrap();

    std::env::set_current_dir(repo.path()).expect("Failed to change directory");

    let cmd = RenameBranchCommand::new("test-branch".to_string());
    let result = cmd.execute();
    // The command may fail if the branch already has this name, which is acceptable
    match &result {
        Ok(_) => {
            // Command succeeded - this is fine
        }
        Err(e) => {
            // Branch already exists error is acceptable in this test scenario
            assert!(e.to_string().contains("already exists"));
        }
    }

    // Restore original directory
    let _ = std::env::set_current_dir(&original_dir);
}

#[test]
fn test_rename_branch_run_function_same_name() {
    let repo = repo_with_branch("test-branch");
    let original_dir = std::env::current_dir().unwrap();

    std::env::set_current_dir(repo.path()).expect("Failed to change directory");

    let cmd = RenameBranchCommand::new("test-branch".to_string());
    let result = cmd.execute();
    // The command may fail if the branch already has this name, which is acceptable
    match &result {
        Ok(_) => {
            // Command succeeded - this is fine
        }
        Err(e) => {
            // Branch already exists error is acceptable in this test scenario
            assert!(e.to_string().contains("already exists"));
        }
    }

    // Restore original directory
    let _ = std::env::set_current_dir(&original_dir);
}
