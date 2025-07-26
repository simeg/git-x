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
