use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

use git_x::commands::repository::NewBranchCommand;
use git_x::core::traits::Command as CommandTrait;

fn create_test_repo() -> (TempDir, PathBuf, String) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let repo_path = temp_dir.path().to_path_buf();

    // Initialize git repo
    Command::new("git")
        .args(["init"])
        .current_dir(&repo_path)
        .assert()
        .success();

    // Configure git
    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(&repo_path)
        .assert()
        .success();

    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(&repo_path)
        .assert()
        .success();

    // Create initial commit
    fs::write(repo_path.join("README.md"), "Initial commit").expect("Failed to write file");
    Command::new("git")
        .args(["add", "README.md"])
        .current_dir(&repo_path)
        .assert()
        .success();

    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(&repo_path)
        .assert()
        .success();

    // Get the actual default branch name
    let branch_output = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to get current branch");
    let default_branch = String::from_utf8_lossy(&branch_output.stdout)
        .trim()
        .to_string();

    (temp_dir, repo_path, default_branch)
}

#[test]
fn test_new_branch_run_function_outside_git_repo() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["new", "test-branch"])
        .current_dir(temp_dir.path())
        .assert()
        .success() // The command succeeds but shows an error message
        .stderr(predicate::str::contains("not a git repository"));
}

#[test]
fn test_new_branch_creates_and_switches() {
    let (_temp_dir, repo_path, _default_branch) = create_test_repo();

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["new", "feature-branch"])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("feature-branch"));

    // Verify we're on the new branch
    let mut check_cmd = Command::new("git");
    check_cmd
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stdout("feature-branch\n");
}

#[test]
fn test_new_branch_with_from_option() {
    let (_temp_dir, repo_path, default_branch) = create_test_repo();

    // Create another branch first
    Command::new("git")
        .args(["checkout", "-b", "develop"])
        .current_dir(&repo_path)
        .assert()
        .success();

    // Add a commit to develop
    fs::write(repo_path.join("develop.txt"), "develop branch").expect("Failed to write file");
    Command::new("git")
        .args(["add", "develop.txt"])
        .current_dir(&repo_path)
        .assert()
        .success();
    Command::new("git")
        .args(["commit", "-m", "Add develop file"])
        .current_dir(&repo_path)
        .assert()
        .success();

    // Go back to default branch (main or master)
    Command::new("git")
        .args(["checkout", &default_branch])
        .current_dir(&repo_path)
        .assert()
        .success();

    // Create new branch from develop
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["new", "feature-from-develop", "--from", "develop"])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("feature-from-develop"));

    // Verify we're on the new branch and it has the develop file
    let mut check_cmd = Command::new("git");
    check_cmd
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stdout("feature-from-develop\n");

    // Check that develop.txt exists (showing it was created from develop)
    assert!(repo_path.join("develop.txt").exists());
}

#[test]
fn test_new_branch_invalid_name_empty() {
    let (_temp_dir, repo_path, _default_branch) = create_test_repo();

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["new", ""])
        .current_dir(&repo_path)
        .assert()
        .success() // The command succeeds but shows validation error
        .stderr(predicate::str::contains("cannot be empty"));
}

#[test]
fn test_new_branch_invalid_name_dash() {
    let (_temp_dir, repo_path, _default_branch) = create_test_repo();

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["new", "--", "-invalid"])
        .current_dir(&repo_path)
        .assert()
        .success() // The command succeeds but shows validation error
        .stderr(predicate::str::contains(
            "Branch name cannot start with a dash",
        ));
}

#[test]
fn test_new_branch_invalid_name_double_dot() {
    let (_temp_dir, repo_path, _default_branch) = create_test_repo();

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["new", "feature..branch"])
        .current_dir(&repo_path)
        .assert()
        .success() // The command succeeds but shows validation error
        .stderr(predicate::str::contains("Branch name cannot contain '..'"));
}

#[test]
fn test_new_branch_invalid_name_spaces() {
    let (_temp_dir, repo_path, _default_branch) = create_test_repo();

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["new", "feature branch"])
        .current_dir(&repo_path)
        .assert()
        .success() // The command succeeds but shows validation error
        .stderr(predicate::str::contains(
            "Branch name cannot contain spaces",
        ));
}

#[test]
fn test_new_branch_existing_branch() {
    let (_temp_dir, repo_path, default_branch) = create_test_repo();

    // Try to create a branch that already exists (using the actual default branch)
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["new", &default_branch])
        .current_dir(&repo_path)
        .assert()
        .success() // The command succeeds but shows error
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn test_new_branch_invalid_base() {
    let (_temp_dir, repo_path, _default_branch) = create_test_repo();

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["new", "feature", "--from", "nonexistent"])
        .current_dir(&repo_path)
        .assert()
        .success() // The command succeeds but shows error
        .stderr(predicate::str::contains("does not exist"));
}

#[test]
fn test_new_branch_command_help() {
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["new", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Create and switch to a new branch",
        ));
}

#[test]
fn test_new_branch_command_direct() {
    let (_temp_dir, repo_path, _default_branch) = create_test_repo();
    let original_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/"));

    std::env::set_current_dir(&repo_path).unwrap();

    // Generate a unique branch name to avoid conflicts in CI
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let unique_branch = format!("feature/test-{timestamp}");

    let cmd = NewBranchCommand::new(unique_branch.clone(), None);
    let result = cmd.execute();

    // Should succeed and return formatted output
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Creating new branch"));
    assert!(output.contains(&unique_branch));

    // Restore original directory
    let _ = std::env::set_current_dir(&original_dir);
}

#[test]
fn test_new_branch_command_with_from() {
    let (_temp_dir, repo_path, default_branch) = create_test_repo();
    let original_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/"));

    // Change to repo directory
    std::env::set_current_dir(&repo_path).unwrap();

    // Generate a unique branch name to avoid conflicts in CI
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let unique_branch = format!("feature/test-{timestamp}");

    // Test with specific base branch
    let cmd = NewBranchCommand::new(unique_branch, Some(default_branch));
    let result = cmd.execute();

    assert!(result.is_ok(), "Command failed: {:?}", result.err());
    let output = result.unwrap();
    assert!(output.contains("Successfully created"));

    // Restore original directory
    let _ = std::env::set_current_dir(&original_dir);
}
