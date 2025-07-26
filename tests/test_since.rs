use assert_cmd::Command;
use assert_cmd::assert::OutputAssertExt;
use std::fs;
use std::process::Command as StdCommand;
use tempfile::TempDir;

#[test]
fn test_git_xsince_outputs_commits_since_ref() {
    let repo = setup_repo_with_commits();
    let path = repo.path();

    let output = Command::cargo_bin("git-x")
        .unwrap()
        .args(["since", "HEAD~1"])
        .current_dir(path)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("🔍 Commits since HEAD~1:"));
    assert!(stdout.contains("second commit"));
}

#[test]
fn test_git_xsince_no_new_commits() {
    let repo = setup_repo_with_commits();
    let path = repo.path();

    let output = Command::cargo_bin("git-x")
        .unwrap()
        .args(["since", "HEAD"])
        .current_dir(path)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("✅ No new commits since HEAD"));
}
fn setup_repo_with_commits() -> TempDir {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path();

    StdCommand::new("git")
        .arg("init")
        .current_dir(path)
        .assert()
        .success();

    fs::write(path.join("file.txt"), "arbitrary").unwrap();
    StdCommand::new("git")
        .args(["add", "."])
        .current_dir(path)
        .assert()
        .success();
    StdCommand::new("git")
        .args(["commit", "-m", "initial commit"])
        .current_dir(path)
        .assert()
        .success();

    // Create a second commit
    fs::write(path.join("file.txt"), "updated content").unwrap();
    StdCommand::new("git")
        .args(["add", "."])
        .current_dir(path)
        .assert()
        .success();
    StdCommand::new("git")
        .args(["commit", "-m", "second commit"])
        .current_dir(path)
        .assert()
        .success();

    temp
}
