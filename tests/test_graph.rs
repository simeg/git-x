mod common;

use common::basic_repo;
use git_x::graph::*;

#[test]
fn test_git_xgraph_runs_without_error() {
    let repo = basic_repo();
    repo.run_git_x(&["graph"]).success();
}

#[test]
fn test_git_xgraph_outputs_graph_symbols() {
    use assert_cmd::Command;

    let repo = basic_repo();
    let output = Command::cargo_bin("git-x")
        .unwrap()
        .arg("graph")
        .current_dir(repo.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("*") || stdout.contains("|"),
        "Expected ASCII graph symbols in output"
    );
}

// Unit tests for helper functions
#[test]
fn test_get_git_log_args() {
    assert_eq!(
        get_git_log_args(),
        ["log", "--oneline", "--graph", "--decorate", "--all"]
    );
}

#[test]
fn test_format_git_error() {
    assert_eq!(
        format_git_error("not a git repository"),
        "❌ git log failed:\nnot a git repository"
    );
    assert_eq!(
        format_git_error("permission denied"),
        "❌ git log failed:\npermission denied"
    );
}

#[test]
fn test_graph_run_function() {
    let repo = common::basic_repo();

    // Change to repo directory and run the function directly
    std::env::set_current_dir(repo.path()).unwrap();

    // Test that the function returns Ok and contains git graph output
    let result = git_x::graph::run();
    assert!(result.is_ok());

    let output = result.unwrap();
    // Should contain git log output with graph symbols
    assert!(
        output.contains("*") || output.contains("|") || output.contains("initial commit"),
        "Expected git graph output"
    );
}

#[test]
fn test_graph_run_function_in_non_git_directory() {
    let temp_dir = tempfile::tempdir().unwrap();

    // Change to non-git directory to trigger error path
    std::env::set_current_dir(temp_dir.path()).unwrap();

    // Test that the function returns an error when not in a git repository
    let result = git_x::graph::run();
    assert!(result.is_err());

    let error = result.unwrap_err();
    let error_msg = format!("{error}");
    assert!(error_msg.contains("git log failed"));
}
