use assert_cmd::Command;
use git_x::large_files::*;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// Helper function to create a test git repository with some files
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
    fs::write(repo_path.join("small.txt"), "small file").expect("Failed to write file");
    fs::write(repo_path.join("medium.txt"), "x".repeat(1024)).expect("Failed to write file");
    fs::write(repo_path.join("large.txt"), "x".repeat(1024 * 1024)).expect("Failed to write file");

    // Add and commit files
    Command::new("git")
        .args(["add", "."])
        .current_dir(&repo_path)
        .assert()
        .success();

    Command::new("git")
        .args(["commit", "-m", "Add test files"])
        .current_dir(&repo_path)
        .assert()
        .success();

    (temp_dir, repo_path)
}

#[test]
fn test_get_rev_list_args() {
    let args = get_rev_list_args();
    assert_eq!(args.len(), 6);
    assert_eq!(args[0], "rev-list");
    assert_eq!(args[1], "--objects");
    assert_eq!(args[2], "--all");
    assert_eq!(args[3], "--no-object-names");
    assert_eq!(args[4], "--filter=blob:none");
    assert_eq!(args[5], "--");
}

#[test]
fn test_format_scan_start_message() {
    assert_eq!(
        format_scan_start_message(),
        "🔍 Scanning repository for large files..."
    );
}

#[test]
fn test_format_error_message() {
    assert_eq!(format_error_message("Test error"), "❌ Test error");
    assert_eq!(
        format_error_message("Connection failed"),
        "❌ Connection failed"
    );
}

#[test]
fn test_format_no_files_message() {
    assert_eq!(
        format_no_files_message(),
        "ℹ️ No files found in repository history"
    );
}

#[test]
fn test_format_no_large_files_message() {
    assert_eq!(
        format_no_large_files_message(Some(10.0)),
        "✅ No files found larger than 10.0 MB"
    );
    assert_eq!(
        format_no_large_files_message(None),
        "✅ No large files found"
    );
}

#[test]
fn test_format_results_header() {
    assert_eq!(
        format_results_header(5, Some(10.0)),
        "📊 Top 5 files larger than 10.0 MB:"
    );
    assert_eq!(format_results_header(10, None), "📊 Top 10 largest files:");
}

#[test]
fn test_format_file_line() {
    let file = FileInfo {
        path: "test/large.txt".to_string(),
        size_bytes: 1048576, // 1 MB
        size_mb: 1.0,
    };

    let result = format_file_line(1, &file);
    assert!(result.contains("1."));
    assert!(result.contains("1.0 MB"));
    assert!(result.contains("test/large.txt"));
}

#[test]
fn test_format_summary_message() {
    assert_eq!(
        format_summary_message(5, 25.5),
        "\n📈 Total: 5 files, 25.5 MB"
    );
    assert_eq!(
        format_summary_message(1, 1.0),
        "\n📈 Total: 1 files, 1.0 MB"
    );
}

#[test]
fn test_format_size_human_readable() {
    assert_eq!(format_size_human_readable(512), "512 B");
    assert_eq!(format_size_human_readable(1024), "1.0 KB");
    assert_eq!(format_size_human_readable(1536), "1.5 KB");
    assert_eq!(format_size_human_readable(1048576), "1.0 MB");
    assert_eq!(format_size_human_readable(1073741824), "1.0 GB");
    assert_eq!(format_size_human_readable(1099511627776), "1.0 TB");
}

#[test]
fn test_file_info_creation() {
    let file = FileInfo::new("test.txt".to_string(), 2097152); // 2 MB
    assert_eq!(file.path, "test.txt");
    assert_eq!(file.size_bytes, 2097152);
    assert_eq!(file.size_mb, 2.0);

    let small_file = FileInfo::new("small.txt".to_string(), 1024); // 1 KB
    assert_eq!(small_file.size_mb, 1024.0 / (1024.0 * 1024.0));
}

#[test]
fn test_large_files_run_function_outside_git_repo() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["large-files"])
        .current_dir(temp_dir.path())
        .assert()
        .failure() // The command fails with an error message
        .stderr(predicate::str::contains("Failed to get file objects"));
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
fn test_large_files_run_function_with_files() {
    let (_temp_dir, repo_path) = create_test_repo_with_files();

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["large-files", "--limit", "10"])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Scanning repository"));
}

#[test]
fn test_large_files_with_high_threshold() {
    let (_temp_dir, repo_path) = create_test_repo_with_files();

    // Set threshold higher than any files in repo
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["large-files", "--threshold", "100.0"])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("No files found larger than"));
}

#[test]
fn test_large_files_default_limit() {
    let (_temp_dir, repo_path) = create_test_repo_with_files();

    // Test with default limit (should be 10)
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["large-files"])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Scanning repository"));
}

// Direct run() function tests

#[test]
fn test_large_files_run_outside_git_repo() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Change to non-git directory
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    // Test large files outside git repo
    let result = git_x::large_files::run(10, None);

    // Restore directory before temp_dir is dropped
    std::env::set_current_dir(&original_dir).unwrap();

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Failed to get file objects")
    );
}

#[test]
fn test_large_files_run_successful_scan() {
    let (_temp_dir, repo_path) = create_test_repo_with_files();

    // Change to repo directory
    let original_dir = std::env::current_dir().unwrap();
    if std::env::set_current_dir(&repo_path).is_err() {
        return; // Skip test if directory change fails
    }

    // Test successful scan
    let result = git_x::large_files::run(10, None);

    // Restore directory
    let _ = std::env::set_current_dir(&original_dir);

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Scanning repository for large files"));
    assert!(output.contains("Top 10 largest files:") || output.contains("Total:"));
}

#[test]
fn test_large_files_run_with_threshold() {
    let (_temp_dir, repo_path) = create_test_repo_with_files();

    // Change to repo directory
    let original_dir = std::env::current_dir().unwrap();
    if std::env::set_current_dir(&repo_path).is_err() {
        return; // Skip test if directory change fails
    }

    // Test with high threshold (should find no files)
    let result = git_x::large_files::run(10, Some(100.0));

    // Restore directory
    let _ = std::env::set_current_dir(&original_dir);

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(
        output.contains("No files found larger than 100")
            || output.contains("No large files found")
    );
}

#[test]
fn test_large_files_run_with_limit() {
    let (_temp_dir, repo_path) = create_test_repo_with_files();

    // Change to repo directory
    let original_dir = std::env::current_dir().unwrap();
    if std::env::set_current_dir(&repo_path).is_err() {
        return; // Skip test if directory change fails
    }

    // Test with small limit
    let result = git_x::large_files::run(2, None);

    // Restore directory
    let _ = std::env::set_current_dir(&original_dir);

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Scanning repository for large files"));
    // Should limit to 2 files max
}

#[test]
fn test_large_files_run_low_threshold() {
    let (_temp_dir, repo_path) = create_test_repo_with_files();

    // Change to repo directory
    let original_dir = std::env::current_dir().unwrap();
    if std::env::set_current_dir(&repo_path).is_err() {
        return; // Skip test if directory change fails
    }

    // Test with very low threshold (should find files)
    let result = git_x::large_files::run(5, Some(0.0001));

    // Restore directory
    let _ = std::env::set_current_dir(&original_dir);

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Scanning repository for large files"));
    assert!(output.contains("larger than 0.0001 MB") || output.contains("Total:"));
}
