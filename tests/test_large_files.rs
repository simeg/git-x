use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;
use tempfile::TempDir;

use git_x::commands::analysis::LargeFilesCommand;
use git_x::core::traits::Command as CommandTrait;

fn create_test_repo_with_files() -> (TempDir, PathBuf) {
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

    // Create files of different sizes
    for (name, content) in [
        ("small.txt", "small file".as_bytes()),
        ("medium.txt", &vec![b'x'; 1024]),
        ("large.txt", &vec![b'x'; 1024 * 1024]),
    ] {
        let mut file = fs::File::create(repo_path.join(name)).expect("Failed to create file");
        use std::io::Write;
        file.write_all(content).expect("Failed to write");
        file.sync_all().expect("Failed to sync");
    }

    // Give it time to persist the files to disk
    sleep(Duration::from_millis(1000));

    // Add and commit files
    Command::new("git")
        .args(["add", "."])
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

#[test]
fn test_large_files_run_function_outside_git_repo() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["large-files"])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("❌ Git command failed"));
}

#[test]
fn test_large_files_command_help() {
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["large-files", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Find largest files in repository history",
        ));
}

#[test]
fn test_large_files_with_limit() {
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["large-files", "--limit", "5", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Number of files to show"));
}

#[test]
fn test_large_files_with_threshold() {
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["large-files", "--threshold", "1.5", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Minimum file size in MB"));
}

#[test]
fn test_large_files_default_limit() {
    let (temp_dir, repo_path) = create_test_repo_with_files();

    // Test with default limit (should be 10)
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["large-files"])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("MB"));

    // Keep temp_dir alive
    drop(temp_dir);
}

#[test]
fn test_large_files_command_direct() {
    let (temp_dir, repo_path) = create_test_repo_with_files();
    let original_dir = std::env::current_dir().unwrap();

    std::env::set_current_dir(&repo_path).unwrap();

    let cmd = LargeFilesCommand::new(Some(1.0), Some(10));
    let result = cmd.execute();

    // Should succeed and return formatted output
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("MB") || output.contains("No files"));

    // Restore original directory
    let _ = std::env::set_current_dir(&original_dir);
    drop(temp_dir);
}

#[test]
fn test_large_files_command_with_threshold() {
    let (temp_dir, repo_path) = create_test_repo_with_files();
    let original_dir = std::env::current_dir().unwrap();

    // Change to repo directory
    std::env::set_current_dir(&repo_path).unwrap();

    // Test with different threshold
    let cmd = LargeFilesCommand::new(Some(0.1), Some(5));
    let result = cmd.execute();

    assert!(result.is_ok());

    // Restore original directory
    let _ = std::env::set_current_dir(&original_dir);
    drop(temp_dir);
}
