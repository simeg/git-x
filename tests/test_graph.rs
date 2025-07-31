mod common;

use common::basic_repo;

#[test]
fn test_git_graph_runs_without_error() {
    let repo = basic_repo();
    repo.run_git_x(&["graph"]).success();
}

#[test]
fn test_git_graph_outputs_graph_symbols() {
    use git_x::commands::analysis::GraphCommand;
    use git_x::core::traits::Command;

    let repo = basic_repo();
    let original_dir = std::env::current_dir().unwrap();

    std::env::set_current_dir(repo.path()).unwrap();

    let cmd = GraphCommand::new();
    let result = cmd.execute();

    assert!(result.is_ok());
    let stdout = result.unwrap();
    assert!(
        stdout.contains("*") || stdout.contains("|"),
        "Expected ASCII graph symbols in output"
    );

    // Restore original directory
    let _ = std::env::set_current_dir(&original_dir);
}

// Unit tests now handled by common module tests

#[test]
fn test_graph_run_function() {
    use git_x::commands::analysis::GraphCommand;
    use git_x::core::traits::Command;

    let repo = basic_repo();
    let original_dir = std::env::current_dir().unwrap();

    std::env::set_current_dir(repo.path()).unwrap();

    let cmd = GraphCommand::new();
    let result = cmd.execute();

    // Should succeed and return formatted output
    assert!(result.is_ok());

    // Restore original directory
    let _ = std::env::set_current_dir(&original_dir);
}

#[test]
fn test_graph_run_function_in_non_git_directory() {
    use git_x::commands::analysis::GraphCommand;
    use git_x::core::traits::Command;

    let temp_dir = tempfile::tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();

    // Change to non-git directory to trigger error path
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let cmd = GraphCommand::new();
    let result = cmd.execute();

    // Should fail gracefully in non-git directory
    assert!(result.is_err());

    // Restore original directory
    let _ = std::env::set_current_dir(&original_dir);
}
