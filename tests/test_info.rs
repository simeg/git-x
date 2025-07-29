mod common;

use common::{repo_with_branch, repo_with_remote_ahead};
use git_x::core::output::Format;
use predicates::str::contains;

#[test]
fn test_info_output_contains_expected_lines() {
    let repo = repo_with_branch("test-branch");

    repo.run_git_x(&["info"])
        .success()
        .stdout(contains("Repo:"))
        .stdout(contains("Branch: test-branch"))
        .stdout(contains("Last Commit: \"initial commit"));
}

#[test]
fn test_info_output_includes_ahead_behind() {
    let repo = repo_with_branch("test-branch");
    let _remote = repo.setup_remote("test-branch");

    // Add a new commit to be ahead
    repo.add_commit("file.txt", "arbitrary", "local commit");

    repo.run_git_x(&["info"])
        .success()
        .stdout(contains("Ahead: 1"))
        .stdout(contains("Behind: 0"));
}

#[test]
fn test_info_output_shows_behind() {
    let (repo, _remote) = repo_with_remote_ahead("test-branch");

    repo.run_git_x(&["info"])
        .success()
        .stdout(contains("Ahead: 0"))
        .stdout(contains("Behind: 1"));
}

// Unit tests for common utilities
#[test]
fn test_format_functions() {
    let error_msg = Format::error("Test error");
    assert!(error_msg.contains("❌"));
    assert!(error_msg.contains("Test error"));

    let success_msg = Format::success("Test success");
    assert!(success_msg.contains("✅"));
    assert!(success_msg.contains("Test success"));

    let info_msg = Format::info("Test info");
    assert!(info_msg.contains("ℹ️"));
    assert!(info_msg.contains("Test info"));
}

#[test]
fn test_info_run_function() {
    let (repo, _remote) = repo_with_remote_ahead("main");

    // Change to repo directory and run the function directly
    std::env::set_current_dir(repo.path()).unwrap();

    // Test that the function doesn't panic and git commands work
    // This repo has a remote upstream so it should work
    let _ = git_x::info::run();
}
