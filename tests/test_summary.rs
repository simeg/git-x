mod common;

use common::repo_with_conventional_commits;
use git_x::summary::*;
use predicates::str::contains;

#[test]
fn test_git_xsummary_shows_grouped_commits() {
    let repo = repo_with_conventional_commits();

    repo.run_git_x(&["summary", "--since", "3 days ago"])
        .success()
        .stdout(contains("🗞️ Commit summary since"))
        .stdout(contains("📅"))
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
fn test_get_git_log_summary_args() {
    let args = get_git_log_summary_args("3 days ago");
    assert_eq!(args[0], "log");
    assert_eq!(args[1], "--since");
    assert_eq!(args[2], "3 days ago");
    assert!(args[3].contains("--pretty=format:"));
    assert_eq!(args[4], "--date=short");
}

#[test]
fn test_format_git_error_message() {
    assert_eq!(format_git_error_message(), "❌ Failed to retrieve commits");
}

#[test]
fn test_is_stdout_empty() {
    assert!(is_stdout_empty(""));
    assert!(is_stdout_empty("   "));
    assert!(is_stdout_empty("\n\t  \n"));
    assert!(!is_stdout_empty("some content"));
}

#[test]
fn test_format_no_commits_message() {
    assert_eq!(
        format_no_commits_message("3 days ago"),
        "✅ No commits found since 3 days ago"
    );
    assert_eq!(
        format_no_commits_message("last week"),
        "✅ No commits found since last week"
    );
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
    assert_eq!(
        format_commit_entry("fix: bug in parser"),
        " - 🐛 fix: bug in parser"
    );
    assert_eq!(
        format_commit_entry("  add new feature  "),
        " - ✨ add new feature"
    );
}

#[test]
fn test_format_commit_meta() {
    assert_eq!(
        format_commit_meta("John Doe", "2 hours ago"),
        "(by John Doe, 2 hours ago)"
    );
}

#[test]
fn test_format_summary_header() {
    assert_eq!(
        format_summary_header("3 days ago"),
        "🗞️ Commit summary since 3 days ago:\n"
    );
}

#[test]
fn test_format_date_header() {
    use chrono::NaiveDate;
    let date = NaiveDate::from_ymd_opt(2023, 7, 15).unwrap();
    assert_eq!(format_date_header(&date), "📅 2023-07-15");
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
    let repo = common::repo_with_conventional_commits();

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
    git_x::summary::run("1 day ago".to_string());

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
    git_x::summary::run("1 minute ago".to_string());

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
    git_x::summary::run("1 day ago".to_string());

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
    git_x::summary::run("1 day from now".to_string());

    // Restore original directory
    let _ = std::env::set_current_dir(&original_dir);
}
