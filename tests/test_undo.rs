mod common;

use common::repo_with_commits;
use git_x::commands::commit::UndoCommand;
use git_x::core::traits::Command as CommandTrait;
use predicates::str::contains;
use std::process::Command;

// Helper to check if we should run potentially destructive tests
fn should_run_destructive_tests() -> bool {
    // Only run destructive tests in CI or when explicitly enabled
    std::env::var("CI").is_ok()
        || std::env::var("GITHUB_ACTIONS").is_ok()
        || std::env::var("ENABLE_DESTRUCTIVE_TESTS").is_ok()
}

#[test]
fn test_git_undo_soft_resets_last_commit() {
    if !should_run_destructive_tests() {
        return;
    }

    let repo = repo_with_commits(2);

    repo.run_git_x(&["undo"])
        .success()
        .stdout(contains("Last commit undone"));

    // Verify that the commit was undone but the file changes remain
    let log = Command::new("git")
        .args(["log", "--oneline"])
        .current_dir(repo.path())
        .output()
        .unwrap();
    let log_output = String::from_utf8_lossy(&log.stdout);
    assert!(log_output.contains("initial"));
    assert!(!log_output.contains("commit 2"));

    let diff = Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .current_dir(repo.path())
        .output()
        .unwrap();
    let diff_output = String::from_utf8_lossy(&diff.stdout);
    assert!(diff_output.contains("file.txt"));
}

#[test]
fn test_undo_command_direct() {
    if !should_run_destructive_tests() {
        return;
    }

    let repo = repo_with_commits(3);
    let original_dir = std::env::current_dir().unwrap();

    std::env::set_current_dir(repo.path()).unwrap();

    let cmd = UndoCommand::new();
    let result = cmd.execute();

    // Should succeed and return formatted output
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Last commit undone"));

    // Restore original directory
    let _ = std::env::set_current_dir(&original_dir);
}

#[test]
fn test_undo_command_traits() {
    let cmd = UndoCommand::new();

    // Test Command trait implementation
    assert_eq!(cmd.name(), "undo");
    assert_eq!(
        cmd.description(),
        "Undo the last commit (without losing changes)"
    );
}
