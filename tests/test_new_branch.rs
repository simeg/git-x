use assert_cmd::Command;
use git_x::new_branch::*;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// Helper function to create a test git repository
fn create_test_repo() -> (TempDir, PathBuf, String) {
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

    // Get the actual default branch name
    let branch_output = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to get current branch");
    let default_branch = String::from_utf8_lossy(&branch_output.stdout)
        .trim()
        .to_string();

    (temp_dir, repo_path, default_branch)
}

#[test]
fn test_get_validation_rules() {
    let rules = get_validation_rules();
    assert!(!rules.is_empty());
    assert!(rules.contains(&"Cannot be empty"));
    assert!(rules.contains(&"Cannot start with a dash"));
    assert!(rules.contains(&"Cannot contain '..'"));
    assert!(rules.contains(&"Cannot contain spaces"));
    assert!(rules.contains(&"Cannot contain ~^:?*[\\"));
}

#[test]
fn test_format_error_message() {
    assert_eq!(format_error_message("Test error"), "❌ Test error");
    assert_eq!(
        format_error_message("Invalid branch name"),
        "❌ Invalid branch name"
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
fn test_format_invalid_base_message() {
    assert_eq!(
        format_invalid_base_message("nonexistent"),
        "❌ Base branch or ref 'nonexistent' does not exist"
    );
    assert_eq!(
        format_invalid_base_message("unknown-branch"),
        "❌ Base branch or ref 'unknown-branch' does not exist"
    );
}

#[test]
fn test_format_creating_branch_message() {
    assert_eq!(
        format_creating_branch_message("feature", "main"),
        "🌿 Creating branch 'feature' from 'main'..."
    );
    assert_eq!(
        format_creating_branch_message("hotfix", "develop"),
        "🌿 Creating branch 'hotfix' from 'develop'..."
    );
}

#[test]
fn test_format_success_message() {
    assert_eq!(
        format_success_message("feature"),
        "✅ Created and switched to branch 'feature'"
    );
    assert_eq!(
        format_success_message("new-feature"),
        "✅ Created and switched to branch 'new-feature'"
    );
}

#[test]
fn test_get_git_branch_args() {
    assert_eq!(get_git_branch_args(), ["branch", "-"]);
}

#[test]
fn test_get_git_switch_args() {
    assert_eq!(get_git_switch_args(), ["switch", "-"]);
}

#[test]
fn test_new_branch_run_function_outside_git_repo() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["new", "test-branch"])
        .current_dir(temp_dir.path())
        .assert()
        .success() // The command succeeds but shows an error message
        .stderr(predicate::str::contains("Not in a git repository"));
}

#[test]
fn test_new_branch_creates_and_switches() {
    let (_temp_dir, repo_path, _default_branch) = create_test_repo();

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["new", "feature-branch"])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Created and switched to branch"));

    // Verify we're on the new branch
    let mut check_cmd = Command::new("git");
    check_cmd
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stdout("feature-branch\n");
}

#[test]
fn test_new_branch_with_from_option() {
    let (_temp_dir, repo_path, default_branch) = create_test_repo();

    // Create another branch first
    Command::new("git")
        .args(["checkout", "-b", "develop"])
        .current_dir(&repo_path)
        .assert()
        .success();

    // Add a commit to develop
    fs::write(repo_path.join("develop.txt"), "develop branch").expect("Failed to write file");
    Command::new("git")
        .args(["add", "develop.txt"])
        .current_dir(&repo_path)
        .assert()
        .success();
    Command::new("git")
        .args(["commit", "-m", "Add develop file"])
        .current_dir(&repo_path)
        .assert()
        .success();

    // Go back to default branch (main or master)
    Command::new("git")
        .args(["checkout", &default_branch])
        .current_dir(&repo_path)
        .assert()
        .success();

    // Create new branch from develop
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["new", "feature-from-develop", "--from", "develop"])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Created and switched to branch"));

    // Verify we're on the new branch and it has the develop file
    let mut check_cmd = Command::new("git");
    check_cmd
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stdout("feature-from-develop\n");

    // Check that develop.txt exists (showing it was created from develop)
    assert!(repo_path.join("develop.txt").exists());
}

#[test]
fn test_new_branch_invalid_name_empty() {
    let (_temp_dir, repo_path, _default_branch) = create_test_repo();

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["new", ""])
        .current_dir(&repo_path)
        .assert()
        .success() // The command succeeds but shows validation error
        .stderr(predicate::str::contains("cannot be empty"));
}

#[test]
fn test_new_branch_invalid_name_dash() {
    let (_temp_dir, repo_path, _default_branch) = create_test_repo();

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["new", "--", "-invalid"])
        .current_dir(&repo_path)
        .assert()
        .success() // The command succeeds but shows validation error
        .stderr(predicate::str::contains("is reserved"));
}

#[test]
fn test_new_branch_invalid_name_double_dot() {
    let (_temp_dir, repo_path, _default_branch) = create_test_repo();

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["new", "feature..branch"])
        .current_dir(&repo_path)
        .assert()
        .success() // The command succeeds but shows validation error
        .stderr(predicate::str::contains("not a valid branch name"));
}

#[test]
fn test_new_branch_invalid_name_spaces() {
    let (_temp_dir, repo_path, _default_branch) = create_test_repo();

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["new", "feature branch"])
        .current_dir(&repo_path)
        .assert()
        .success() // The command succeeds but shows validation error
        .stderr(predicate::str::contains("contains invalid characters"));
}

#[test]
fn test_new_branch_existing_branch() {
    let (_temp_dir, repo_path, default_branch) = create_test_repo();

    // Try to create a branch that already exists (using the actual default branch)
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["new", &default_branch])
        .current_dir(&repo_path)
        .assert()
        .success() // The command succeeds but shows error
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn test_new_branch_invalid_base() {
    let (_temp_dir, repo_path, _default_branch) = create_test_repo();

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["new", "feature", "--from", "nonexistent"])
        .current_dir(&repo_path)
        .assert()
        .success() // The command succeeds but shows error
        .stderr(predicate::str::contains("does not exist"));
}

#[test]
fn test_new_branch_command_help() {
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["new", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Create and switch to a new branch",
        ));
}

// Additional comprehensive test coverage
#[test]
fn test_message_formatting_edge_cases() {
    // Test with special characters and edge cases
    assert!(
        format_creating_branch_message("test/branch-123", "origin/main")
            .contains("test/branch-123")
    );
    assert!(
        format_creating_branch_message("test/branch-123", "origin/main").contains("origin/main")
    );

    assert!(format_success_message("feature/issue-456").contains("feature/issue-456"));
    assert!(format_branch_exists_message("hotfix/urgent").contains("hotfix/urgent"));
    assert!(
        format_invalid_base_message("refs/remotes/origin/feature")
            .contains("refs/remotes/origin/feature")
    );
}

#[test]
fn test_format_consistency() {
    // Test that all format functions return non-empty strings for reasonable inputs
    assert!(!format_error_message("test").is_empty());
    assert!(!format_branch_exists_message("test").is_empty());
    assert!(!format_invalid_base_message("test").is_empty());
    assert!(!format_creating_branch_message("test", "main").is_empty());
    assert!(!format_success_message("test").is_empty());

    // Test that they include expected emojis or symbols
    assert!(format_error_message("test").contains("❌"));
    assert!(format_branch_exists_message("test").contains("❌"));
    assert!(format_invalid_base_message("test").contains("❌"));
    assert!(format_creating_branch_message("test", "main").contains("🌿"));
    assert!(format_success_message("test").contains("✅"));
}
