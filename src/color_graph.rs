use crate::{GitXError, Result};
use std::process::Command;

pub fn run() {
    match run_color_graph() {
        Ok(output) => print!("{output}"),
        Err(e) => eprintln!("{}", crate::common::Format::error(&e.to_string())),
    }
}

fn run_color_graph() -> Result<String> {
    let output = Command::new("git")
        .args(get_color_git_log_args())
        .output()
        .map_err(GitXError::Io)?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(GitXError::GitCommand(format!(
            "git log failed: {}",
            stderr.trim()
        )))
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
