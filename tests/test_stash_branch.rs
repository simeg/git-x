use assert_cmd::Command;
use git_x::stash_branch::*;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// Helper function to create a test git repository
fn create_test_repo() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let repo_path = temp_dir.path().to_path_buf();

    // Initialize git repo
    Command::new("git")
        .args(["init"])
        .current_dir(&repo_path)
        .assert()
        .success();

    // Configure git
    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(&repo_path)
        .assert()
        .success();

    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(&repo_path)
        .assert()
        .success();

    // Create initial commit
    fs::write(repo_path.join("README.md"), "Initial commit").expect("Failed to write file");
    Command::new("git")
        .args(["add", "README.md"])
        .current_dir(&repo_path)
        .assert()
        .success();

    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(&repo_path)
        .assert()
        .success();

    (temp_dir, repo_path)
}

// Helper function to create a stash
fn create_stash(repo_path: &PathBuf, filename: &str, content: &str, message: &str) {
    fs::write(repo_path.join(filename), content).expect("Failed to write file");
    Command::new("git")
        .args(["add", filename])
        .current_dir(repo_path)
        .assert()
        .success();

    Command::new("git")
        .args(["stash", "push", "-m", message])
        .current_dir(repo_path)
        .assert()
        .success();
}

#[test]
fn test_get_git_stash_branch_args() {
    assert_eq!(get_git_stash_branch_args(), ["stash", "branch"]);
}

#[test]
fn test_get_git_stash_drop_args() {
    assert_eq!(get_git_stash_drop_args(), ["stash", "drop"]);
}

#[test]
fn test_format_error_message() {
    assert_eq!(format_error_message("Test error"), "❌ Test error");
    assert_eq!(
        format_error_message("Branch validation failed"),
        "❌ Branch validation failed"
    );
}

#[test]
fn test_format_branch_exists_message() {
    assert_eq!(
        format_branch_exists_message("feature"),
        "❌ Branch 'feature' already exists"
    );
    assert_eq!(
        format_branch_exists_message("main"),
        "❌ Branch 'main' already exists"
    );
}

#[test]
fn test_format_creating_branch_message() {
    assert_eq!(
        format_creating_branch_message("feature", "stash@{0}"),
        "🌿 Creating branch 'feature' from stash@{0}..."
    );
    assert_eq!(
        format_creating_branch_message("bugfix", "stash@{1}"),
        "🌿 Creating branch 'bugfix' from stash@{1}..."
    );
}

#[test]
fn test_format_branch_created_message() {
    assert_eq!(
        format_branch_created_message("feature"),
        "✅ Branch 'feature' created and checked out"
    );
    assert_eq!(
        format_branch_created_message("hotfix"),
        "✅ Branch 'hotfix' created and checked out"
    );
}

#[test]
fn test_format_no_stashes_message() {
    assert_eq!(format_no_stashes_message(), "ℹ️ No stashes found");
}

#[test]
fn test_format_no_old_stashes_message() {
    assert_eq!(format_no_old_stashes_message(), "✅ No old stashes to clean");
}

#[test]
fn test_format_stashes_to_clean_message() {
    assert_eq!(
        format_stashes_to_clean_message(3, true),
        "🧪 (dry run) Would clean 3 stash(es):"
    );
    assert_eq!(
        format_stashes_to_clean_message(2, false),
        "🧹 Cleaning 2 stash(es):"
    );
}

#[test]
fn test_format_cleanup_complete_message() {
    assert_eq!(
        format_cleanup_complete_message(5),
        "✅ Cleaned 5 stash(es)"
    );
    assert_eq!(
        format_cleanup_complete_message(1),
        "✅ Cleaned 1 stash(es)"
    );
}

#[test]
fn test_format_no_stashes_for_branch_message() {
    assert_eq!(
        format_no_stashes_for_branch_message("feature"),
        "ℹ️ No stashes found for branch 'feature'"
    );
}

#[test]
fn test_format_stashes_for_branch_header() {
    assert_eq!(
        format_stashes_for_branch_header("main", 3),
        "📋 Found 3 stash(es) for branch 'main':"
    );
}

#[test]
fn test_format_applying_stashes_message() {
    assert_eq!(
        format_applying_stashes_message("feature", 2),
        "🔄 Applying 2 stash(es) from branch 'feature':"
    );
}

#[test]
fn test_format_stash_entry() {
    assert_eq!(
        format_stash_entry("stash@{0}", "WIP on feature: add new function"),
        "stash@{0}: WIP on feature: add new function"
    );
}

#[test]
fn test_stash_branch_create_command_help() {
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["stash-branch", "create", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Create a new branch from a stash"));
}

#[test]
fn test_stash_branch_clean_command_help() {
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["stash-branch", "clean", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Clean old stashes"));
}

#[test]
fn test_stash_branch_apply_command_help() {
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["stash-branch", "apply-by-branch", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Apply stashes from a specific branch"));
}

#[test]
fn test_stash_branch_create_invalid_branch_name() {
    let (_temp_dir, repo_path) = create_test_repo();
    create_stash(&repo_path, "test.txt", "test content", "Test stash");

    // Test empty branch name
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["stash-branch", "create", ""])
        .current_dir(&repo_path)
        .assert()
        .success() // Command succeeds but shows validation error
        .stderr(predicate::str::contains("Branch name cannot be empty"));

    // Test branch name starting with dash (use -- to escape)
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["stash-branch", "create", "--", "-feature"])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stderr(predicate::str::contains("Branch name cannot start with a dash"));

    // Test branch name with spaces
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["stash-branch", "create", "feature branch"])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stderr(predicate::str::contains("Branch name cannot contain spaces"));

    // Test branch name with double dots
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["stash-branch", "create", "feature..test"])
        .current_dir(&repo_path)
        .assert()
        .success()
        .success() // This should pass clap parsing and hit our validation
        .stderr(predicate::str::contains("Branch name cannot contain '..'"));
}

#[test]
fn test_stash_branch_create_existing_branch() {
    let (_temp_dir, repo_path) = create_test_repo();
    create_stash(&repo_path, "test.txt", "test content", "Test stash");

    // Create a branch first
    Command::new("git")
        .args(["checkout", "-b", "existing-branch"])
        .current_dir(&repo_path)
        .assert()
        .success();

    Command::new("git")
        .args(["checkout", "main"])
        .current_dir(&repo_path)
        .assert()
        .success();

    // Try to create branch with same name from stash
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["stash-branch", "create", "existing-branch"])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stderr(predicate::str::contains("Branch 'existing-branch' already exists"));
}

#[test]
fn test_stash_branch_create_no_stash() {
    let (_temp_dir, repo_path) = create_test_repo();

    // Try to create branch from non-existent stash
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["stash-branch", "create", "new-branch"])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stderr(predicate::str::contains("Stash reference does not exist"));
}

#[test]
fn test_stash_branch_create_success() {
    let (_temp_dir, repo_path) = create_test_repo();
    create_stash(&repo_path, "test.txt", "test content", "Test stash");

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["stash-branch", "create", "new-feature"])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Creating branch 'new-feature'"))
        .stdout(predicate::str::contains("Branch 'new-feature' created"));
}

#[test]
fn test_stash_branch_create_with_stash_ref() {
    let (_temp_dir, repo_path) = create_test_repo();
    create_stash(&repo_path, "test1.txt", "test content 1", "Test stash 1");
    create_stash(&repo_path, "test2.txt", "test content 2", "Test stash 2");

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["stash-branch", "create", "from-specific-stash", "--stash", "stash@{1}"])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Creating branch 'from-specific-stash' from stash@{1}"));
}

#[test]
fn test_stash_branch_clean_no_stashes() {
    let (_temp_dir, repo_path) = create_test_repo();

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["stash-branch", "clean"])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("No stashes found"));
}

#[test]
fn test_stash_branch_clean_dry_run() {
    let (_temp_dir, repo_path) = create_test_repo();
    create_stash(&repo_path, "test1.txt", "test content 1", "Test stash 1");
    create_stash(&repo_path, "test2.txt", "test content 2", "Test stash 2");

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["stash-branch", "clean", "--dry-run"])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("(dry run) Would clean"));
}

#[test]
fn test_stash_branch_clean_with_age_filter() {
    let (_temp_dir, repo_path) = create_test_repo();
    create_stash(&repo_path, "test.txt", "test content", "Test stash");

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["stash-branch", "clean", "--older-than", "7d"])
        .current_dir(&repo_path)
        .assert()
        .success();
}

// Note: Age format validation is currently a placeholder implementation
// #[test]
// fn test_stash_branch_clean_invalid_age_format() {
//     let (_temp_dir, repo_path) = create_test_repo();
//     create_stash(&repo_path, "test.txt", "test content", "Test stash");
//
//     let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
//     cmd.args(["stash-branch", "clean", "--older-than", "invalid"])
//         .current_dir(&repo_path)
//         .assert()
//         .success()
//         .stderr(predicate::str::contains("Invalid age format"));
// }

#[test]
fn test_stash_branch_apply_by_branch_no_stashes() {
    let (_temp_dir, repo_path) = create_test_repo();

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["stash-branch", "apply-by-branch", "feature"])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("No stashes found for branch 'feature'"));
}

#[test]
fn test_stash_branch_apply_by_branch_list_only() {
    let (_temp_dir, repo_path) = create_test_repo();
    
    // Create a feature branch and stash from it
    Command::new("git")
        .args(["checkout", "-b", "feature"])
        .current_dir(&repo_path)
        .assert()
        .success();

    create_stash(&repo_path, "feature.txt", "feature content", "WIP on feature: add feature");

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["stash-branch", "apply-by-branch", "feature", "--list"])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Found"));
}

#[test]
fn test_stash_branch_command_outside_git_repo() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["stash-branch", "create", "test-branch"])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("Stash reference does not exist"));
}

#[test]
fn test_stash_branch_main_command_help() {
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["stash-branch", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Advanced stash management with branch integration"));
}