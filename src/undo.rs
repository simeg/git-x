use std::process::Command;

pub fn run() {
    let status = Command::new("git")
        .args(get_git_reset_args())
        .status()
        .expect("Failed to execute git reset");

    if status.success() {
        println!("{}", format_success_message());
    } else {
        eprintln!("{}", format_error_message());
    }
}

// Helper function to get git reset command args
pub fn get_git_reset_args() -> [&'static str; 3] {
    ["reset", "--soft", "HEAD~1"]
}

// Helper function to format success message
pub fn format_success_message() -> &'static str {
    "Last commit undone (soft reset). Changes kept in working directory."
}

// Helper function to format error message
pub fn format_error_message() -> &'static str {
    "❌ Failed to undo last commit."
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_git_reset_args() {
        assert_eq!(get_git_reset_args(), ["reset", "--soft", "HEAD~1"]);
    }

    #[test]
    fn test_format_success_message() {
        assert_eq!(
            format_success_message(),
            "Last commit undone (soft reset). Changes kept in working directory."
        );
    }

    #[test]
    fn test_format_error_message() {
        assert_eq!(format_error_message(), "❌ Failed to undo last commit.");
    }
}
