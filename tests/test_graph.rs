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
