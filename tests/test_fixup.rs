mod common;

use assert_cmd::Command;
use common::basic_repo;
use git_x::fixup::*;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_is_valid_commit_hash_format() {
    // Valid hashes
    assert!(is_valid_commit_hash_format("abc123"));
    assert!(is_valid_commit_hash_format("1234567890abcdef"));
    assert!(is_valid_commit_hash_format(
        "abcdef1234567890abcdef1234567890abcdef12"
    ));

    // Invalid hashes
    assert!(!is_valid_commit_hash_format(""));
    assert!(!is_valid_commit_hash_format("abc"));
    assert!(!is_valid_commit_hash_format("xyz123")); // invalid hex chars
    assert!(!is_valid_commit_hash_format(
        "1234567890abcdef1234567890abcdef123456789"
    )); // too long
    assert!(!is_valid_commit_hash_format("abc 123")); // contains space
}

#[test]
fn test_get_commit_hash_validation_rules() {
    let rules = get_commit_hash_validation_rules();
    assert!(!rules.is_empty());
    assert!(rules.contains(&"Must be 4-40 characters long"));
    assert!(rules.contains(&"Must contain only hex characters (0-9, a-f)"));
    assert!(rules.contains(&"Must reference an existing commit"));
}

#[test]
fn test_get_git_fixup_args() {
    assert_eq!(get_git_fixup_args(), ["commit", "--fixup"]);
}

#[test]
fn test_get_git_rebase_args() {
    assert_eq!(get_git_rebase_args(), ["rebase", "-i", "--autosquash"]);
}

#[test]
fn test_format_error_message() {
    assert_eq!(format_error_message("Test error"), "❌ Test error");
    assert_eq!(
        format_error_message("Commit not found"),
        "❌ Commit not found"
    );
}

#[test]
fn test_format_no_changes_message() {
    assert_eq!(
        format_no_changes_message(),
        "❌ No staged changes found. Please stage your changes first with 'git add'"
    );
}

#[test]
fn test_format_creating_fixup_message() {
    assert_eq!(
        format_creating_fixup_message("abc123"),
        "🔧 Creating fixup commit for abc123..."
    );
    assert_eq!(
        format_creating_fixup_message("def456"),
        "🔧 Creating fixup commit for def456..."
    );
}

#[test]
fn test_format_fixup_created_message() {
    assert_eq!(
        format_fixup_created_message("abc123"),
        "✅ Fixup commit created for abc123"
    );
    assert_eq!(
        format_fixup_created_message("def456"),
        "✅ Fixup commit created for def456"
    );
}

#[test]
fn test_format_starting_rebase_message() {
    assert_eq!(
        format_starting_rebase_message(),
        "🔄 Starting interactive rebase with autosquash..."
    );
}

#[test]
fn test_format_rebase_success_message() {
    assert_eq!(
        format_rebase_success_message(),
        "✅ Interactive rebase completed successfully"
    );
}

#[test]
fn test_format_manual_rebase_hint() {
    assert_eq!(
        format_manual_rebase_hint("abc123"),
        "💡 To squash the fixup commit, run: git rebase -i --autosquash abc123^"
    );
    assert_eq!(
        format_manual_rebase_hint("def456"),
        "💡 To squash the fixup commit, run: git rebase -i --autosquash def456^"
    );
}

#[test]
fn test_fixup_run_function_outside_git_repo() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["fixup", "abc123"])
        .current_dir(temp_dir.path())
        .assert()
        .success() // The command succeeds but shows an error message
        .stderr(predicate::str::contains("Commit hash does not exist"));
}

#[test]
fn test_fixup_invalid_commit_hash() {
    let repo = basic_repo();

    repo.run_git_x(&["fixup", "nonexistent123"])
        .success() // The command succeeds but shows an error message
        .stderr(predicate::str::contains("Commit hash does not exist"));
}

#[test]
fn test_fixup_no_staged_changes() {
    let repo = basic_repo();
    let commit_hash = repo.create_commit_with_hash("test.txt", "test content", "Test commit");

    repo.run_git_x(&["fixup", &commit_hash[0..7]]) // Use short hash
        .success() // The command succeeds but shows an error message
        .stderr(predicate::str::contains("No staged changes found"));
}

#[test]
fn test_fixup_with_staged_changes() {
    let repo = basic_repo();
    let commit_hash = repo.create_commit_with_hash("test.txt", "test content", "Test commit");

    // Create and stage some changes
    fs::write(repo.path().join("test.txt"), "modified content").expect("Failed to write file");
    repo.stage_files(&["test.txt"]);

    repo.run_git_x(&["fixup", &commit_hash[0..7]]) // Use short hash
        .success()
        .stdout(predicate::str::contains("Fixup commit created"))
        .stdout(predicate::str::contains("To squash the fixup commit"));
}

#[test]
fn test_fixup_with_rebase_flag() {
    let repo = basic_repo();
    let commit_hash = repo.create_commit_with_hash("test.txt", "test content", "Test commit");

    // Create and stage some changes
    fs::write(repo.path().join("test.txt"), "modified content").expect("Failed to write file");
    repo.stage_files(&["test.txt"]);

    // Set environment to make interactive rebase work in tests
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["fixup", &commit_hash[0..7], "--rebase"])
        .current_dir(repo.path())
        .env("GIT_SEQUENCE_EDITOR", "true") // Auto-accept rebase plan
        .assert()
        .success()
        .stdout(predicate::str::contains("Fixup commit created"));
}

#[test]
fn test_fixup_command_help() {
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["fixup", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Create fixup commits for easier interactive rebasing",
        ));
}

#[test]
fn test_fixup_rebase_flag() {
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["fixup", "abc123", "--rebase", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Automatically rebase with autosquash after creating fixup",
        ));
}
