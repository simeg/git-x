use std::path::Path;
use std::process::Command;

pub fn run() {
    // Get repo name
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .expect("Failed to get repo root");
    let repo_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let repo_name = Path::new(&repo_path)
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "unknown".to_string());

    // Get current branch
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .expect("Failed to get current branch");
    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Get upstream tracking branch
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"])
        .output()
        .unwrap_or_else(|_| panic!("Failed to get upstream for {branch}"));
    let tracking_raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let tracking = if tracking_raw.is_empty() {
        "(no upstream)".to_string()
    } else {
        tracking_raw
    };

    // Get ahead/behind counts
    let output = Command::new("git")
        .args(["rev-list", "--left-right", "--count", "HEAD...@{u}"])
        .output()
        .expect("Failed to get ahead/behind count");
    let counts = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let parts: Vec<&str> = counts.split_whitespace().collect();
    let ahead = parts.first().unwrap_or(&"0");
    let behind = parts.get(1).unwrap_or(&"0");

    // Get last commit message and relative date
    let output = Command::new("git")
        .args(["log", "-1", "--pretty=format:%s (%cr)"])
        .output()
        .expect("Failed to get last commit");
    let last_commit = String::from_utf8_lossy(&output.stdout).trim().to_string();

    let bold = console::Style::new().bold();

    // Print all the info
    println!("📂 Repo: {}", bold.apply_to(repo_name));
    println!("🔀 Branch: {}", bold.apply_to(branch));
    println!("🌿 Tracking: {}", bold.apply_to(tracking));
    println!(
        "⬆️ Ahead: {} ⬇️ Behind: {}",
        bold.apply_to(ahead),
        bold.apply_to(behind)
    );
    println!("📌 Last Commit: \"{}\"", bold.apply_to(last_commit));
}
