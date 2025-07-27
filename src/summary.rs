use std::collections::BTreeMap;
use std::process::Command;

use chrono::{NaiveDate, Utc};

pub fn run(since: String) {
    let git_log = Command::new("git")
        .args(get_git_log_summary_args(&since))
        .output()
        .expect("Failed to run git log");

    if !git_log.status.success() {
        eprintln!("{}", format_git_error_message());
        return;
    }

    let stdout = String::from_utf8_lossy(&git_log.stdout);
    if is_stdout_empty(&stdout) {
        println!("{}", format_no_commits_message(&since));
        return;
    }

    let grouped = parse_git_log_output(&stdout);
    print_commit_summary(&since, &grouped);
}

// Helper function to get git log summary args
pub fn get_git_log_summary_args(since: &str) -> Vec<String> {
    vec![
        "log".to_string(),
        "--since".to_string(),
        since.to_string(),
        "--pretty=format:%h|%ad|%s|%an|%cr".to_string(),
        "--date=short".to_string(),
    ]
}

// Helper function to format git error message
pub fn format_git_error_message() -> &'static str {
    "❌ Failed to retrieve commits"
}

// Helper function to check if stdout is empty
pub fn is_stdout_empty(stdout: &str) -> bool {
    stdout.trim().is_empty()
}

// Helper function to format no commits message
pub fn format_no_commits_message(since: &str) -> String {
    format!("✅ No commits found since {since}")
}

// Helper function to parse git log output
pub fn parse_git_log_output(stdout: &str) -> BTreeMap<NaiveDate, Vec<String>> {
    let mut grouped: BTreeMap<NaiveDate, Vec<String>> = BTreeMap::new();

    for line in stdout.lines() {
        if let Some((date, formatted_commit)) = parse_commit_line(line) {
            grouped.entry(date).or_default().push(formatted_commit);
        }
    }

    grouped
}

// Helper function to parse a single commit line
pub fn parse_commit_line(line: &str) -> Option<(NaiveDate, String)> {
    let parts: Vec<&str> = line.splitn(5, '|').collect();
    if parts.len() != 5 {
        return None;
    }

    let date = parse_commit_date(parts[1])?;
    let entry = format_commit_entry(parts[2]);
    let meta = format_commit_meta(parts[3], parts[4]);
    Some((date, format!("{entry} {meta}")))
}

// Helper function to parse commit date
pub fn parse_commit_date(date_str: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .ok()
        .or_else(|| Some(Utc::now().date_naive()))
}

// Helper function to format commit entry
pub fn format_commit_entry(message: &str) -> String {
    format!(" - {} {}", get_commit_emoji_public(message), message.trim())
}

// Helper function to format commit meta
pub fn format_commit_meta(author: &str, time: &str) -> String {
    format!("(by {}, {})", author, time)
}

// Helper function to print commit summary
pub fn print_commit_summary(since: &str, grouped: &BTreeMap<NaiveDate, Vec<String>>) {
    println!("{}", format_summary_header(since));

    for (date, commits) in grouped.iter().rev() {
        println!("{}", format_date_header(date));
        for commit in commits {
            println!("{commit}");
        }
        println!();
    }
}

// Helper function to format summary header
pub fn format_summary_header(since: &str) -> String {
    format!("🗞️ Commit summary since {since}:\n")
}

// Helper function to format date header
pub fn format_date_header(date: &NaiveDate) -> String {
    format!("📅 {date}")
}

// Helper function to get emoji for commit message (public version for testing)
pub fn get_commit_emoji_public(message: &str) -> &'static str {
    let lower_msg = message.to_lowercase();
    if lower_msg.contains("fix") || lower_msg.contains("bug") {
        "🐛"
    } else if lower_msg.contains("feat") || lower_msg.contains("add") {
        "✨"
    } else if lower_msg.contains("remove") || lower_msg.contains("delete") {
        "🔥"
    } else if lower_msg.contains("refactor") {
        "🛠"
    } else {
        "🔹"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

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
        let date = NaiveDate::from_ymd_opt(2023, 7, 15).unwrap();
        assert_eq!(format_date_header(&date), "📅 2023-07-15");
    }

    #[test]
    fn test_parse_git_log_output() {
        let output = "abc123|2023-07-15|fix: bug|John|2h ago\ndef456|2023-07-14|add feature|Jane|1d ago";
        let result = parse_git_log_output(output);
        assert_eq!(result.len(), 2);
    }
}
