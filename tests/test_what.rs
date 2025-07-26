use assert_cmd::Command;
use assert_cmd::assert::OutputAssertExt;
use predicates::prelude::*;
use std::fs;
use std::process::Command as StdCommand;
use tempfile::tempdir;

fn init_test_repo_with_changes() -> std::path::PathBuf {
    let dir = tempdir().expect("failed to create temp dir");
    let repo_path = dir.keep();

    StdCommand::new("git")
        .args(["init", "--initial-branch=main"])
        .current_dir(&repo_path)
        .assert()
        .success();

    // git init
    StdCommand::new("git")
        .arg("init")
        .current_dir(&repo_path)
        .assert()
        .success();

    // make main branch and initial commit
    fs::write(repo_path.join("file.txt"), "arbitrary").unwrap();
    StdCommand::new("git")
        .args(["add", "."])
        .current_dir(&repo_path)
        .assert()
        .success();
    StdCommand::new("git")
        .args(["commit", "-m", "initial commit"])
        .current_dir(&repo_path)
        .assert()
        .success();

    // create and switch to feature branch
    StdCommand::new("git")
        .args(["checkout", "-b", "feature/test"])
        .current_dir(&repo_path)
        .assert()
        .success();

    // make a change
    fs::write(repo_path.join("file.txt"), "arbitrary-2").unwrap();
    StdCommand::new("git")
        .args(["add", "."])
        .current_dir(&repo_path)
        .assert()
        .success();
    StdCommand::new("git")
        .args(["commit", "-m", "modified file"])
        .current_dir(&repo_path)
        .assert()
        .success();

    repo_path
}

#[test]
fn test_git_xwhat_shows_diff_and_commits() {
    let repo = init_test_repo_with_changes();

    let mut cmd = Command::cargo_bin("git-x").unwrap();
    cmd.current_dir(&repo)
        .arg("what")
        .assert()
        .success()
        .stdout(predicate::str::contains("Branch: feature/test vs main"))
        .stdout(predicate::str::contains("+ 1 commits ahead"))
        .stdout(predicate::str::contains("Changes:"))
        .stdout(predicate::str::contains("~ file.txt"));
}
