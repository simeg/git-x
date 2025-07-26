mod common;

use common::repo_with_commits;
use predicates::str::contains;

#[test]
fn test_git_xsince_outputs_commits_since_ref() {
    let repo = repo_with_commits(2);

    repo.run_git_x(&["since", "HEAD~1"])
        .success()
        .stdout(contains("🔍 Commits since HEAD~1:"))
        .stdout(contains("commit 2"));
}

#[test]
fn test_git_xsince_no_new_commits() {
    let repo = repo_with_commits(2);

    repo.run_git_x(&["since", "HEAD"])
        .success()
        .stdout(contains("✅ No new commits since HEAD"));
}
