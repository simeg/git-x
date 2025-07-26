mod common;

use common::repo_with_merged_branch;
use predicates::boolean::PredicateBooleanExt;
use predicates::str::contains;

#[test]
fn test_prune_branches_deletes_merged_branch() {
    let repo = repo_with_merged_branch("feature/delete-me", "main");

    repo.run_git_x(&["prune-branches"])
        .success()
        .stdout(contains("🧹 Deleted merged branch 'feature/delete-me'"));
}

#[test]
fn test_prune_branches_respects_exclude() {
    let repo = repo_with_merged_branch("feature/delete-me", "main");

    // Create another merged branch
    repo.create_branch("feature/keep-me");
    repo.checkout_branch("main");
    repo.merge_branch("feature/keep-me");

    repo.run_git_x(&["prune-branches", "--except", "feature/keep-me"])
        .success()
        .stdout(contains("🧹 Deleted merged branch 'feature/delete-me'"))
        .stdout(contains("✅ No merged branches to prune").not());
}
