use std::process::Command;

pub fn run() {
    let output = Command::new("git")
        .args(get_color_git_log_args())
        .output()
        .expect("Failed to run git log");

    if is_command_successful(&output) {
        print_git_output(&output.stdout);
    } else {
        print_git_error(&output.stderr);
    }
}

// Helper function to check if command was successful
pub fn is_command_successful(output: &std::process::Output) -> bool {
    output.status.success()
}

// Helper function to print git output
pub fn print_git_output(stdout: &[u8]) {
    let result = convert_output_to_string(stdout);
    println!("{result}");
}

// Helper function to print git error
pub fn print_git_error(stderr: &[u8]) {
    let err = convert_output_to_string(stderr);
    eprintln!("{}", format_color_git_error(&err));
}

// Helper function to convert output to string
pub fn convert_output_to_string(output: &[u8]) -> String {
    String::from_utf8_lossy(output).to_string()
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
    use std::process::{Output, ExitStatus};

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

    #[test]
    fn test_is_command_successful() {
        use std::os::unix::process::ExitStatusExt;
        
        let success_output = Output {
            status: ExitStatus::from_raw(0),
            stdout: vec![],
            stderr: vec![],
        };
        assert!(is_command_successful(&success_output));

        let failure_output = Output {
            status: ExitStatus::from_raw(256), // Exit code 1
            stdout: vec![],
            stderr: vec![],
        };
        assert!(!is_command_successful(&failure_output));
    }

    #[test]
    fn test_convert_output_to_string() {
        assert_eq!(convert_output_to_string(b"hello world"), "hello world");
        assert_eq!(convert_output_to_string(b""), "");
        assert_eq!(convert_output_to_string(b"git log output"), "git log output");
    }
}
