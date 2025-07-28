use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::process::Command as StdCommand;
use tempfile::TempDir;

mod common;

#[test]
fn test_technical_debt_in_non_git_repo() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.current_dir(temp_dir.path())
        .arg("technical-debt")
        .assert()
        .success()
        .stderr(predicate::str::contains("Git command failed"));
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

    // Configure git user for commits
    StdCommand::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to set git user name");

    StdCommand::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to set git user email");

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.current_dir(temp_dir.path())
        .arg("technical-debt")
        .assert()
        .success()
        .stdout(predicate::str::contains("Technical Debt Analysis"));
}

#[test]
fn test_technical_debt_with_basic_repo() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Initialize git repo
    StdCommand::new("git")
        .args(["init"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to init git repo");

    // Configure git user for commits
    StdCommand::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to set git user name");

    StdCommand::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to set git user email");

    // Create initial commit
    fs::write(temp_dir.path().join("README.md"), "# Test Repo").expect("Failed to write file");
    StdCommand::new("git")
        .args(["add", "README.md"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to add file");

    StdCommand::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to commit");

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.current_dir(temp_dir.path())
        .arg("technical-debt")
        .assert()
        .success()
        .stdout(predicate::str::contains("Technical Debt Analysis"))
        .stdout(predicate::str::contains("Large Commits"))
        .stdout(predicate::str::contains("File Hotspots"))
        .stdout(predicate::str::contains("Long-lived Branches"))
        .stdout(predicate::str::contains("Code Churn"))
        .stdout(predicate::str::contains("Binary Files"));
}

#[test]
fn test_technical_debt_detects_binary_files() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Initialize git repo
    StdCommand::new("git")
        .args(["init"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to init git repo");

    // Configure git user for commits
    StdCommand::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to set git user name");

    StdCommand::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to set git user email");

    // Create files including binary files
    fs::write(temp_dir.path().join("README.md"), "# Test Repo").expect("Failed to write file");
    fs::write(temp_dir.path().join("image.png"), b"\x89PNG\r\n\x1a\n")
        .expect("Failed to write binary file");
    fs::write(temp_dir.path().join("document.pdf"), b"%PDF-1.4")
        .expect("Failed to write binary file");

    StdCommand::new("git")
        .args(["add", "."])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to add files");

    StdCommand::new("git")
        .args(["commit", "-m", "Add files with binaries"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to commit");

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.current_dir(temp_dir.path())
        .arg("technical-debt")
        .assert()
        .success()
        .stdout(predicate::str::contains("Binary Files"))
        .stdout(predicate::str::contains("2 binary files found"));
}

#[test]
fn test_technical_debt_with_long_lived_branch() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Initialize git repo
    StdCommand::new("git")
        .args(["init"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to init git repo");

    // Configure git user for commits
    StdCommand::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to set git user name");

    StdCommand::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to set git user email");

    // Create initial commit
    fs::write(temp_dir.path().join("README.md"), "# Test Repo").expect("Failed to write file");
    StdCommand::new("git")
        .args(["add", "README.md"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to add file");

    StdCommand::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to commit");

    // Create a feature branch
    StdCommand::new("git")
        .args(["checkout", "-b", "feature/old-feature"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to create branch");

    // Add a commit to the feature branch
    fs::write(temp_dir.path().join("feature.txt"), "Feature content")
        .expect("Failed to write file");
    StdCommand::new("git")
        .args(["add", "feature.txt"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to add file");

    StdCommand::new("git")
        .args(["commit", "-m", "Add feature"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to commit");

    // Switch back to main
    StdCommand::new("git")
        .args(["checkout", "master"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to switch to master");

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.current_dir(temp_dir.path())
        .arg("technical-debt")
        .assert()
        .success()
        .stdout(predicate::str::contains("Long-lived Branches"));
}

#[test]
fn test_technical_debt_command_available() {
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("technical-debt"));
}

#[test]
fn test_technical_debt_output_format() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Initialize git repo
    StdCommand::new("git")
        .args(["init"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to init git repo");

    // Configure git user for commits
    StdCommand::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to set git user name");

    StdCommand::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to set git user email");

    // Create multiple files and commits to generate some activity
    for i in 1..=3 {
        fs::write(
            temp_dir.path().join(format!("file{i}.txt")),
            format!("Content of file {i}"),
        )
        .expect("Failed to write file");

        StdCommand::new("git")
            .args(["add", &format!("file{i}.txt")])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to add file");

        StdCommand::new("git")
            .args(["commit", "-m", &format!("Add file {i}")])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to commit");
    }

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.current_dir(temp_dir.path())
        .arg("technical-debt")
        .assert()
        .success()
        .stdout(predicate::str::contains("🔍")) // Analysis icon
        .stdout(predicate::str::contains("📊")) // Large commits icon
        .stdout(predicate::str::contains("🔥")) // Hotspots icon
        .stdout(predicate::str::contains("🌿")) // Branches icon
        .stdout(predicate::str::contains("🔄")) // Churn icon
        .stdout(predicate::str::contains("📦")) // Binary files icon
        .stdout(predicate::str::contains("Analysis complete!"));
}

#[test]
fn test_technical_debt_with_frequent_modifications() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Initialize git repo
    StdCommand::new("git")
        .args(["init"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to init git repo");

    // Configure git user for commits
    StdCommand::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to set git user name");

    StdCommand::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to set git user email");

    // Create a file and modify it multiple times to create a hotspot
    let hotspot_file = temp_dir.path().join("hotspot.rs");

    for i in 1..=8 {
        fs::write(
            &hotspot_file,
            format!("// Version {i}\nfn main() {{ println!(\"Version {i}\"); }}"),
        )
        .expect("Failed to write hotspot file");

        StdCommand::new("git")
            .args(["add", "hotspot.rs"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to add file");

        StdCommand::new("git")
            .args(["commit", "-m", &format!("Update hotspot file v{i}")])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to commit");
    }

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.current_dir(temp_dir.path())
        .arg("technical-debt")
        .assert()
        .success()
        .stdout(predicate::str::contains("File Hotspots"))
        .stdout(predicate::str::contains("hotspot.rs"));
}

#[test]
fn test_technical_debt_error_handling() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Test in directory that's not a git repo
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.current_dir(temp_dir.path())
        .arg("technical-debt")
        .assert()
        .success() // Should not crash, but show errors to stderr
        .stderr(predicate::str::contains("Git command failed"));
}
