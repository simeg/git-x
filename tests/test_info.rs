mod common;

use common::{repo_with_branch, repo_with_remote_ahead};
use predicates::str::contains;

#[test]
fn test_info_output_contains_expected_lines() {
    let repo = repo_with_branch("test-branch");

    repo.run_git_x(&["info"])
        .success()
        .stdout(contains("Repo:"))
        .stdout(contains("Branch: test-branch"))
        .stdout(contains("Last Commit: \"initial commit"));
}

#[test]
fn test_info_output_includes_ahead_behind() {
    let repo = repo_with_branch("test-branch");
    let _remote = repo.setup_remote("test-branch");

    // Add a new commit to be ahead
    repo.add_commit("file.txt", "arbitrary", "local commit");

    repo.run_git_x(&["info"])
        .success()
        .stdout(contains("Ahead: 1"))
        .stdout(contains("Behind: 0"));
}

#[test]
fn test_info_output_shows_behind() {
    let (repo, _remote) = repo_with_remote_ahead("test-branch");

    repo.run_git_x(&["info"])
        .success()
        .stdout(contains("Ahead: 0"))
        .stdout(contains("Behind: 1"));
}
