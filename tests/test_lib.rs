use git_x::{GitXError, Result};
use serial_test::serial;
use std::io;

#[test]
#[serial]
fn test_gitx_error_git_command() {
    let error = GitXError::GitCommand("git status failed".to_string());
    assert_eq!(format!("{error}"), "Git command failed: git status failed");
}

#[test]
#[serial]
fn test_gitx_error_io() {
    let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
    let error = GitXError::Io(io_error);
    assert_eq!(format!("{error}"), "IO error: File not found");
}

#[test]
#[serial]
fn test_gitx_error_parse() {
    let error = GitXError::Parse("Invalid format".to_string());
    assert_eq!(format!("{error}"), "Parse error: Invalid format");
}

#[test]
#[serial]
fn test_gitx_error_debug() {
    let error = GitXError::GitCommand("test".to_string());
    let debug_str = format!("{error:?}");
    assert!(debug_str.contains("GitCommand"));
    assert!(debug_str.contains("test"));
}

#[test]
#[serial]
fn test_gitx_error_is_error() {
    let error = GitXError::Parse("test error".to_string());
    let _: &dyn std::error::Error = &error;
}

#[test]
#[serial]
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
#[serial]
fn test_result_type() {
    let success: i32 = 42;
    assert_eq!(success, 42);

    let failure: Result<i32> = Err(GitXError::Parse("test".to_string()));
    assert!(failure.is_err());
}

#[test]
#[serial]
fn test_error_chain() {
    let io_error = io::Error::other("Original error");
    let gitx_error: GitXError = io_error.into();

    let error_string = format!("{gitx_error}");
    assert!(error_string.contains("IO error"));
    assert!(error_string.contains("Original error"));
}

#[test]
#[serial]
fn test_gitx_error_source() {
    let io_error = io::Error::new(io::ErrorKind::InvalidInput, "Bad input");
    let gitx_error = GitXError::Io(io_error);

    // Test that it implements Error trait properly
    use std::error::Error;
    let error_trait: &dyn Error = &gitx_error;
    assert!(error_trait.source().is_some());
}

#[test]
#[serial]
fn test_gitx_error_source_variants() {
    use std::error::Error;

    // Test IO error source (should return Some)
    let io_error = io::Error::other("test");
    let gitx_io_error = GitXError::Io(io_error);
    assert!(gitx_io_error.source().is_some());

    // Test GitCommand error source (should return None)
    let git_error = GitXError::GitCommand("test".to_string());
    assert!(git_error.source().is_none());

    // Test Parse error source (should return None)
    let parse_error = GitXError::Parse("test".to_string());
    assert!(parse_error.source().is_none());
}

#[test]
#[serial]
fn test_gitx_error_debug_all_variants() {
    // Test Debug trait for all error variants
    let git_error = GitXError::GitCommand("git failed".to_string());
    let git_debug = format!("{git_error:?}");
    assert!(git_debug.contains("GitCommand"));
    assert!(git_debug.contains("git failed"));

    let io_error = GitXError::Io(io::Error::other("io failed"));
    let io_debug = format!("{io_error:?}");
    assert!(io_debug.contains("Io"));

    let parse_error = GitXError::Parse("parse failed".to_string());
    let parse_debug = format!("{parse_error:?}");
    assert!(parse_debug.contains("Parse"));
    assert!(parse_debug.contains("parse failed"));
}

#[test]
#[serial]
fn test_result_type_alias() {
    // Test the Result type alias works correctly
    fn test_function() -> Result<String> {
        Ok("success".to_string())
    }

    fn test_function_error() -> Result<String> {
        Err(GitXError::Parse("test error".to_string()))
    }

    assert!(test_function().is_ok());
    assert!(test_function_error().is_err());

    let success_result = test_function().unwrap();
    assert_eq!(success_result, "success");
}
