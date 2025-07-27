use std::process::Command;

pub fn run(reference: String) {
    let output = Command::new("git")
        .args([
            "log",
            &format_git_log_range(&reference),
            "--pretty=format:- %h %s",
        ])
        .output()
        .expect("Failed to run git log");

    if !output.status.success() {
        eprintln!("❌ Failed to retrieve commits since '{reference}'");
        return;
    }

    let log = String::from_utf8_lossy(&output.stdout);
    if is_log_empty(&log) {
        println!("✅ No new commits since {reference}");
    } else {
        println!("🔍 Commits since {reference}:");
        println!("{log}");
    }
}

// Helper function to format git log range
pub fn format_git_log_range(reference: &str) -> String {
    format!("{reference}..HEAD")
}

// Helper function to check if log output is empty
pub fn is_log_empty(log_output: &str) -> bool {
    log_output.trim().is_empty()
}


