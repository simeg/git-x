mod common;

use git_x::color_graph::*;
use std::process::{ExitStatus, Output};

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
    assert_eq!(
        convert_output_to_string(b"git log output"),
        "git log output"
    );
}

#[test]
fn test_color_graph_run_function() {
    let repo = common::basic_repo();

    // Change to repo directory and run the function directly
    std::env::set_current_dir(repo.path()).unwrap();

    // Test that the function doesn't panic and git commands work
    git_x::color_graph::run();
}

#[test]
fn test_print_git_output() {
    let test_output = b"* commit1 message\n* commit2 message";

    // Test that print_git_output doesn't panic
    git_x::color_graph::print_git_output(test_output);
}

#[test]
fn test_print_git_error() {
    let test_error = b"not a git repository";

    // Test that print_git_error doesn't panic
    git_x::color_graph::print_git_error(test_error);
}

#[test]
fn test_color_graph_run_function_in_non_git_directory() {
    let temp_dir = tempfile::tempdir().unwrap();

    // Change to non-git directory to trigger error path
    std::env::set_current_dir(temp_dir.path()).unwrap();

    // Test that the function handles git command failure gracefully
    git_x::color_graph::run();
}
