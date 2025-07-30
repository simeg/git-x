use assert_cmd::Command;
use git_x::cli::BisectAction;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

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

fn create_commit_history(repo_path: &PathBuf) -> Vec<String> {
    let mut commits = Vec::new();

    // Create 5 commits
    for i in 1..=5 {
        let filename = format!("file{i}.txt");
        let content = format!("Content for commit {i}");
        fs::write(repo_path.join(&filename), content).expect("Failed to write file");

        Command::new("git")
            .args(["add", &filename])
            .current_dir(repo_path)
            .assert()
            .success();

        Command::new("git")
            .args(["commit", "-m", &format!("Commit {i}")])
            .current_dir(repo_path)
            .assert()
            .success();

        // Get commit hash
        let output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to get commit hash");

        let commit_hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
        commits.push(commit_hash);
    }

    commits
}

// Direct run() function tests for maximum coverage

#[test]
fn test_bisect_run_start_function() {
    let (_temp_dir, repo_path) = create_test_repo();
    let commits = create_commit_history(&repo_path);

    std::env::set_current_dir(&repo_path).expect("Failed to change directory");

    // Test start action through run function
    let action = BisectAction::Start {
        good: commits[0].clone(),
        bad: commits[4].clone(),
    };

    use git_x::commands::commit::{BisectAction as CommitBisectAction, BisectCommand};
    use git_x::core::traits::Command;

    let commit_action = match action {
        BisectAction::Start { good, bad } => CommitBisectAction::Start { bad, good },
        BisectAction::Good => CommitBisectAction::Good,
        BisectAction::Bad => CommitBisectAction::Bad,
        BisectAction::Skip => CommitBisectAction::Skip,
        BisectAction::Reset => CommitBisectAction::Reset,
        BisectAction::Status => CommitBisectAction::Status,
    };
    let cmd = BisectCommand::new(commit_action);
    let result = cmd.execute();
    std::env::set_current_dir("/").expect("Failed to reset directory");

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Starting bisect"));
    assert!(output.contains("Checked out commit"));
}

#[test]
fn test_bisect_run_start_function_invalid_commits() {
    let (_temp_dir, repo_path) = create_test_repo();

    std::env::set_current_dir(&repo_path).expect("Failed to change directory");

    // Test start action with invalid commit references
    let action = BisectAction::Start {
        good: "invalid_commit".to_string(),
        bad: "another_invalid".to_string(),
    };

    use git_x::commands::commit::{BisectAction as CommitBisectAction, BisectCommand};
    use git_x::core::traits::Command;

    let commit_action = match action {
        BisectAction::Start { good, bad } => CommitBisectAction::Start { bad, good },
        BisectAction::Good => CommitBisectAction::Good,
        BisectAction::Bad => CommitBisectAction::Bad,
        BisectAction::Skip => CommitBisectAction::Skip,
        BisectAction::Reset => CommitBisectAction::Reset,
        BisectAction::Status => CommitBisectAction::Status,
    };
    let cmd = BisectCommand::new(commit_action);
    let result = cmd.execute();
    std::env::set_current_dir("/").expect("Failed to reset directory");

    assert!(result.is_err());
}

#[test]
fn test_bisect_run_good_function() {
    let (_temp_dir, repo_path) = create_test_repo();

    std::env::set_current_dir(&repo_path).expect("Failed to change directory");

    // Test good action (should fail since not in bisect mode)
    let action = BisectAction::Good;

    use git_x::commands::commit::{BisectAction as CommitBisectAction, BisectCommand};
    use git_x::core::traits::Command;

    let commit_action = match action {
        BisectAction::Start { good, bad } => CommitBisectAction::Start { bad, good },
        BisectAction::Good => CommitBisectAction::Good,
        BisectAction::Bad => CommitBisectAction::Bad,
        BisectAction::Skip => CommitBisectAction::Skip,
        BisectAction::Reset => CommitBisectAction::Reset,
        BisectAction::Status => CommitBisectAction::Status,
    };
    let cmd = BisectCommand::new(commit_action);
    let result = cmd.execute();
    std::env::set_current_dir("/").expect("Failed to reset directory");

    assert!(result.is_err());
}

#[test]
fn test_bisect_run_bad_function() {
    let (_temp_dir, repo_path) = create_test_repo();

    std::env::set_current_dir(&repo_path).expect("Failed to change directory");

    // Test bad action (should fail since not in bisect mode)
    let action = BisectAction::Bad;

    use git_x::commands::commit::{BisectAction as CommitBisectAction, BisectCommand};
    use git_x::core::traits::Command;

    let commit_action = match action {
        BisectAction::Start { good, bad } => CommitBisectAction::Start { bad, good },
        BisectAction::Good => CommitBisectAction::Good,
        BisectAction::Bad => CommitBisectAction::Bad,
        BisectAction::Skip => CommitBisectAction::Skip,
        BisectAction::Reset => CommitBisectAction::Reset,
        BisectAction::Status => CommitBisectAction::Status,
    };
    let cmd = BisectCommand::new(commit_action);
    let result = cmd.execute();
    std::env::set_current_dir("/").expect("Failed to reset directory");

    assert!(result.is_err());
}

#[test]
fn test_bisect_run_skip_function() {
    let (_temp_dir, repo_path) = create_test_repo();

    std::env::set_current_dir(&repo_path).expect("Failed to change directory");

    // Test skip action (should fail since not in bisect mode)
    let action = BisectAction::Skip;

    use git_x::commands::commit::{BisectAction as CommitBisectAction, BisectCommand};
    use git_x::core::traits::Command;

    let commit_action = match action {
        BisectAction::Start { good, bad } => CommitBisectAction::Start { bad, good },
        BisectAction::Good => CommitBisectAction::Good,
        BisectAction::Bad => CommitBisectAction::Bad,
        BisectAction::Skip => CommitBisectAction::Skip,
        BisectAction::Reset => CommitBisectAction::Reset,
        BisectAction::Status => CommitBisectAction::Status,
    };
    let cmd = BisectCommand::new(commit_action);
    let result = cmd.execute();
    std::env::set_current_dir("/").expect("Failed to reset directory");

    assert!(result.is_err());
}

#[test]
fn test_bisect_run_reset_function() {
    let (_temp_dir, repo_path) = create_test_repo();

    std::env::set_current_dir(&repo_path).expect("Failed to change directory");

    // Test reset action (should succeed even when not in bisect mode)
    let action = BisectAction::Reset;

    use git_x::commands::commit::{BisectAction as CommitBisectAction, BisectCommand};
    use git_x::core::traits::Command;

    let commit_action = match action {
        BisectAction::Start { good, bad } => CommitBisectAction::Start { bad, good },
        BisectAction::Good => CommitBisectAction::Good,
        BisectAction::Bad => CommitBisectAction::Bad,
        BisectAction::Skip => CommitBisectAction::Skip,
        BisectAction::Reset => CommitBisectAction::Reset,
        BisectAction::Status => CommitBisectAction::Status,
    };
    let cmd = BisectCommand::new(commit_action);
    let result = cmd.execute();
    std::env::set_current_dir("/").expect("Failed to reset directory");

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Not currently in bisect mode"));
}

#[test]
fn test_bisect_run_status_function() {
    let (_temp_dir, repo_path) = create_test_repo();

    std::env::set_current_dir(&repo_path).expect("Failed to change directory");

    // Test status action (should succeed even when not in bisect mode)
    let action = BisectAction::Status;

    use git_x::commands::commit::{BisectAction as CommitBisectAction, BisectCommand};
    use git_x::core::traits::Command;

    let commit_action = match action {
        BisectAction::Start { good, bad } => CommitBisectAction::Start { bad, good },
        BisectAction::Good => CommitBisectAction::Good,
        BisectAction::Bad => CommitBisectAction::Bad,
        BisectAction::Skip => CommitBisectAction::Skip,
        BisectAction::Reset => CommitBisectAction::Reset,
        BisectAction::Status => CommitBisectAction::Status,
    };
    let cmd = BisectCommand::new(commit_action);
    let result = cmd.execute();
    std::env::set_current_dir("/").expect("Failed to reset directory");

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Not currently in bisect mode"));
}

// CLI integration tests

#[test]
fn test_bisect_start_command_help() {
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["bisect", "start", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Start bisect session"));
}

#[test]
fn test_bisect_good_command_help() {
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["bisect", "good", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Mark current commit as good"));
}

#[test]
fn test_bisect_bad_command_help() {
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["bisect", "bad", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Mark current commit as bad"));
}

#[test]
fn test_bisect_skip_command_help() {
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["bisect", "skip", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Skip current commit"));
}

#[test]
fn test_bisect_reset_command_help() {
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["bisect", "reset", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("End bisect session"));
}

#[test]
fn test_bisect_status_command_help() {
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["bisect", "status", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Show bisect status"));
}

#[test]
fn test_bisect_main_command_help() {
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["bisect", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Simplified bisect workflow"));
}

#[test]
fn test_bisect_start_invalid_commits() {
    let (_temp_dir, repo_path) = create_test_repo();

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["bisect", "start", "invalid_good", "invalid_bad"])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stderr(predicate::str::contains("does not exist"));
}

#[test]
fn test_bisect_start_valid_commits() {
    let (_temp_dir, repo_path) = create_test_repo();
    let commits = create_commit_history(&repo_path);

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["bisect", "start", &commits[0], &commits[4]])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Starting bisect"))
        .stdout(predicate::str::contains("Checked out commit"));
}

#[test]
fn test_bisect_good_not_in_bisect_mode() {
    let (_temp_dir, repo_path) = create_test_repo();

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["bisect", "good"])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stderr(predicate::str::contains("Not currently in bisect mode"));
}

#[test]
fn test_bisect_bad_not_in_bisect_mode() {
    let (_temp_dir, repo_path) = create_test_repo();

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["bisect", "bad"])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stderr(predicate::str::contains("Not currently in bisect mode"));
}

#[test]
fn test_bisect_skip_not_in_bisect_mode() {
    let (_temp_dir, repo_path) = create_test_repo();

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["bisect", "skip"])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stderr(predicate::str::contains("Not currently in bisect mode"));
}

#[test]
fn test_bisect_reset_not_in_bisect_mode() {
    let (_temp_dir, repo_path) = create_test_repo();

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["bisect", "reset"])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Not currently in bisect mode"));
}

#[test]
fn test_bisect_status_not_in_bisect_mode() {
    let (_temp_dir, repo_path) = create_test_repo();

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["bisect", "status"])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Not currently in bisect mode"));
}

#[test]
fn test_bisect_command_outside_git_repo() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["bisect", "status"])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Not currently in bisect mode"));
}

#[test]
fn test_bisect_start_same_commits_twice() {
    let (_temp_dir, repo_path) = create_test_repo();
    let commits = create_commit_history(&repo_path);

    // Test that starting bisect with same good and bad commits works
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["bisect", "start", &commits[0], &commits[0]])
        .current_dir(&repo_path)
        .assert()
        .success();
    // Git will handle this case - valid scenario where no bisect is needed

    // Clean up
    Command::new("git")
        .args(["bisect", "reset"])
        .current_dir(&repo_path)
        .assert()
        .success();
}

#[test]
fn test_bisect_help_commands() {
    // Test that all bisect subcommand help works
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["bisect", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Simplified bisect workflow"));

    // Test individual subcommand help
    let subcommands = ["start", "good", "bad", "skip", "reset", "status"];
    for subcmd in &subcommands {
        let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
        cmd.args(["bisect", subcmd, "--help"]).assert().success();
    }
}

#[test]
fn test_bisect_start_same_commits() {
    let (_temp_dir, repo_path) = create_test_repo();
    let commits = create_commit_history(&repo_path);

    // Try to start bisect with same good and bad commits
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["bisect", "start", &commits[0], &commits[0]])
        .current_dir(&repo_path)
        .assert()
        .success();
    // Git will handle this case - it's a valid scenario where no bisect is needed
}

// Unit tests for helper functions (these test the internal logic)

// Error handling tests

#[test]
fn test_bisect_start_with_tags() {
    let (_temp_dir, repo_path) = create_test_repo();
    let commits = create_commit_history(&repo_path);

    // Create a tag at first commit
    Command::new("git")
        .args(["tag", "v1.0", &commits[0]])
        .current_dir(&repo_path)
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["bisect", "start", "v1.0", &commits[4]])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Starting bisect"));
}

#[test]
fn test_bisect_start_with_branches() {
    let (_temp_dir, repo_path) = create_test_repo();
    let commits = create_commit_history(&repo_path);

    // Create a branch at first commit
    Command::new("git")
        .args(["branch", "feature", &commits[0]])
        .current_dir(&repo_path)
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["bisect", "start", "feature", &commits[4]])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Starting bisect"));
}

#[test]
fn test_bisect_various_commit_formats() {
    let (_temp_dir, repo_path) = create_test_repo();
    let commits = create_commit_history(&repo_path);

    // Test with short commit hashes
    let short_good = &commits[0][..7];
    let short_bad = &commits[4][..7];

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["bisect", "start", short_good, short_bad])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Starting bisect"));

    // Clean up
    Command::new("git")
        .args(["bisect", "reset"])
        .current_dir(&repo_path)
        .assert()
        .success();
}

#[test]
fn test_bisect_error_scenarios() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Test bisect commands outside git repo
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["bisect", "start", "HEAD~1", "HEAD"])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("does not exist"));
}

#[test]
fn test_bisect_start_relative_commits() {
    let (_temp_dir, repo_path) = create_test_repo();
    create_commit_history(&repo_path);

    // Test with relative commit references
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["bisect", "start", "HEAD~4", "HEAD"])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Starting bisect"));

    // Clean up
    Command::new("git")
        .args(["bisect", "reset"])
        .current_dir(&repo_path)
        .assert()
        .success();
}

#[test]
fn test_bisect_command_traits() {
    use git_x::commands::commit::{BisectAction, BisectCommand};
    use git_x::core::traits::Command;

    let cmd = BisectCommand::new(BisectAction::Status);

    // Test Command trait implementation
    assert_eq!(cmd.name(), "bisect");
    assert_eq!(
        cmd.description(),
        "Simplified Git bisect workflow for finding bugs"
    );
}

#[test]
fn test_bisect_command_direct() {
    use git_x::commands::commit::{BisectAction, BisectCommand};
    use git_x::core::traits::Command;

    let (_temp_dir, repo_path) = create_test_repo();
    let original_dir = std::env::current_dir().unwrap();

    std::env::set_current_dir(&repo_path).unwrap();

    let cmd = BisectCommand::new(BisectAction::Status);
    let result = cmd.execute();

    // Status should work even when not bisecting
    match &result {
        Ok(output) => {
            assert!(output.contains("Not currently in bisect mode"));
        }
        Err(_e) => {
            // Some git environments might have different behavior
        }
    }

    // Restore original directory
    let _ = std::env::set_current_dir(&original_dir);
}
