mod common;

use common::repo_with_commits;
use predicates::str::contains;
use std::process::Command;

#[test]
fn test_git_xundo_soft_resets_last_commit() {
    let repo = repo_with_commits(2);

    repo.run_git_x(&["undo"])
        .success()
        .stdout(contains("Last commit undone"));

    // Verify that the commit was undone but the file changes remain
    let log = Command::new("git")
        .args(["log", "--oneline"])
        .current_dir(repo.path())
        .output()
        .unwrap();
    let log_output = String::from_utf8_lossy(&log.stdout);
    assert!(log_output.contains("initial"));
    assert!(!log_output.contains("commit 2"));

    let diff = Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .current_dir(repo.path())
        .output()
        .unwrap();
    let diff_output = String::from_utf8_lossy(&diff.stdout);
    assert!(diff_output.contains("file.txt"));
}
