mod common;

use common::repo_with_feature_ahead;
use predicates::str::contains;

#[test]
fn test_git_xwhat_shows_diff_and_commits() {
    let repo = repo_with_feature_ahead("feature/test", "main");

    repo.run_git_x(&["what"])
        .success()
        .stdout(contains("Branch: feature/test vs main"))
        .stdout(contains("+ 1 commits ahead"))
        .stdout(contains("Changes:"))
        .stdout(contains("~ file.txt"));
}
