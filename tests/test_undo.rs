use assert_cmd::Command;
use assert_cmd::assert::OutputAssertExt;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_git_xundo_soft_resets_last_commit() {
    let repo = setup_repo_with_commit();
    let path = repo.path();

    Command::cargo_bin("git-x")
        .unwrap()
        .arg("undo")
        .current_dir(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Last commit undone"));

    // Verify that the commit was undone but the file changes remain
    let log = std::process::Command::new("git")
        .args(["log", "--oneline"])
        .current_dir(path)
        .output()
        .unwrap();
    let log_output = String::from_utf8_lossy(&log.stdout);
    assert!(log_output.contains("initial"));
    assert!(!log_output.contains("second"));

    let diff = std::process::Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .current_dir(path)
        .output()
        .unwrap();
    let diff_output = String::from_utf8_lossy(&diff.stdout);
    assert!(diff_output.contains("file.txt"));
}

fn setup_repo_with_commit() -> TempDir {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path();

    std::process::Command::new("git")
        .arg("init")
        .current_dir(path)
        .assert()
        .success();

    fs::write(path.join("file.txt"), "initial content").unwrap();
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(path)
        .assert()
        .success();
    std::process::Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(path)
        .assert()
        .success();

    fs::write(path.join("file.txt"), "changed content").unwrap(); // <-- Fix
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(path)
        .assert()
        .success();
    std::process::Command::new("git")
        .args(["commit", "-m", "second"])
        .current_dir(path)
        .assert()
        .success();

    temp
}
