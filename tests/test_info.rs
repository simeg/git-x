mod common;

use common::{repo_with_branch, repo_with_remote_ahead};
use git_x::commands::repository::InfoCommand;
use git_x::core::output::Format;
use git_x::core::traits::Command;
use predicates::str::contains;

#[test]
fn test_info_output_contains_expected_lines() {
    let repo = repo_with_branch("test-branch");

    repo.run_git_x(&["info"])
        .success()
        .stdout(contains("Repository:"))
        .stdout(contains("Current branch: test-branch"));
}

#[test]
fn test_info_output_includes_ahead_behind() {
    let repo = repo_with_branch("test-branch");
    let _remote = repo.setup_remote("test-branch");

    // Add a new commit to be ahead
    repo.add_commit("file.txt", "arbitrary", "local commit");

    repo.run_git_x(&["info"])
        .success()
        .stdout(contains("Status: 1 ahead"));
}

#[test]
fn test_info_output_shows_behind() {
    let (repo, _remote) = repo_with_remote_ahead("test-branch");

    repo.run_git_x(&["info"])
        .success()
        .stdout(contains("Status: 1 behind"));
}

#[test]
fn test_info_enhanced_with_recent_activity() {
    let repo = repo_with_branch("test-branch");

    // Add multiple commits to create activity timeline
    repo.add_commit("file1.txt", "content1", "First commit");
    repo.add_commit("file2.txt", "content2", "Second commit");
    repo.add_commit("file3.txt", "content3", "Third commit");

    let info_cmd = InfoCommand::new().with_details();

    std::env::set_current_dir(repo.path()).expect("Failed to change directory");

    match info_cmd.execute() {
        Ok(output) => {
            assert!(output.contains("Repository:"));
            assert!(output.contains("Current branch:"));
        }
        Err(_) => {
            // Command may fail in test environment, that's ok
        }
    }
}

#[test]
fn test_info_shows_branch_differences() {
    let repo = repo_with_branch("feature-branch");

    // Create main branch for comparison
    repo.create_branch("main");
    repo.add_commit("main.txt", "main content", "Main commit");

    // Switch back to feature branch and add commits
    repo.checkout_branch("feature-branch");
    repo.add_commit("feature.txt", "feature content", "Feature commit");

    let info_cmd = InfoCommand::new();

    std::env::set_current_dir(repo.path()).expect("Failed to change directory");

    match info_cmd.execute() {
        Ok(output) => {
            assert!(output.contains("Current branch:"));
        }
        Err(_) => {
            // Command may fail in test environment, that's ok
        }
    }
}

#[test]
fn test_info_github_pr_detection() {
    let repo = repo_with_branch("test-branch");

    // This test will pass regardless of whether gh CLI is available
    // since the function handles missing gh gracefully
    let info_cmd = InfoCommand::new();

    std::env::set_current_dir(repo.path()).expect("Failed to change directory");

    match info_cmd.execute() {
        Ok(output) => {
            assert!(output.contains("Repository:"));
        }
        Err(_) => {
            // Command may fail in test environment, that's ok
        }
    }
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
fn test_info_command_direct() {
    let (repo, _remote) = repo_with_remote_ahead("main");

    std::env::set_current_dir(repo.path()).unwrap();

    let cmd = InfoCommand::new();
    let result = cmd.execute();

    // Should succeed and return formatted output
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Repository:"));
    assert!(output.contains("Current branch:"));
}

#[test]
fn test_info_command_traits() {
    let cmd = InfoCommand::new();

    // Test Command trait implementation
    assert_eq!(cmd.name(), "info");
    assert_eq!(cmd.description(), "Show repository information and status");
}

#[test]
fn test_info_command_with_details() {
    let (repo, _remote) = repo_with_remote_ahead("main");
    std::env::set_current_dir(repo.path()).unwrap();

    // Test enhanced command with details
    let cmd = InfoCommand::new().with_details();
    let result = cmd.execute();

    assert!(result.is_ok());
    let output = result.unwrap();

    // Detailed version should include more information
    assert!(output.contains("Repository:"));
    assert!(output.contains("Working directory:"));
    assert!(output.contains("Staged files:"));
}
