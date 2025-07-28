mod common;

use common::basic_repo;

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

// Unit tests now handled by common module tests

#[test]
fn test_graph_run_function() {
    let repo = common::basic_repo();
    let original_dir = std::env::current_dir().unwrap();

    // Change to repo directory and run the function directly
    std::env::set_current_dir(repo.path()).unwrap();

    // Test that the function doesn't panic and git commands work
    git_x::graph::run();

    // Restore original directory
    let _ = std::env::set_current_dir(&original_dir);
}

#[test]
fn test_graph_run_function_in_non_git_directory() {
    let temp_dir = tempfile::tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();

    // Change to non-git directory to trigger error path
    std::env::set_current_dir(temp_dir.path()).unwrap();

    // Test that the function handles git command failure gracefully
    git_x::graph::run();

    // Restore original directory
    let _ = std::env::set_current_dir(&original_dir);
}
