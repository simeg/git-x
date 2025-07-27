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

#[test]
fn test_get_current_branch_success() {
    let (_temp_dir, repo_path) = create_test_repo();

    // Change to the repo directory and call get_current_branch
    std::env::set_current_dir(&repo_path).expect("Failed to change directory");

    let result = get_current_branch();
    std::env::set_current_dir("/").expect("Failed to reset directory");

    assert!(result.is_ok());
    let branch = result.unwrap();
    assert!(!branch.is_empty());
}

#[test]
fn test_get_current_branch_not_git_repo() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    std::env::set_current_dir(temp_dir.path()).expect("Failed to change directory");

    let result = get_current_branch();
    std::env::set_current_dir("/").expect("Failed to reset directory");

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Not in a git repository");
}

#[test]
fn test_get_upstream_branch_no_upstream() {
    let (_temp_dir, repo_path) = create_test_repo();

    std::env::set_current_dir(&repo_path).expect("Failed to change directory");

    let result = get_upstream_branch("main");
    std::env::set_current_dir("/").expect("Failed to reset directory");

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "No upstream branch configured");
}

#[test]
fn test_sync_status_enum_equality() {
    assert_eq!(SyncStatus::UpToDate, SyncStatus::UpToDate);
    assert_eq!(SyncStatus::Behind(5), SyncStatus::Behind(5));
    assert_eq!(SyncStatus::Ahead(3), SyncStatus::Ahead(3));
    assert_eq!(SyncStatus::Diverged(2, 4), SyncStatus::Diverged(2, 4));

    assert_ne!(SyncStatus::Behind(1), SyncStatus::Behind(2));
    assert_ne!(SyncStatus::Ahead(1), SyncStatus::Ahead(2));
    assert_ne!(SyncStatus::UpToDate, SyncStatus::Behind(1));
}

#[test]
fn test_sync_status_debug() {
    let status = SyncStatus::Diverged(3, 5);
    let debug_str = format!("{status:?}");
    assert!(debug_str.contains("Diverged"));
    assert!(debug_str.contains("3"));
    assert!(debug_str.contains("5"));
}

#[test]
fn test_fetch_upstream_success() {
    let (_temp_dir, repo_path) = create_test_repo();

    // Add a remote
    Command::new("git")
        .args([
            "remote",
            "add",
            "origin",
            "https://github.com/test/repo.git",
        ])
        .current_dir(&repo_path)
        .assert()
        .success();

    std::env::set_current_dir(&repo_path).expect("Failed to change directory");

    // Test fetch with invalid remote (will fail but tests the error path)
    let result = fetch_upstream("origin/main");
    std::env::set_current_dir("/").expect("Failed to reset directory");

    // Should fail because remote doesn't exist, but tests the function
    assert!(result.is_err());
}

#[test]
fn test_get_sync_status_patterns() {
    let (_temp_dir, repo_path) = create_test_repo();

    std::env::set_current_dir(&repo_path).expect("Failed to change directory");

    // This will fail since there's no upstream, but tests the error path
    let result = get_sync_status("main", "origin/main");
    std::env::set_current_dir("/").expect("Failed to reset directory");

    assert!(result.is_err());
}

#[test]
fn test_sync_with_upstream_merge() {
    let (_temp_dir, repo_path) = create_test_repo();

    std::env::set_current_dir(&repo_path).expect("Failed to change directory");

    // This will fail since there's no upstream, but tests the error path
    let result = sync_with_upstream("origin/main", true);
    std::env::set_current_dir("/").expect("Failed to reset directory");

    assert!(result.is_err());
}

#[test]
fn test_sync_with_upstream_rebase() {
    let (_temp_dir, repo_path) = create_test_repo();

    std::env::set_current_dir(&repo_path).expect("Failed to change directory");

    // This will fail since there's no upstream, but tests the error path
    let result = sync_with_upstream("origin/main", false);
    std::env::set_current_dir("/").expect("Failed to reset directory");

    assert!(result.is_err());
}

#[test]
fn test_run_function_complete_flow() {
    // Simple test that verifies the main run function executes without crashing
    // when called from outside a git repository (error path)
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.arg("sync")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("Not in a git repository"));
}
