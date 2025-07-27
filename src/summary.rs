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
    format!("(by {author}, {time})")
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
