use crate::{GitXError, Result};
use std::collections::BTreeMap;
use std::process::Command;

use chrono::{NaiveDate, Utc};

pub fn run(since: String) {
    match run_summary(&since) {
        Ok(output) => print!("{output}"),
        Err(e) => eprintln!("{}", crate::common::Format::error(&e.to_string())),
    }
}

fn run_summary(since: &str) -> Result<String> {
    let git_log = Command::new("git")
        .args(get_git_log_summary_args(since))
        .output()
        .map_err(GitXError::Io)?;

    if !git_log.status.success() {
        let stderr = String::from_utf8_lossy(&git_log.stderr);
        return Err(GitXError::GitCommand(format!(
            "git log failed: {}",
            stderr.trim()
        )));
    }

    let stdout = String::from_utf8_lossy(&git_log.stdout);
    if is_stdout_empty(&stdout) {
        return Ok(format_no_commits_message(since));
    }

    let grouped = parse_git_log_output(&stdout);
    Ok(format_commit_summary(since, &grouped))
}

// Helper function to get git log summary args
pub fn get_git_log_summary_args(since: &str) -> Vec<&str> {
    vec![
        "log",
        "--since",
        since,
        "--pretty=format:%h|%ad|%s|%an|%cr",
        "--date=short",
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
pub fn format_commit_summary(since: &str, grouped: &BTreeMap<NaiveDate, Vec<String>>) -> String {
    let mut result = format_summary_header(since);

    for (date, commits) in grouped.iter().rev() {
        result.push_str(&format_date_header(date));
        result.push('\n');
        for commit in commits {
            result.push_str(commit);
            result.push('\n');
        }
        result.push('\n');
    }

    result
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
    // Use case-insensitive matching without allocation
    let msg_bytes = message.as_bytes();
    if msg_bytes.windows(3).any(|w| w.eq_ignore_ascii_case(b"fix"))
        || msg_bytes.windows(3).any(|w| w.eq_ignore_ascii_case(b"bug"))
    {
        "🐛"
    } else if msg_bytes
        .windows(4)
        .any(|w| w.eq_ignore_ascii_case(b"feat"))
        || msg_bytes.windows(3).any(|w| w.eq_ignore_ascii_case(b"add"))
    {
        "✨"
    } else if msg_bytes
        .windows(6)
        .any(|w| w.eq_ignore_ascii_case(b"remove"))
        || msg_bytes
            .windows(6)
            .any(|w| w.eq_ignore_ascii_case(b"delete"))
    {
        "🔥"
    } else if msg_bytes
        .windows(8)
        .any(|w| w.eq_ignore_ascii_case(b"refactor"))
    {
        "🛠"
    } else {
        "🔹"
    }
}
