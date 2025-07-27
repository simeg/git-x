use std::collections::BTreeMap;
use std::process::Command;

use chrono::{NaiveDate, Utc};

pub fn run(since: String) {
    let git_log = Command::new("git")
        .args([
            "log",
            "--since",
            since.as_str(),
            "--pretty=format:%h|%ad|%s|%an|%cr",
            "--date=short",
        ])
        .output()
        .expect("Failed to run git log");

    if !git_log.status.success() {
        eprintln!("❌ Failed to retrieve commits");
        return;
    }

    let stdout = String::from_utf8_lossy(&git_log.stdout);
    if stdout.trim().is_empty() {
        println!("✅ No commits found since {since}");
        return;
    }

    let mut grouped: BTreeMap<NaiveDate, Vec<String>> = BTreeMap::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.splitn(5, '|').collect();
        if parts.len() != 5 {
            continue;
        }
        let date = NaiveDate::parse_from_str(parts[1], "%Y-%m-%d")
            .unwrap_or_else(|_| Utc::now().date_naive());
        let entry = format!(" - {} {}", emoji_for(parts[2]), parts[2].trim());
        let meta = format!("(by {}, {})", parts[3], parts[4]);
        grouped
            .entry(date)
            .or_default()
            .push(format!("{entry} {meta}"));
    }

    println!("🗞️ Commit summary since {since}:\n");

    for (date, commits) in grouped.iter().rev() {
        println!("📅 {date}");
        for commit in commits {
            println!("{commit}");
        }
        println!();
    }
}

fn emoji_for(message: &str) -> &str {
    let lower = message.to_lowercase();
    if lower.contains("fix") || lower.contains("bug") {
        "🐛"
    } else if lower.contains("feat") || lower.contains("add") {
        "✨"
    } else if lower.contains("remove") || lower.contains("delete") {
        "🔥"
    } else if lower.contains("refactor") {
        "🛠"
    } else {
        "🔹"
    }
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

// Helper function to format git log args
pub fn get_git_log_summary_args(since: &str) -> Vec<String> {
    vec![
        "log".to_string(),
        "--since".to_string(),
        since.to_string(),
        "--pretty=format:%h|%ad|%s|%an|%cr".to_string(),
        "--date=short".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
