use assert_cmd::Command;
use predicates::str::contains;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_info_output_contains_expected_lines() {
    let repo_path = init_test_repo("README.md", "# test repo", "initial commit");

    Command::cargo_bin("git-x")
        .unwrap()
        .arg("info")
        .current_dir(&repo_path)
        .assert()
        .success()
        .stdout(contains("📂 Repo:"))
        .stdout(contains("🔀 Branch: test-branch"))
        .stdout(contains("📌 Last Commit: \"initial commit"));
}

#[test]
fn test_info_output_includes_ahead_behind() {
    let repo_path = init_test_repo("init.txt", "init", "initial");

    // Create a dummy remote and push current branch to it
    let remote_path = tempfile::tempdir().unwrap().keep();
    Command::new("git")
        .arg("init")
        .arg("--bare")
        .arg(remote_path.to_str().unwrap())
        .assert()
        .success();

    Command::new("git")
        .args(["remote", "add", "origin", remote_path.to_str().unwrap()])
        .current_dir(&repo_path)
        .assert()
        .success();

    Command::new("git")
        .args(["push", "-u", "origin", "test-branch"])
        .current_dir(&repo_path)
        .assert()
        .success();

    // Add a new commit to be ahead
    fs::write(repo_path.join("file.txt"), "arbitrary").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(&repo_path)
        .assert()
        .success();

    Command::new("git")
        .args(["commit", "-m", "local commit"])
        .current_dir(&repo_path)
        .assert()
        .success();

    // Now test for ahead count
    Command::cargo_bin("git-x")
        .unwrap()
        .arg("info")
        .current_dir(&repo_path)
        .assert()
        .success()
        .stdout(contains("⬆️ Ahead: 1"))
        .stdout(contains("⬇️ Behind: 0"));
}

#[test]
fn test_info_output_shows_behind() {
    let repo_path = init_test_repo("init.txt", "init", "initial");

    // Create a dummy remote
    let remote_path = tempfile::tempdir().unwrap().keep();
    Command::new("git")
        .arg("init")
        .arg("--bare")
        .arg(remote_path.to_str().unwrap())
        .assert()
        .success();

    // Add the remote and push
    Command::new("git")
        .args(["remote", "add", "origin", remote_path.to_str().unwrap()])
        .current_dir(&repo_path)
        .assert()
        .success();

    Command::new("git")
        .args(["push", "-u", "origin", "test-branch"])
        .current_dir(&repo_path)
        .assert()
        .success();

    // Clone the remote into another temp dir and make a new commit
    let temp_clone = tempdir().unwrap();
    let clone_path = temp_clone.path();

    Command::new("git")
        .args(["clone", remote_path.to_str().unwrap(), "."])
        .current_dir(clone_path)
        .assert()
        .success();

    Command::new("git")
        .args(["checkout", "test-branch"])
        .current_dir(clone_path)
        .assert()
        .success();

    fs::write(clone_path.join("file2.txt"), "commit from remote").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(clone_path)
        .assert()
        .success();

    Command::new("git")
        .args(["commit", "-m", "commit from remote"])
        .current_dir(clone_path)
        .assert()
        .success();

    Command::new("git")
        .args(["push"])
        .current_dir(clone_path)
        .assert()
        .success();

    // Pull from local repo to see we are behind
    Command::new("git")
        .args(["fetch"])
        .current_dir(&repo_path)
        .assert()
        .success();

    Command::cargo_bin("git-x")
        .unwrap()
        .arg("info")
        .current_dir(&repo_path)
        .assert()
        .success()
        .stdout(contains("⬆️ Ahead: 0"))
        .stdout(contains("⬇️ Behind: 1"));
}

fn init_test_repo(file_name: &str, file_content: &str, commit_message: &str) -> std::path::PathBuf {
    let dir = tempdir().expect("Failed to create temp dir");
    let repo_path = dir.keep();

    // Initialize Git repo and checkout branch
    Command::new("git")
        .arg("init")
        .current_dir(&repo_path)
        .assert()
        .success();
    Command::new("git")
        .args(["checkout", "-b", "test-branch"])
        .current_dir(&repo_path)
        .assert()
        .success();

    // Add file and commit
    fs::write(repo_path.join(file_name), file_content).unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(&repo_path)
        .assert()
        .success();
    Command::new("git")
        .args(["commit", "-m", commit_message])
        .current_dir(&repo_path)
        .assert()
        .success();

    repo_path
}
