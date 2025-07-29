mod common;

#[test]
fn test_color_graph_run_function() {
    let repo = common::basic_repo();
    let original_dir = std::env::current_dir().unwrap();

    // Change to repo directory and run the function directly
    std::env::set_current_dir(repo.path()).unwrap();

    // Test that the function doesn't panic and git commands work
    let _ = git_x::color_graph::run();

    // Restore original directory
    let _ = std::env::set_current_dir(&original_dir);
}

#[test]
fn test_color_graph_run_function_in_non_git_directory() {
    let temp_dir = tempfile::tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();

    // Change to non-git directory to trigger error path
    std::env::set_current_dir(temp_dir.path()).unwrap();

    // Test that the function handles git command failure gracefully
    let _ = git_x::color_graph::run();

    // Restore original directory
    let _ = std::env::set_current_dir(&original_dir);
}
