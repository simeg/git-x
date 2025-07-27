mod common;

use common::repo_with_merged_branch;
use predicates::boolean::PredicateBooleanExt;
use predicates::str::contains;
use git_x::prune_branches::*;

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

// Unit tests for helper functions
#[test]
fn test_get_default_protected_branches() {
    assert_eq!(get_default_protected_branches(), vec!["main", "master", "develop"]);
}

#[test]
fn test_parse_except_branches() {
    assert_eq!(
        parse_except_branches("feature,hotfix"),
        vec!["feature".to_string(), "hotfix".to_string()]
    );
    assert_eq!(
        parse_except_branches("  branch1  ,  branch2  "),
        vec!["branch1".to_string(), "branch2".to_string()]
    );
    assert_eq!(parse_except_branches("single"), vec!["single".to_string()]);
    assert_eq!(parse_except_branches(""), Vec::<String>::new());
}

#[test]
fn test_get_all_protected_branches() {
    let default_only = get_all_protected_branches(None);
    assert_eq!(default_only, vec!["main", "master", "develop"]);
    
    let with_except = get_all_protected_branches(Some("feature,hotfix"));
    assert_eq!(with_except, vec!["main", "master", "develop", "feature", "hotfix"]);
}

#[test]
fn test_clean_git_branch_name() {
    assert_eq!(clean_git_branch_name("  feature/test  "), "feature/test");
    assert_eq!(clean_git_branch_name("* main"), "main");
    assert_eq!(clean_git_branch_name("develop"), "develop");
}

#[test]
fn test_is_branch_protected() {
    let protected = vec!["main".to_string(), "develop".to_string()];
    
    assert!(is_branch_protected("main", "current", &protected));
    assert!(is_branch_protected("develop", "current", &protected));
    assert!(is_branch_protected("current", "current", &protected));
    assert!(!is_branch_protected("feature", "current", &protected));
}

#[test]
fn test_get_git_branch_delete_args() {
    assert_eq!(
        get_git_branch_delete_args("feature"),
        ["branch".to_string(), "-d".to_string(), "feature".to_string()]
    );
}

#[test]
fn test_format_branch_deleted_message() {
    assert_eq!(
        format_branch_deleted_message("feature/test"),
        "🧹 Deleted merged branch 'feature/test'"
    );
}

#[test]
fn test_format_branch_delete_failed_message() {
    assert_eq!(
        format_branch_delete_failed_message("feature/test"),
        "⚠️ Failed to delete branch 'feature/test'"
    );
}

#[test]
fn test_format_no_branches_to_prune_message() {
    assert_eq!(format_no_branches_to_prune_message(), "✅ No merged branches to prune.");
}
