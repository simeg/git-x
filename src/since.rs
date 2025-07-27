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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_git_log_range() {
        assert_eq!(format_git_log_range("main"), "main..HEAD");
        assert_eq!(
            format_git_log_range("origin/develop"),
            "origin/develop..HEAD"
        );
        assert_eq!(format_git_log_range("abc123"), "abc123..HEAD");
    }

    #[test]
    fn test_is_log_empty() {
        assert!(is_log_empty(""));
        assert!(is_log_empty("   "));
        assert!(is_log_empty("\n\t  \n"));
        assert!(!is_log_empty("- abc123 commit message"));
        assert!(!is_log_empty("some content"));
    }
}
