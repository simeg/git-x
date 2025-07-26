use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_rename_branch_in_isolated_repo() {
    let repo_path = init_test_repo();

    // Rename the branch from test-branch to renamed-branch
    let status = Command::new("git")
        .args(["branch", "-m", "renamed-branch"])
        .current_dir(&repo_path)
        .status()
        .expect("Failed to rename branch");
    assert!(status.success());

    // Verify the current branch name
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to get current branch");

    assert!(output.status.success());
    let current_branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(current_branch, "renamed-branch");
}

fn init_test_repo() -> std::path::PathBuf {
    let dir = tempdir().expect("Failed to create temp dir");
    let repo_path = dir.keep();

    // Initialize a new git repository
    let status = Command::new("git")
        .arg("init")
        .current_dir(&repo_path)
        .status()
        .expect("Failed to init git repo");
    assert!(status.success());

    // Create an initial file and commit it
    fs::write(repo_path.join("README.md"), "# Test Repo").expect("Failed to write file");
    let status = Command::new("git")
        .args(["add", "."])
        .current_dir(&repo_path)
        .status()
        .expect("Failed to git add");
    assert!(status.success());

    let status = Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(&repo_path)
        .status()
        .expect("Failed to commit");
    assert!(status.success());

    // Create and checkout a test branch
    let status = Command::new("git")
        .args(["checkout", "-b", "test-branch"])
        .current_dir(&repo_path)
        .status()
        .expect("Failed to create branch");
    assert!(status.success());

    repo_path
}
