mod common;

use assert_cmd::Command;
use common::basic_repo;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_fixup_run_function_outside_git_repo() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["fixup", "abc123"])
        .current_dir(temp_dir.path())
        .assert()
        .success() // The command succeeds but shows an error message
        .stderr(predicate::str::contains("Commit hash does not exist"));
}

#[test]
fn test_fixup_command_direct() {
    use git_x::commands::commit::FixupCommand;
    use git_x::core::traits::Command;

    let repo = basic_repo();
    let original_dir = std::env::current_dir().unwrap();

    std::env::set_current_dir(repo.path()).unwrap();

    let cmd = FixupCommand::new("HEAD".to_string(), false);
    let result = cmd.execute();

    // The fixup command may fail due to no staged changes, which is acceptable
    match &result {
        Ok(_output) => {
            // Command succeeded - this would be rare in this test setup
        }
        Err(e) => {
            // No staged changes error is expected in this test scenario
            assert!(
                e.to_string().contains("No staged changes")
                    || e.to_string().contains("Git command failed")
            );
        }
    }

    // Restore original directory
    let _ = std::env::set_current_dir(&original_dir);
}

#[test]
fn test_fixup_invalid_commit_hash() {
    let repo = basic_repo();

    repo.run_git_x(&["fixup", "nonexistent123"])
        .success() // The command succeeds but shows an error message
        .stderr(predicate::str::contains("Invalid commit hash format"));
}

#[test]
fn test_fixup_no_staged_changes() {
    let repo = basic_repo();
    let commit_hash = repo.create_commit_with_hash("test.txt", "test content", "Test commit");

    repo.run_git_x(&["fixup", &commit_hash[0..7]]) // Use short hash
        .success() // The command succeeds but shows an error message
        .stderr(predicate::str::contains("No staged changes found"));
}

#[test]
fn test_fixup_with_staged_changes() {
    let repo = basic_repo();
    let commit_hash = repo.create_commit_with_hash("test.txt", "test content", "Test commit");

    // Create and stage some changes
    fs::write(repo.path().join("test.txt"), "modified content").expect("Failed to write file");
    repo.stage_files(&["test.txt"]);

    repo.run_git_x(&["fixup", &commit_hash[0..7]]) // Use short hash
        .success()
        .stdout(predicate::str::contains("fixup! Test commit"));
}

#[test]
fn test_fixup_with_rebase_flag() {
    let repo = basic_repo();
    let commit_hash = repo.create_commit_with_hash("test.txt", "test content", "Test commit");

    // Create and stage some changes
    fs::write(repo.path().join("test.txt"), "modified content").expect("Failed to write file");
    repo.stage_files(&["test.txt"]);

    // Set environment to make interactive rebase work in tests
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["fixup", &commit_hash[0..7], "--rebase"])
        .current_dir(repo.path())
        .env("GIT_SEQUENCE_EDITOR", "true") // Auto-accept rebase plan
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "🔄 Starting interactive rebase with autosquash",
        ));
}

#[test]
fn test_fixup_command_help() {
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["fixup", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Create fixup commits for easier interactive rebasing",
        ));
}
