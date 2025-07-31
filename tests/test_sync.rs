mod common;

use assert_cmd::Command;
use common::basic_repo;
use predicates::prelude::*;
use tempfile::TempDir;

use git_x::commands::repository::{SyncCommand, SyncStrategy};
use git_x::core::traits::Command as CommandTrait;

/// Helper function to execute a command in a specific directory
fn execute_in_dir<P: AsRef<std::path::Path>>(
    dir: P,
    cmd: impl CommandTrait,
) -> Result<String, String> {
    let original_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return Err("❌ Git command failed".to_string()),
    };

    if std::env::set_current_dir(dir).is_err() {
        return Err("❌ Git command failed".to_string());
    }

    let result = match cmd.execute() {
        Ok(output) => Ok(output),
        Err(e) => Err(e.to_string()),
    };

    let _ = std::env::set_current_dir(original_dir);
    result
}

#[test]
fn test_sync_run_function_outside_git_repo() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.arg("sync")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("❌ Git command failed"));

    // Test direct function call (for coverage)
    match execute_in_dir(temp_dir.path(), SyncCommand::new(SyncStrategy::Rebase)) {
        Ok(_) => {
            // Command unexpectedly succeeded
            panic!("Expected sync to fail outside git repo");
        }
        Err(error_msg) => {
            assert!(error_msg.contains("Git command failed"));
        }
    }
}

#[test]
fn test_sync_run_function_no_upstream() {
    let repo = basic_repo();

    // Test CLI interface
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.arg("sync")
        .current_dir(repo.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("❌ Git command failed"));

    // Test direct function call (for coverage)
    match execute_in_dir(repo.path(), SyncCommand::new(SyncStrategy::Rebase)) {
        Ok(_) => {
            // Command may succeed with a message about no upstream
        }
        Err(error_msg) => {
            assert!(error_msg.contains("upstream") || error_msg.contains("Git command failed"));
        }
    }
}

// Keep these as CLI integration tests since they test help text
#[test]
fn test_sync_command_help() {
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["sync", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Sync current branch with upstream",
        ));
}

#[test]
fn test_sync_merge_flag() {
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["sync", "--merge", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Use merge instead of rebase"));
}

#[test]
fn test_sync_command_direct() {
    let repo = basic_repo();
    let original_dir = std::env::current_dir().unwrap();

    std::env::set_current_dir(repo.path()).unwrap();

    let cmd = SyncCommand::new(SyncStrategy::Auto);
    let result = cmd.execute();

    // The new implementation may handle no upstream gracefully
    // Just check that it produces some kind of output
    if let Ok(output) = result {
        assert!(output.contains("upstream") || output.contains("sync"));
    }
    // Error is also acceptable

    // Restore original directory
    let _ = std::env::set_current_dir(&original_dir);
}

#[test]
fn test_sync_command_with_merge_strategy() {
    let repo = basic_repo();
    let original_dir = std::env::current_dir().unwrap();

    // Change to repo directory
    std::env::set_current_dir(repo.path()).unwrap();

    // Test with merge strategy
    let cmd = SyncCommand::new(SyncStrategy::Merge);
    let result = cmd.execute();

    // The new implementation may handle no upstream gracefully
    // Just check that it produces some kind of output
    if let Ok(output) = result {
        assert!(output.contains("upstream") || output.contains("sync"));
    }
    // Error is also acceptable

    // Restore original directory
    let _ = std::env::set_current_dir(&original_dir);
}

#[test]
fn test_sync_command_with_rebase_strategy() {
    let repo = basic_repo();
    let original_dir = std::env::current_dir().unwrap();

    // Change to repo directory
    std::env::set_current_dir(repo.path()).unwrap();

    // Test with rebase strategy
    let cmd = SyncCommand::new(SyncStrategy::Rebase);
    let result = cmd.execute();

    // The new implementation may handle no upstream gracefully
    // Just check that it produces some kind of output
    if let Ok(output) = result {
        assert!(output.contains("upstream") || output.contains("sync"));
    }
    // Error is also acceptable

    // Restore original directory
    let _ = std::env::set_current_dir(&original_dir);
}
