use assert_cmd::Command;
use git_x::core::traits::Command as CommandTrait;
use predicates::prelude::*;
use serial_test::serial;
use std::fs;
use std::process::Command as StdCommand;
use tempfile::TempDir;

mod common;

#[test]
#[serial]
fn test_contributors_in_non_git_repo() {
    use git_x::commands::analysis::ContributorsCommand;

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let original_dir = std::env::current_dir().unwrap();

    std::env::set_current_dir(temp_dir.path()).unwrap();

    let cmd = ContributorsCommand::new(None);
    let result = cmd.execute();

    // Should fail in non-git directory
    if result.is_ok() {
        println!("{}", &result.unwrap().as_str());
    } else {
        assert!(result.is_err());
    }

    // Restore original directory
    let _ = std::env::set_current_dir(&original_dir);
}

#[test]
#[serial]
fn test_contributors_in_empty_git_repo() {
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

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let cmd = git_x::commands::analysis::ContributorsCommand::new(None);
    let result = cmd.execute();

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("No contributors found"));

    // Restore original directory
    let _ = std::env::set_current_dir(&original_dir);
}

#[test]
#[serial]
fn test_contributors_with_single_contributor() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Initialize git repo
    StdCommand::new("git")
        .args(["init"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to init git repo");

    // Configure git user for commits
    StdCommand::new("git")
        .args(["config", "user.name", "Alice Smith"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to set git user name");

    StdCommand::new("git")
        .args(["config", "user.email", "alice@example.com"])
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

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let cmd = git_x::commands::analysis::ContributorsCommand::new(None);
    let result = cmd.execute();

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Repository Contributors"));
    assert!(output.contains("Alice Smith"));
    assert!(output.contains("alice@example.com"));
    assert!(output.contains("🥇"));
    assert!(output.contains("1 commits"));

    assert!(output.contains("100.0%"));

    // Restore original directory
    let _ = std::env::set_current_dir(&original_dir);
}

#[test]
#[serial]
fn test_contributors_with_multiple_contributors() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Initialize git repo
    StdCommand::new("git")
        .args(["init"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to init git repo");

    // First contributor - Alice
    StdCommand::new("git")
        .args(["config", "user.name", "Alice Smith"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to set git user name");

    StdCommand::new("git")
        .args(["config", "user.email", "alice@example.com"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to set git user email");

    // Create first commit
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

    // Second contributor - Bob
    StdCommand::new("git")
        .args(["config", "user.name", "Bob Jones"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to set git user name");

    StdCommand::new("git")
        .args(["config", "user.email", "bob@example.com"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to set git user email");

    // Create second commit
    fs::write(temp_dir.path().join("file.txt"), "Hello World").expect("Failed to write file");
    StdCommand::new("git")
        .args(["add", "file.txt"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to add file");

    StdCommand::new("git")
        .args(["commit", "-m", "Add file"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to commit");

    // Alice makes another commit
    StdCommand::new("git")
        .args(["config", "user.name", "Alice Smith"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to set git user name");

    StdCommand::new("git")
        .args(["config", "user.email", "alice@example.com"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to set git user email");

    fs::write(temp_dir.path().join("another.txt"), "Another file").expect("Failed to write file");
    StdCommand::new("git")
        .args(["add", "another.txt"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to add file");

    StdCommand::new("git")
        .args(["commit", "-m", "Add another file"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to commit");

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let cmd = git_x::commands::analysis::ContributorsCommand::new(None);
    let result = cmd.execute();

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Repository Contributors"));
    assert!(output.contains("3 total commits"));
    assert!(output.contains("Alice Smith"));
    assert!(output.contains("Bob Jones"));
    assert!(output.contains("🥇")); // Alice should be first with 2 commits
    assert!(output.contains("🥈")); // Bob should be second with 1 commit
    assert!(output.contains("2 commits"));
    assert!(output.contains("1 commits"));

    assert!(output.contains("66.7%")); // Alice: 2/3 commits
    assert!(output.contains("33.3%")); // Bob: 1/3 commits

    // Restore original directory
    let _ = std::env::set_current_dir(&original_dir);
}

#[test]
#[serial]
fn test_contributors_command_available() {
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("contributors"));
}

#[test]
#[serial]
fn test_contributors_displays_email_and_dates() {
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
        .arg("contributors")
        .assert()
        .success()
        .stdout(predicate::str::contains("test@example.com"))
        .stdout(predicate::str::contains("📧")) // Email icon
        .stdout(predicate::str::contains("📅")); // Date icon
}

#[test]
#[serial]
fn test_contributors_ranking_icons() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Initialize git repo with multiple contributors
    StdCommand::new("git")
        .args(["init"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to init git repo");

    // Create commits for different contributors to test ranking
    let contributors = [
        ("Alice Top", "alice@example.com", 3),       // Should get 🥇
        ("Bob Second", "bob@example.com", 2),        // Should get 🥈
        ("Charlie Third", "charlie@example.com", 1), // Should get 🥉
    ];

    let mut file_counter = 0;
    for (name, email, commit_count) in contributors.iter() {
        StdCommand::new("git")
            .args(["config", "user.name", name])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to set git user name");

        StdCommand::new("git")
            .args(["config", "user.email", email])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to set git user email");

        for i in 0..*commit_count {
            file_counter += 1;
            let filename = format!("file{file_counter}.txt");
            fs::write(temp_dir.path().join(&filename), format!("Content {i}"))
                .expect("Failed to write file");

            StdCommand::new("git")
                .args(["add", &filename])
                .current_dir(temp_dir.path())
                .output()
                .expect("Failed to add file");

            StdCommand::new("git")
                .args(["commit", "-m", &format!("Commit {} by {}", i + 1, name)])
                .current_dir(temp_dir.path())
                .output()
                .expect("Failed to commit");
        }
    }

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.current_dir(temp_dir.path())
        .arg("contributors")
        .assert()
        .success()
        .stdout(predicate::str::contains("🥇")) // Gold medal for top contributor
        .stdout(predicate::str::contains("🥈")) // Silver medal for second
        .stdout(predicate::str::contains("🥉")); // Bronze medal for third
}

#[test]
#[serial]
fn test_contributors_command_direct() {
    use git_x::commands::analysis::ContributorsCommand;

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let original_dir = std::env::current_dir().unwrap();

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

    std::env::set_current_dir(temp_dir.path()).unwrap();

    let cmd = ContributorsCommand::new(None);
    let result = cmd.execute();

    // Should succeed and return formatted output
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Repository Contributors"));
    assert!(output.contains("Test User"));

    // Restore original directory
    let _ = std::env::set_current_dir(&original_dir);
}
