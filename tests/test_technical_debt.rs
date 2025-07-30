use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::process::Command as StdCommand;
use tempfile::TempDir;

mod common;

use git_x::commands::analysis::TechnicalDebtCommand;
use git_x::core::traits::Command as CommandTrait;

#[test]
fn test_technical_debt_in_non_git_repo() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.current_dir(temp_dir.path())
        .arg("technical-debt")
        .assert()
        .success()
        .stdout(predicate::str::contains("Technical Debt Analysis"));
}

#[test]
fn test_technical_debt_in_empty_git_repo() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Initialize git repo
    StdCommand::new("git")
        .args(["init"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to init git repo");

    // Configure git
    StdCommand::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to configure git");

    StdCommand::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to configure git");

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.current_dir(temp_dir.path())
        .arg("technical-debt")
        .assert()
        .success()
        .stdout(predicate::str::contains("Technical Debt Analysis"));
}

#[test]
fn test_technical_debt_help() {
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["technical-debt", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Analyze code complexity and technical debt metrics",
        ));
}

#[test]
fn test_technical_debt_with_files() {
    let repo = common::basic_repo();

    // Add some files to make the analysis more interesting
    fs::write(repo.path().join("src").join("main.rs"), "fn main() {}\n").ok();
    fs::write(repo.path().join("README.md"), "# Test Repo\n").ok();

    // Add and commit files
    StdCommand::new("git")
        .args(["add", "."])
        .current_dir(repo.path())
        .output()
        .expect("Failed to add files");

    StdCommand::new("git")
        .args(["commit", "-m", "Add initial files"])
        .current_dir(repo.path())
        .output()
        .expect("Failed to commit files");

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.current_dir(repo.path())
        .arg("technical-debt")
        .assert()
        .success()
        .stdout(predicate::str::contains("Technical Debt Analysis"));
}

#[test]
fn test_technical_debt_command_direct() {
    let repo = common::basic_repo();
    let original_dir = std::env::current_dir().unwrap();

    std::env::set_current_dir(repo.path()).unwrap();

    let cmd = TechnicalDebtCommand::new();
    let result = cmd.execute();

    // Should succeed and return formatted output
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Technical Debt Analysis"));

    // Restore original directory
    let _ = std::env::set_current_dir(&original_dir);
}

#[test]
fn test_technical_debt_command_traits() {
    let cmd = TechnicalDebtCommand::new();

    // Test Command trait implementation
    assert_eq!(cmd.name(), "technical-debt");
    assert_eq!(cmd.description(), "Analyze technical debt indicators");
}

#[test]
fn test_technical_debt_command_outside_git_repo() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let original_dir = std::env::current_dir().unwrap();

    // Change to non-git directory
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let cmd = TechnicalDebtCommand::new();
    let result = cmd.execute();

    // The new implementation handles git command failures gracefully and still returns output
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Technical Debt Analysis"));

    // Restore original directory
    let _ = std::env::set_current_dir(&original_dir);
}
