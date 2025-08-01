use serial_test::serial;
mod common;

use git_x::commands::analysis::ColorGraphCommand;
use git_x::core::traits::Command;

#[test]
#[serial]
fn test_color_graph_run_function() {
    let repo = common::basic_repo();
    let original_dir = std::env::current_dir().unwrap();

    std::env::set_current_dir(repo.path()).unwrap();

    let cmd = ColorGraphCommand::new();
    let result = cmd.execute();

    // Should succeed and return formatted output
    assert!(result.is_ok());

    // Restore original directory
    let _ = std::env::set_current_dir(&original_dir);
}

#[test]
#[serial]
fn test_color_graph_run_function_in_non_git_directory() {
    let temp_dir = tempfile::tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();

    // Change to non-git directory to trigger error path
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let cmd = ColorGraphCommand::new();
    let result = cmd.execute();

    // Should fail gracefully in non-git directory
    assert!(result.is_err());

    // Restore original directory
    let _ = std::env::set_current_dir(&original_dir);
}
