use std::process::Command;

pub fn run() {
    let output = Command::new("git")
        .args(get_color_git_log_args())
        .output()
        .expect("Failed to run git log");

    if output.status.success() {
        let result = String::from_utf8_lossy(&output.stdout);
        println!("{result}");
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        eprintln!("{}", format_color_git_error(&err));
    }
}

// Helper function to get color git log arguments
pub fn get_color_git_log_args() -> [&'static str; 7] {
    [
        "log",
        "--oneline",
        "--graph",
        "--decorate",
        "--all",
        "--color=always",
        "--pretty=format:%C(auto)%h%d %s %C(dim)(%an, %ar)%C(reset)",
    ]
}

// Helper function to format error message
pub fn format_color_git_error(stderr: &str) -> String {
    format!("❌ git log failed:\n{stderr}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_color_git_log_args() {
        let args = get_color_git_log_args();
        assert_eq!(args[0], "log");
        assert_eq!(args[1], "--oneline");
        assert_eq!(args[2], "--graph");
        assert_eq!(args[3], "--decorate");
        assert_eq!(args[4], "--all");
        assert_eq!(args[5], "--color=always");
        assert!(args[6].contains("--pretty=format:"));
        assert!(args[6].contains("%C(auto)"));
    }

    #[test]
    fn test_format_color_git_error() {
        assert_eq!(
            format_color_git_error("not a git repository"),
            "❌ git log failed:\nnot a git repository"
        );
        assert_eq!(
            format_color_git_error("permission denied"),
            "❌ git log failed:\npermission denied"
        );
    }
}
