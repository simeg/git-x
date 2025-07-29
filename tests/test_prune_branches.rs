mod common;

use common::repo_with_merged_branch;
use git_x::prune_branches::*;
use predicates::boolean::PredicateBooleanExt;
use predicates::str::contains;

#[test]
fn test_prune_branches_deletes_merged_branch() {
    let repo = repo_with_merged_branch("feature/delete-me", "main");

    repo.run_git_x(&["prune-branches"])
        .success()
        .stdout(contains("Deleted branch feature/delete-me"));
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
        .stdout(contains("Deleted branch feature/delete-me"))
        .stdout(contains("✅ No merged branches to prune").not());
}

#[test]
fn test_get_all_protected_branches() {
    let default_only = get_all_protected_branches(None);
    assert_eq!(default_only, vec!["main", "master", "develop"]);

    let with_except = get_all_protected_branches(Some("feature,hotfix"));
    assert_eq!(
        with_except,
        vec!["main", "master", "develop", "feature", "hotfix"]
    );
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
fn test_prune_branches_run_function() {
    let repo = repo_with_merged_branch("feature/delete-me", "main");

    std::env::set_current_dir(repo.path()).expect("Failed to change directory");

    // Set environment variable to make it non-interactive for testing
    unsafe {
        std::env::set_var("GIT_X_NON_INTERACTIVE", "1");
    }

    // Test run function directly without exceptions
    let result = run(None, false);
    assert!(result.is_ok());

    unsafe {
        std::env::remove_var("GIT_X_NON_INTERACTIVE");
    }
    std::env::set_current_dir("/").expect("Failed to reset directory");
}

#[test]
fn test_prune_branches_run_function_with_except() {
    let repo = repo_with_merged_branch("feature/delete-me", "main");

    std::env::set_current_dir(repo.path()).expect("Failed to change directory");

    // Set environment variable to make it non-interactive for testing
    unsafe {
        std::env::set_var("GIT_X_NON_INTERACTIVE", "1");
    }

    // Test run function with exceptions
    let result = run(Some("main,master".to_string()), false);
    assert!(result.is_ok());

    unsafe {
        std::env::remove_var("GIT_X_NON_INTERACTIVE");
    }
    std::env::set_current_dir("/").expect("Failed to reset directory");
}

#[test]
fn test_prune_branches_run_function_dry_run() {
    let repo = repo_with_merged_branch("feature/delete-me", "main");

    std::env::set_current_dir(repo.path()).expect("Failed to change directory");

    // Test run function with dry-run enabled
    let result = run(None, true);
    assert!(result.is_ok());

    std::env::set_current_dir("/").expect("Failed to reset directory");
}

#[test]
fn test_prune_branches_run_function_dry_run_with_except() {
    let repo = repo_with_merged_branch("feature/delete-me", "main");

    std::env::set_current_dir(repo.path()).expect("Failed to change directory");

    // Test run function with dry-run and exceptions
    let result = run(Some("main,master".to_string()), true);
    assert!(result.is_ok());

    std::env::set_current_dir("/").expect("Failed to reset directory");
}

#[test]
fn test_prune_branches_run_function_error_handling() {
    // Test error handling outside of git repository
    let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
    std::env::set_current_dir(temp_dir.path()).expect("Failed to change directory");

    let result = run(None, false);
    assert!(result.is_err());

    std::env::set_current_dir("/").expect("Failed to reset directory");
}

// Test dry-run CLI integration
#[test]
fn test_prune_branches_dry_run_flag() {
    let repo = repo_with_merged_branch("feature/delete-me", "main");

    repo.run_git_x(&["prune-branches", "--dry-run"])
        .success()
        .stdout(contains("🧪 (dry run)"))
        .stdout(contains("branches would be deleted:"));
}

#[test]
fn test_prune_branches_dry_run_with_except() {
    let repo = repo_with_merged_branch("feature/delete-me", "main");

    // Create another merged branch
    repo.create_branch("feature/keep-me");
    repo.checkout_branch("main");
    repo.merge_branch("feature/keep-me");

    repo.run_git_x(&["prune-branches", "--dry-run", "--except", "feature/keep-me"])
        .success()
        .stdout(contains("🧪 (dry run)"))
        .stdout(contains("feature/delete-me"));
}
