mod common;

use common::{repo_with_branch, repo_with_remote_ahead};
use git_x::info::*;
use predicates::str::contains;
use tempfile::TempDir;

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

    // Test that the function returns Ok and contains expected content
    let result = git_x::info::run();
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.contains("Repo:"));
    assert!(output.contains("Branch:"));
    assert!(output.contains("Tracking:"));
    assert!(output.contains("Ahead:"));
    assert!(output.contains("Behind:"));
    assert!(output.contains("Last Commit:"));
}

#[test]
fn test_info_run_function_error_case() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Create a completely isolated directory that definitely isn't a git repo
    let isolated_dir = temp_dir.path().join("isolated");
    std::fs::create_dir(&isolated_dir).expect("Failed to create isolated directory");

    // Unset GIT_DIR and GIT_WORK_TREE to ensure git doesn't find parent repos
    let original_git_dir = std::env::var("GIT_DIR").ok();
    let original_git_work_tree = std::env::var("GIT_WORK_TREE").ok();
    unsafe {
        std::env::remove_var("GIT_DIR");
        std::env::remove_var("GIT_WORK_TREE");
    }

    let original_dir = std::env::current_dir().expect("Failed to get current directory");
    std::env::set_current_dir(&isolated_dir).expect("Failed to change directory");

    // Test that the function returns an error when not in a git repo
    let result = git_x::info::run();

    // Restore original directory and environment BEFORE temp_dir is dropped
    std::env::set_current_dir(original_dir).expect("Failed to reset directory");
    unsafe {
        if let Some(git_dir) = original_git_dir {
            std::env::set_var("GIT_DIR", git_dir);
        }
        if let Some(git_work_tree) = original_git_work_tree {
            std::env::set_var("GIT_WORK_TREE", git_work_tree);
        }
    }

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(
        format!("{error}"),
        "Git command failed: Not in a git repository"
    );
}
