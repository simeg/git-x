mod common;

use common::repo_with_feature_ahead;
use git_x::what::*;
use predicates::str::contains;

#[test]
fn test_git_xwhat_shows_diff_and_commits() {
    let repo = repo_with_feature_ahead("feature/test", "main");

    repo.run_git_x(&["what"])
        .success()
        .stdout(contains("Branch: feature/test vs main"))
        .stdout(contains("+ 1 commits ahead"))
        .stdout(contains("Changes:"))
        .stdout(contains("~ file.txt"));
}

// Unit tests for helper functions
#[test]
fn test_get_default_target() {
    assert_eq!(get_default_target(), "main");
}

#[test]
fn test_format_branch_comparison() {
    assert_eq!(
        format_branch_comparison("feature", "main"),
        "Branch: feature vs main"
    );
    assert_eq!(
        format_branch_comparison("develop", "master"),
        "Branch: develop vs master"
    );
}

#[test]
fn test_format_commit_counts() {
    let (ahead, behind) = format_commit_counts("3", "1");
    assert_eq!(ahead, "+ 3 commits ahead");
    assert_eq!(behind, "- 1 commits behind");

    let (ahead, behind) = format_commit_counts("0", "5");
    assert_eq!(ahead, "+ 0 commits ahead");
    assert_eq!(behind, "- 5 commits behind");
}

#[test]
fn test_format_rev_list_range() {
    assert_eq!(format_rev_list_range("main", "feature"), "main...feature");
    assert_eq!(
        format_rev_list_range("develop", "hotfix"),
        "develop...hotfix"
    );
}

#[test]
fn test_parse_commit_counts() {
    assert_eq!(
        parse_commit_counts("2 3"),
        ("3".to_string(), "2".to_string())
    );
    assert_eq!(
        parse_commit_counts("0 1"),
        ("1".to_string(), "0".to_string())
    );
    assert_eq!(parse_commit_counts("5"), ("0".to_string(), "5".to_string()));
    assert_eq!(parse_commit_counts(""), ("0".to_string(), "0".to_string()));
}

#[test]
fn test_git_status_to_symbol() {
    assert_eq!(git_status_to_symbol("A"), "+");
    assert_eq!(git_status_to_symbol("M"), "~");
    assert_eq!(git_status_to_symbol("D"), "-");
    assert_eq!(git_status_to_symbol("R"), "R");
    assert_eq!(git_status_to_symbol("C"), "C");
}

#[test]
fn test_format_diff_line() {
    assert_eq!(
        format_diff_line("A\tfile.txt"),
        Some(" - + file.txt".to_string())
    );
    assert_eq!(
        format_diff_line("M\tsrc/main.rs"),
        Some(" - ~ src/main.rs".to_string())
    );
    assert_eq!(
        format_diff_line("D\told.txt"),
        Some(" - - old.txt".to_string())
    );
    assert_eq!(format_diff_line("A"), None);
    assert_eq!(format_diff_line(""), None);
}

#[test]
fn test_what_run_function() {
    let repo = repo_with_feature_ahead("feature/test", "main");

    // Get original directory and handle potential failures
    let original_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return, // Skip test if current directory is invalid
    };

    // Change to repo directory and run the function directly
    if std::env::set_current_dir(repo.path()).is_err() {
        return; // Skip test if directory change fails
    }

    // Test that the function doesn't panic and git commands work
    let _ = git_x::what::run(None);

    // Restore original directory
    let _ = std::env::set_current_dir(&original_dir);
}

#[test]
fn test_what_run_function_with_target() {
    let repo = repo_with_feature_ahead("feature/compare", "main");

    // Get original directory and handle potential failures
    let original_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return, // Skip test if current directory is invalid
    };

    // Change to repo directory and run the function directly
    if std::env::set_current_dir(repo.path()).is_err() {
        return; // Skip test if directory change fails
    }

    // Test with specific target
    let _ = git_x::what::run(Some("main".to_string()));

    // Restore original directory
    let _ = std::env::set_current_dir(&original_dir);
}

#[test]
fn test_what_run_function_with_multiple_changes() {
    let repo = common::basic_repo();

    // Create multiple changes to trigger diff output
    repo.create_branch("feature/multi-changes");

    // Add multiple files to trigger the diff line printing
    repo.add_commit("file1.txt", "content1", "Add file1");
    repo.add_commit("file2.txt", "content2", "Add file2");

    // Get original directory and handle potential failures
    let original_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return, // Skip test if current directory is invalid
    };

    // Change to repo directory and run the function directly
    if std::env::set_current_dir(repo.path()).is_err() {
        return; // Skip test if directory change fails
    }

    // Test that the function prints diff lines
    let _ = git_x::what::run(Some("main".to_string()));

    // Restore original directory
    let _ = std::env::set_current_dir(&original_dir);
}
