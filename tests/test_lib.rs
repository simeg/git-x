use git_x::{GitXError, Result};
use std::io;

#[test]
fn test_gitx_error_git_command() {
    let error = GitXError::GitCommand("git status failed".to_string());
    assert_eq!(format!("{error}"), "Git command failed: git status failed");
}

#[test]
fn test_gitx_error_io() {
    let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
    let error = GitXError::Io(io_error);
    assert_eq!(format!("{error}"), "IO error: File not found");
}

#[test]
fn test_gitx_error_parse() {
    let error = GitXError::Parse("Invalid format".to_string());
    assert_eq!(format!("{error}"), "Parse error: Invalid format");
}

#[test]
fn test_gitx_error_debug() {
    let error = GitXError::GitCommand("test".to_string());
    let debug_str = format!("{error:?}");
    assert!(debug_str.contains("GitCommand"));
    assert!(debug_str.contains("test"));
}

#[test]
fn test_gitx_error_is_error() {
    let error = GitXError::Parse("test error".to_string());
    let _: &dyn std::error::Error = &error;
}

#[test]
fn test_from_io_error() {
    let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "Access denied");
    let gitx_error: GitXError = io_error.into();

    match gitx_error {
        GitXError::Io(err) => {
            assert_eq!(err.kind(), io::ErrorKind::PermissionDenied);
            assert_eq!(format!("{err}"), "Access denied");
        }
        _ => panic!("Expected GitXError::Io"),
    }
}

#[test]
fn test_result_type() {
    let success: i32 = 42;
    assert_eq!(success, 42);

    let failure: Result<i32> = Err(GitXError::Parse("test".to_string()));
    assert!(failure.is_err());
}

#[test]
fn test_error_chain() {
    let io_error = io::Error::other("Original error");
    let gitx_error: GitXError = io_error.into();

    let error_string = format!("{gitx_error}");
    assert!(error_string.contains("IO error"));
    assert!(error_string.contains("Original error"));
}

#[test]
fn test_gitx_error_source() {
    let io_error = io::Error::new(io::ErrorKind::InvalidInput, "Bad input");
    let gitx_error = GitXError::Io(io_error);

    // Test that it implements Error trait properly
    use std::error::Error;
    let error_trait: &dyn Error = &gitx_error;
    assert!(error_trait.source().is_some());
}
