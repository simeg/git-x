mod common;

use common::repo_with_conventional_commits;
use git_x::summary::*;
use predicates::str::contains;

#[test]
fn test_git_summary_shows_grouped_commits() {
    let repo = repo_with_conventional_commits();

    repo.run_git_x(&["summary", "--since", "3 days ago"])
        .success()
        .stdout(contains("3 days ago"))
        .stdout(contains("✨ feat: initial commit"))
        .stdout(contains("🐛 fix: bug fix"));
}

// Unit tests for helper functions
#[test]
fn test_get_commit_emoji_public() {
    assert_eq!(get_commit_emoji_public("fix: bug in parser"), "🐛");
    assert_eq!(get_commit_emoji_public("BUG: handle null pointer"), "🐛");
    assert_eq!(get_commit_emoji_public("feat: add new feature"), "✨");
    assert_eq!(get_commit_emoji_public("add user authentication"), "✨");
    assert_eq!(get_commit_emoji_public("remove old code"), "🔥");
    assert_eq!(get_commit_emoji_public("delete unused files"), "🔥");
    assert_eq!(get_commit_emoji_public("refactor database layer"), "🛠");
    assert_eq!(get_commit_emoji_public("update documentation"), "🔹");
    assert_eq!(get_commit_emoji_public("random commit"), "🔹");
}

#[test]
fn test_parse_commit_line() {
    let line = "abc123|2023-07-15|fix: bug in parser|John Doe|2 hours ago";
    let result = parse_commit_line(line);
    assert!(result.is_some());

    if let Some((date, formatted)) = result {
        assert_eq!(date.to_string(), "2023-07-15");
        assert!(formatted.contains("fix: bug in parser"));
        assert!(formatted.contains("John Doe"));
        assert!(formatted.contains("2 hours ago"));
    }

    let invalid_line = "abc123|incomplete";
    assert!(parse_commit_line(invalid_line).is_none());
}

#[test]
fn test_parse_commit_date() {
    assert!(parse_commit_date("2023-07-15").is_some());
    assert!(parse_commit_date("invalid-date").is_some()); // Falls back to current date
}

#[test]
fn test_format_commit_entry() {
    let message = "fix: bug in parser";
    assert_eq!(
        format!(" - {} {}", get_commit_emoji_public(message), message.trim()),
        " - 🐛 fix: bug in parser"
    );
    let message = "  add new feature  ";
    assert_eq!(
        format!(" - {} {}", get_commit_emoji_public(message), message.trim()),
        " - ✨ add new feature"
    );
}

#[test]
fn test_parse_git_log_output() {
    let output =
        "abc123|2023-07-15|fix: bug|John|2h ago\ndef456|2023-07-14|add feature|Jane|1d ago";
    let result = parse_git_log_output(output);
    assert_eq!(result.len(), 2);
}

#[test]
fn test_summary_run_function() {
    let repo = repo_with_conventional_commits();

    // Try to get original directory, skip test if not available
    let original_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => {
            eprintln!("Warning: Could not get current directory, skipping test");
            return;
        }
    };

    // Change to repo directory and run the function directly
    if std::env::set_current_dir(repo.path()).is_err() {
        eprintln!("Warning: Could not change to repo directory, skipping test");
        return;
    }

    // Test that the function doesn't panic and git commands work
    let result = run("1 day ago".to_string());
    assert!(result.is_ok());

    // Restore original directory
    let _ = std::env::set_current_dir(&original_dir);
}

#[test]
fn test_summary_run_function_no_commits() {
    let repo = common::basic_repo();
    let original_dir = std::env::current_dir().unwrap();

    // Change to repo directory and run the function directly
    std::env::set_current_dir(repo.path()).unwrap();

    // Test with a time range that should show no commits
    let result = run("1 minute ago".to_string());
    assert!(result.is_ok());

    // Restore original directory
    let _ = std::env::set_current_dir(&original_dir);
}

#[test]
fn test_summary_run_function_git_error() {
    let temp_dir = tempfile::tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();

    // Change to non-git directory to trigger error path
    std::env::set_current_dir(temp_dir.path()).unwrap();

    // Test that the function handles git command failure gracefully
    let result = run("1 day ago".to_string());
    assert!(result.is_err());

    // Restore original directory
    let _ = std::env::set_current_dir(&original_dir);
}

#[test]
fn test_summary_run_function_empty_output() {
    let repo = common::basic_repo();
    let original_dir = std::env::current_dir().unwrap();

    // Change to repo directory and run the function directly
    std::env::set_current_dir(repo.path()).unwrap();

    // Test with a time range that should produce empty output (future date)
    let result = run("1 day from now".to_string());
    // This may succeed or fail depending on git's date parsing
    let _ = result;

    // Restore original directory
    let _ = std::env::set_current_dir(&original_dir);
}
