use serial_test::serial;
mod common;

use common::repo_with_merged_branch;
use git_x::commands::branch::PruneBranchesCommand;
use git_x::core::traits::Command;
use predicates::boolean::PredicateBooleanExt;
use predicates::str::contains;

// Helper to check if we should run potentially destructive tests
fn should_run_destructive_tests() -> bool {
    // Only run destructive tests in CI or when explicitly enabled
    std::env::var("CI").is_ok()
        || std::env::var("GITHUB_ACTIONS").is_ok()
        || std::env::var("ENABLE_DESTRUCTIVE_TESTS").is_ok()
}

#[test]
#[serial]
fn test_prune_branches_deletes_merged_branch() {
    if !should_run_destructive_tests() {
        return;
    }

    let repo = repo_with_merged_branch("feature/delete-me", "main");

    repo.run_git_x(&["prune-branches"])
        .success()
        .stdout(contains("Deleted branch feature/delete-me"));
}

#[test]
#[serial]
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
#[serial]
fn test_prune_branches_run_function() {
    if !should_run_destructive_tests() {
        return;
    }

    let repo = repo_with_merged_branch("feature/delete-me", "main");

    std::env::set_current_dir(repo.path()).expect("Failed to change directory");

    // Set environment variable to make it non-interactive for testing
    unsafe {
        std::env::set_var("GIT_X_NON_INTERACTIVE", "1");
    }

    let cmd = PruneBranchesCommand::new(false);
    let result = cmd.execute();
    assert!(result.is_ok());

    unsafe {
        std::env::remove_var("GIT_X_NON_INTERACTIVE");
    }
    std::env::set_current_dir("/").expect("Failed to reset directory");
}

#[test]
#[serial]
fn test_prune_branches_run_function_dry_run() {
    let repo = repo_with_merged_branch("feature/delete-me", "main");

    std::env::set_current_dir(repo.path()).expect("Failed to change directory");

    let cmd = PruneBranchesCommand::new(true);
    let result = cmd.execute();
    assert!(result.is_ok());

    std::env::set_current_dir("/").expect("Failed to reset directory");
}

#[test]
#[serial]
fn test_prune_branches_run_function_error_handling() {
    // Test error handling outside of git repository
    let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
    std::env::set_current_dir(temp_dir.path()).expect("Failed to change directory");

    let cmd = PruneBranchesCommand::new(false);
    let result = cmd.execute();
    assert!(result.is_err());

    std::env::set_current_dir("/").expect("Failed to reset directory");
}

// Test dry-run CLI integration
#[test]
#[serial]
fn test_prune_branches_dry_run_flag() {
    let repo = repo_with_merged_branch("feature/delete-me", "main");

    repo.run_git_x(&["prune-branches", "--dry-run"])
        .success()
        .stdout(contains("🧪 (dry run)"))
        .stdout(contains("branches would be deleted:"));
}

#[test]
#[serial]
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
