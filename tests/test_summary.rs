mod common;

use common::repo_with_conventional_commits;
use predicates::str::contains;

#[test]
fn test_git_xsummary_shows_grouped_commits() {
    let repo = repo_with_conventional_commits();

    repo.run_git_x(&["summary", "--since", "3 days ago"])
        .success()
        .stdout(contains("🗞️ Commit summary since"))
        .stdout(contains("📅"))
        .stdout(contains("✨ feat: initial commit"))
        .stdout(contains("🐛 fix: bug fix"));
}
