use assert_cmd::Command;
use assert_cmd::assert::OutputAssertExt;
use std::fs;
use std::process::Command as StdCommand;
use tempfile::TempDir;

#[test]
fn test_git_xgraph_runs_without_error() {
    let repo = setup_basic_repo();
    let path = repo.path();

    Command::cargo_bin("git-x")
        .unwrap()
        .arg("graph")
        .current_dir(path)
        .assert()
        .success();
}

#[test]
fn test_git_xgraph_outputs_graph_symbols() {
    let repo = setup_basic_repo();
    let path = repo.path();

    let output = Command::cargo_bin("git-x")
        .unwrap()
        .arg("graph")
        .current_dir(path)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("*") || stdout.contains("|"),
        "Expected ASCII graph symbols in output"
    );
}

fn setup_basic_repo() -> TempDir {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path();

    StdCommand::new("git")
        .arg("init")
        .current_dir(path)
        .assert()
        .success();

    fs::write(path.join("README.md"), "# test").unwrap();
    StdCommand::new("git")
        .args(["add", "."])
        .current_dir(path)
        .assert()
        .success();
    StdCommand::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(path)
        .assert()
        .success();

    temp
}
