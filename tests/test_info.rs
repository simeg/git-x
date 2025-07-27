mod common;

use common::{repo_with_branch, repo_with_remote_ahead};
use git_x::info::*;
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

// Unit tests for helper functions
#[test]
fn test_extract_repo_name() {
    assert_eq!(extract_repo_name("/path/to/my-repo"), "my-repo");
    assert_eq!(
        extract_repo_name("/another/path/project-name"),
        "project-name"
    );
    assert_eq!(extract_repo_name("simple-name"), "simple-name");
    assert_eq!(extract_repo_name("/"), "");
}

#[test]
fn test_parse_ahead_behind_counts() {
    assert_eq!(
        parse_ahead_behind_counts("3\t2"),
        ("3".to_string(), "2".to_string())
    );
    assert_eq!(
        parse_ahead_behind_counts("0\t5"),
        ("0".to_string(), "5".to_string())
    );
    assert_eq!(
        parse_ahead_behind_counts(""),
        ("0".to_string(), "0".to_string())
    );
    assert_eq!(
        parse_ahead_behind_counts("1"),
        ("1".to_string(), "0".to_string())
    );
}

#[test]
fn test_format_tracking_branch() {
    assert_eq!(format_tracking_branch("origin/main"), "origin/main");
    assert_eq!(
        format_tracking_branch("upstream/develop"),
        "upstream/develop"
    );
}

#[test]
fn test_info_run_function() {
    let (repo, _remote) = repo_with_remote_ahead("main");

    // Change to repo directory and run the function directly
    std::env::set_current_dir(repo.path()).unwrap();

    // Test that the function doesn't panic and git commands work
    // This repo has a remote upstream so it should work
    git_x::info::run();
}
