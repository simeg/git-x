mod common;

use common::repo_with_branch;
use std::process::Command;

#[test]
fn test_rename_branch_in_isolated_repo() {
    let repo = repo_with_branch("test-branch");

    // Rename the branch from test-branch to renamed-branch
    let status = Command::new("git")
        .args(["branch", "-m", "renamed-branch"])
        .current_dir(repo.path())
        .status()
        .expect("Failed to rename branch");
    assert!(status.success());

    // Verify the current branch name
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(repo.path())
        .output()
        .expect("Failed to get current branch");

    assert!(output.status.success());
    let current_branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(current_branch, "renamed-branch");
}
