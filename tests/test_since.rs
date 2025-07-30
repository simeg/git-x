mod common;

use common::repo_with_commits;
use git_x::commands::analysis::SinceCommand;
use git_x::core::traits::Command;
use predicates::str::contains;

#[test]
fn test_git_xsince_outputs_commits_since_ref() {
    let repo = repo_with_commits(2);

    repo.run_git_x(&["since", "HEAD~1"])
        .success()
        .stdout(contains("🔍 Commits since HEAD~1:"))
        .stdout(contains("commit 2"));
}

#[test]
fn test_git_xsince_no_new_commits() {
    let repo = repo_with_commits(2);

    repo.run_git_x(&["since", "HEAD"])
        .success()
        .stdout(contains("✅ No new commits since HEAD"));
}

#[test]
fn test_since_command_direct() {
    let repo = repo_with_commits(3);
    let original_dir = std::env::current_dir().unwrap();

    std::env::set_current_dir(repo.path()).unwrap();

    let cmd = SinceCommand::new("HEAD~1".to_string());
    let result = cmd.execute();

    // Should succeed and return formatted output
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("🔍 Commits since HEAD~1:"));

    // Restore original directory
    let _ = std::env::set_current_dir(&original_dir);
}

#[test]
fn test_since_command_no_commits() {
    let repo = common::basic_repo();
    let original_dir = std::env::current_dir().unwrap();

    // Change to repo directory
    std::env::set_current_dir(repo.path()).unwrap();

    // Test with a reference that should show no commits
    let cmd = SinceCommand::new("HEAD".to_string());
    let result = cmd.execute();

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("✅ No new commits since HEAD"));

    // Restore original directory
    let _ = std::env::set_current_dir(&original_dir);
}

#[test]
fn test_since_command_traits() {
    let cmd = SinceCommand::new("HEAD~1".to_string());

    // Test Command trait implementation
    assert_eq!(cmd.name(), "since");
    assert_eq!(
        cmd.description(),
        "Show commits since a reference (e.g., cb676ec, origin/main) or time"
    );
}
