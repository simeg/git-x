use crate::command::Command;
use crate::{GitXError, Result};
use std::collections::BTreeMap;
use std::process::Command as StdCommand;

use chrono::{NaiveDate, Utc};

pub fn run(since: String) -> Result<()> {
    let cmd = SummaryCommand;
    cmd.execute(since)
}

/// Command implementation for git summary
pub struct SummaryCommand;

impl Command for SummaryCommand {
    type Input = String;
    type Output = ();

    fn execute(&self, since: String) -> Result<()> {
        run_summary(&since)
    }

    fn name(&self) -> &'static str {
        "summary"
    }

    fn description(&self) -> &'static str {
        "Show a summary of commits since a given date"
    }
}

fn run_summary(since: &str) -> Result<()> {
    let git_log = StdCommand::new("git")
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
        println!("{since}");
        return Ok(());
    }

    let grouped = parse_git_log_output(&stdout);
    print!("{}", format_commit_summary(since, &grouped));
    Ok(())
}

fn get_git_log_summary_args(since: &str) -> Vec<&str> {
    vec![
        "log",
        "--since",
        since,
        "--pretty=format:%h|%ad|%s|%an|%cr",
        "--date=short",
    ]
}

fn is_stdout_empty(stdout: &str) -> bool {
    stdout.trim().is_empty()
}

pub fn parse_git_log_output(stdout: &str) -> BTreeMap<NaiveDate, Vec<String>> {
    let mut grouped: BTreeMap<NaiveDate, Vec<String>> = BTreeMap::new();

    for line in stdout.lines() {
        if let Some((date, formatted_commit)) = parse_commit_line(line) {
            grouped.entry(date).or_default().push(formatted_commit);
        }
    }

    grouped
}

pub fn parse_commit_line(line: &str) -> Option<(NaiveDate, String)> {
    let parts: Vec<&str> = line.splitn(5, '|').collect();
    if parts.len() != 5 {
        return None;
    }

    let date = parse_commit_date(parts[1])?;
    let message = parts[2];
    let entry = format!(" - {} {}", get_commit_emoji_public(message), message.trim());
    let author = parts[3];
    let time = parts[4];
    let meta = format!("(by {author}, {time})");
    Some((date, format!("{entry} {meta}")))
}

pub fn parse_commit_date(date_str: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .ok()
        .or_else(|| Some(Utc::now().date_naive()))
}

pub fn format_commit_summary(since: &str, grouped: &BTreeMap<NaiveDate, Vec<String>>) -> String {
    let mut result = since.to_string();

    for (date, commits) in grouped.iter().rev() {
        result.push_str(&date.to_string());
        result.push('\n');
        for commit in commits {
            result.push_str(commit);
            result.push('\n');
        }
        result.push('\n');
    }

    result
}

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
