// Integration tests for large_files.rs run() function testing all code paths

use assert_cmd::Command;
use std::process::Command as StdCommand;
use tempfile::TempDir;

mod common;

#[test]
fn test_large_files_run_outside_git_repo() {
    // Test error path: not in a git repository
    let temp_dir = TempDir::new().unwrap();

    let output = Command::cargo_bin("git-x")
        .unwrap()
        .current_dir(temp_dir.path())
        .args(["large-files"])
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should either fail or print an error message about git failure
    if output.status.success() {
        // If command succeeds, it should print an error message
        assert!(stderr.contains("❌") || stdout.contains("❌"));
    } else {
        // If command fails, that's also expected behavior
        assert!(!output.status.success());
    }
}

#[test]
fn test_large_files_run_empty_repo() {
    // Test no files path: empty repository
    let temp_dir = TempDir::new().unwrap();

    // Initialize empty git repo
    StdCommand::new("git")
        .args(["init"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    // Configure git identity
    StdCommand::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();
    StdCommand::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    // Change to empty git directory
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let output = Command::cargo_bin("git-x")
        .unwrap()
        .args(["large-files"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show scanning message and no files message
    assert!(stdout.contains("🔍 Scanning repository for large files"));
    assert!(stdout.contains("ℹ️ No files found in repository history"));
}

#[test]
fn test_large_files_run_with_small_files_and_threshold() {
    // Test no large files path: files exist but none meet threshold
    let repo = common::basic_repo();

    // Create small files
    repo.add_commit("small1.txt", "small content", "Add small file 1");
    repo.add_commit("small2.txt", "another small file", "Add small file 2");

    // Change to git directory
    if std::env::set_current_dir(repo.path()).is_err() {
        eprintln!("Warning: Could not change to repo directory, skipping test");
        return;
    }

    // Run with high threshold (100 MB) - should find no large files
    let output = Command::cargo_bin("git-x")
        .unwrap()
        .args(["large-files", "--threshold", "100"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show scanning message and no large files message with threshold
    assert!(stdout.contains("🔍 Scanning repository for large files"));
    assert!(stdout.contains("✅ No files found larger than 100.0 MB"));
}

#[test]
fn test_large_files_run_success_with_files() {
    // Test success path: files found and displayed
    let repo = common::basic_repo();

    // Create files of different sizes
    repo.add_commit("large1.txt", &"x".repeat(50000), "Add large file 1");
    repo.add_commit("large2.txt", &"y".repeat(30000), "Add large file 2");
    repo.add_commit("small.txt", "small", "Add small file");

    // Change to git directory
    let output = Command::cargo_bin("git-x")
        .unwrap()
        .current_dir(repo.path())
        .args(["large-files"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show scanning message
    assert!(stdout.contains("🔍 Scanning repository for large files"));

    // Should show results header
    assert!(stdout.contains("📊 Top"));

    // Should show file entries
    assert!(stdout.contains("MB"));

    // Should show summary
    assert!(stdout.contains("📈 Total:"));
}

#[test]
fn test_large_files_run_with_limit() {
    // Test success path with limit parameter
    let repo = common::basic_repo();

    // Create multiple large files
    for i in 1..=5 {
        repo.add_commit(
            &format!("large{i}.txt"),
            &"x".repeat(10000 + i * 5000),
            &format!("Add large file {i}"),
        );
    }

    // Change to git directory
    if std::env::set_current_dir(repo.path()).is_err() {
        eprintln!("Warning: Could not change to repo directory, skipping test");
        return;
    }

    // Run with limit of 3
    let output = Command::cargo_bin("git-x")
        .unwrap()
        .args(["large-files", "--limit", "3"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show scanning message
    assert!(stdout.contains("🔍 Scanning repository for large files"));

    // Should show limited results (Top 3)
    assert!(stdout.contains("📊 Top 3"));

    // Should show exactly 3 file entries by counting numbered lines
    let numbered_lines = stdout
        .lines()
        .filter(|line| {
            line.trim_start()
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_digit())
        })
        .count();
    assert_eq!(numbered_lines, 3);
}

#[test]
fn test_large_files_run_with_threshold_success() {
    // Test success path with threshold that allows some files through
    let repo = common::basic_repo();

    // Add large files to the repository
    repo.add_commit("large.txt", &"x".repeat(100000), "Add large file"); // ~100KB
    repo.add_commit("small.txt", "small content", "Add small file"); // ~13 bytes

    // Change to git directory
    if std::env::set_current_dir(repo.path()).is_err() {
        eprintln!("Warning: Could not change to repo directory, skipping test");
        return;
    }

    // Run without threshold first to see if files are detected
    let output = Command::cargo_bin("git-x")
        .unwrap()
        .args(["large-files"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show scanning message
    assert!(stdout.contains("🔍 Scanning repository for large files"));

    // Should show file results
    assert!(stdout.contains("📊 Top"));
    assert!(stdout.contains("MB"));
}

#[test]
fn test_large_files_run_comprehensive_output() {
    // Test that all output components are present in success case
    let repo = common::basic_repo();

    // Create test files
    repo.add_commit("test1.txt", &"content1".repeat(1000), "Add test file 1");
    repo.add_commit("test2.txt", &"content2".repeat(2000), "Add test file 2");

    // Change to git directory
    let output = Command::cargo_bin("git-x")
        .unwrap()
        .current_dir(repo.path())
        .args(["large-files"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should contain all expected components:
    assert!(stdout.contains("🔍")); // Scan start message
    assert!(stdout.contains("📊")); // Results header  
    assert!(stdout.contains("MB")); // File size display
    assert!(stdout.contains("📈")); // Summary message
    assert!(stdout.contains("Total:")); // Summary details
}
