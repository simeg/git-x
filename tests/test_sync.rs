use assert_cmd::Command;
use git_x::sync::*;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// Helper function to create a test git repository
fn create_test_repo() -> (TempDir, PathBuf) {
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

    (temp_dir, repo_path)
}

#[test]
fn test_parse_sync_counts() {
    assert_eq!(parse_sync_counts("0\t0"), Ok((0, 0)));
    assert_eq!(parse_sync_counts("1\t0"), Ok((1, 0)));
    assert_eq!(parse_sync_counts("0\t2"), Ok((0, 2)));
    assert_eq!(parse_sync_counts("3\t5"), Ok((3, 5)));
    assert_eq!(parse_sync_counts("10\t20"), Ok((10, 20)));
}

#[test]
fn test_parse_sync_counts_invalid() {
    assert!(parse_sync_counts("").is_err());
    assert!(parse_sync_counts("abc").is_err());
    assert!(parse_sync_counts("1").is_err());
    assert!(parse_sync_counts("1\tabc").is_err());
}

#[test]
fn test_format_sync_start_message() {
    assert_eq!(
        format_sync_start_message("main", "origin/main"),
        "🔄 Syncing branch 'main' with 'origin/main'..."
    );
    assert_eq!(
        format_sync_start_message("feature", "upstream/develop"),
        "🔄 Syncing branch 'feature' with 'upstream/develop'..."
    );
}

#[test]
fn test_format_error_message() {
    assert_eq!(format_error_message("Test error"), "❌ Test error");
    assert_eq!(
        format_error_message("Connection failed"),
        "❌ Connection failed"
    );
}

#[test]
fn test_format_up_to_date_message() {
    assert_eq!(
        format_up_to_date_message(),
        "✅ Branch is up to date with upstream"
    );
}

#[test]
fn test_format_behind_message() {
    assert_eq!(
        format_behind_message(1),
        "⬇️ Branch is 1 commit(s) behind upstream"
    );
    assert_eq!(
        format_behind_message(5),
        "⬇️ Branch is 5 commit(s) behind upstream"
    );
}

#[test]
fn test_format_ahead_message() {
    assert_eq!(
        format_ahead_message(1),
        "⬆️ Branch is 1 commit(s) ahead of upstream"
    );
    assert_eq!(
        format_ahead_message(3),
        "⬆️ Branch is 3 commit(s) ahead of upstream"
    );
}

#[test]
fn test_format_diverged_message() {
    assert_eq!(
        format_diverged_message(2, 3),
        "🔀 Branch has diverged: 2 behind, 3 ahead"
    );
    assert_eq!(
        format_diverged_message(1, 1),
        "🔀 Branch has diverged: 1 behind, 1 ahead"
    );
}

#[test]
fn test_format_diverged_help_message() {
    assert_eq!(
        format_diverged_help_message(),
        "💡 Use --merge flag to merge changes, or handle manually"
    );
}

#[test]
fn test_format_sync_success_message() {
    assert_eq!(
        format_sync_success_message(true),
        "✅ Successfully merged upstream changes"
    );
    assert_eq!(
        format_sync_success_message(false),
        "✅ Successfully rebased onto upstream"
    );
}

#[test]
fn test_sync_run_function_outside_git_repo() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.arg("sync")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("Not in a git repository"));
}

#[test]
fn test_sync_run_function_no_upstream() {
    let (_temp_dir, repo_path) = create_test_repo();

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.arg("sync")
        .current_dir(&repo_path)
        .assert()
        .success()
        .stderr(predicate::str::contains("No upstream branch configured"));
}

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
