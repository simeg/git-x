use assert_cmd::Command;
use assert_cmd::assert::OutputAssertExt;
use std::fs;
use std::process::Command as StdCommand;
use tempfile::tempdir;

#[test]
fn test_clean_branches_dry_run_outputs_expected() {
    let repo = init_test_repo_with_merged_branch();

    Command::cargo_bin("git-x")
        .unwrap()
        .arg("clean-branches")
        .arg("--dry-run")
        .current_dir(&repo)
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "(dry run) Would delete: feature/cleanup",
        ));
}

#[test]
fn test_clean_branches_actually_deletes_branch() {
    let repo = init_test_repo_with_merged_branch();

    // Sanity check: branch exists before cleanup
    let output_before = StdCommand::new("git")
        .args(["branch"])
        .current_dir(&repo)
        .output()
        .expect("Failed to list branches");
    let stdout_before = String::from_utf8_lossy(&output_before.stdout);
    assert!(stdout_before.contains("feature/cleanup"));

    // Run the command (no dry run)
    Command::cargo_bin("git-x")
        .unwrap()
        .arg("clean-branches")
        .current_dir(&repo)
        .assert()
        .success()
        .stdout(predicates::str::contains("🧹 Deleted 1 merged branches:"));

    // Confirm branch was deleted
    let output_after = StdCommand::new("git")
        .args(["branch"])
        .current_dir(&repo)
        .output()
        .expect("Failed to list branches");
    let stdout_after = String::from_utf8_lossy(&output_after.stdout);
    assert!(!stdout_after.contains("feature/cleanup"));
}

fn init_test_repo_with_merged_branch() -> std::path::PathBuf {
    let dir = tempdir().unwrap();
    let path = dir.keep();

    StdCommand::new("git")
        .arg("init")
        .current_dir(&path)
        .assert()
        .success();

    fs::write(path.join("file.txt"), "arbitrary").unwrap();
    StdCommand::new("git")
        .args(["add", "."])
        .current_dir(&path)
        .assert()
        .success();

    StdCommand::new("git")
        .args(["commit", "-m", "init"])
        .current_dir(&path)
        .assert()
        .success();

    StdCommand::new("git")
        .args(["checkout", "-b", "feature/cleanup"])
        .current_dir(&path)
        .assert()
        .success();

    fs::write(path.join("file.txt"), "arbitrary-2").unwrap();
    StdCommand::new("git")
        .args(["commit", "-am", "change"])
        .current_dir(&path)
        .assert()
        .success();

    StdCommand::new("git")
        .args(["checkout", "master"])
        .current_dir(&path)
        .assert()
        .success();

    StdCommand::new("git")
        .args(["merge", "feature/cleanup"])
        .current_dir(&path)
        .assert()
        .success();

    path
}
