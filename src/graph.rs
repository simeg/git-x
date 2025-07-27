use std::process::Command;

pub fn run() {
    let output = Command::new("git")
        .args(get_git_log_args())
        .output()
        .expect("Failed to run git log");

    if output.status.success() {
        let result = String::from_utf8_lossy(&output.stdout);
        println!("{result}");
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        eprintln!("{}", format_git_error(&err));
    }
}

// Helper function to get git log arguments
pub fn get_git_log_args() -> [&'static str; 5] {
    ["log", "--oneline", "--graph", "--decorate", "--all"]
}

// Helper function to format error message
pub fn format_git_error(stderr: &str) -> String {
    format!("❌ git log failed:\n{stderr}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_git_log_args() {
        assert_eq!(
            get_git_log_args(),
            ["log", "--oneline", "--graph", "--decorate", "--all"]
        );
    }

    #[test]
    fn test_format_git_error() {
        assert_eq!(
            format_git_error("not a git repository"),
            "❌ git log failed:\nnot a git repository"
        );
        assert_eq!(
            format_git_error("unknown revision"),
            "❌ git log failed:\nunknown revision"
        );
    }
}
