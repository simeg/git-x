use serial_test::serial;
mod common;

use common::repo_with_conventional_commits;
use git_x::commands::analysis::SummaryCommand;
use git_x::core::traits::Command;
use predicates::str::contains;

#[test]
#[serial]
fn test_git_summary_shows_grouped_commits() {
    let repo = repo_with_conventional_commits();

    repo.run_git_x(&["summary", "--since", "3 days ago"])
        .success()
        .stdout(contains("3 days ago"))
        .stdout(contains("✨ feat: initial commit"))
        .stdout(contains("🐛 fix: bug fix"));
}

#[test]
#[serial]
fn test_summary_command_direct() {
    let repo = repo_with_conventional_commits();
    let original_dir = std::env::current_dir().unwrap();

    std::env::set_current_dir(repo.path()).unwrap();

    let cmd = SummaryCommand::new(Some("1 day ago".to_string()));
    let result = cmd.execute();

    // Should succeed and return formatted output
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Commit Summary since"));

    // Restore original directory
    let _ = std::env::set_current_dir(&original_dir);
}

#[test]
#[serial]
fn test_summary_command_no_since() {
    let repo = common::basic_repo();
    let original_dir = std::env::current_dir().unwrap();

    // Change to repo directory
    std::env::set_current_dir(repo.path()).unwrap();

    // Test with no since parameter (should show repository summary)
    let cmd = SummaryCommand::new(None);
    let result = cmd.execute();

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Repository Summary"));

    // Restore original directory
    let _ = std::env::set_current_dir(&original_dir);
}
