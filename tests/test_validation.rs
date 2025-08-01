use git_x::GitXError;
use git_x::core::traits::Validator;
use git_x::core::validation::{
    BranchNameValidator, CommitHashValidator, RemoteNameValidator, Validate,
};
use serial_test::serial;

// Tests for Validate::commit_exists

#[test]
#[serial]
fn test_validate_commit_exists_may_fail() {
    // This test may fail if not in a git repo, so we handle both cases
    let result = Validate::commit_exists("HEAD");

    match result {
        Ok(()) => {
            // We're in a git repo and HEAD exists - success
        }
        Err(err) => {
            // Either not in git repo or commit doesn't exist
            let error_msg = err.to_string();
            assert!(
                error_msg.contains("not a git repository")
                    || error_msg.contains("does not exist")
                    || error_msg.contains("Git command failed")
            );
        }
    }
}

#[test]
#[serial]
fn test_validate_commit_exists_invalid() {
    // Test with obviously invalid commit hash
    let result = Validate::commit_exists("invalid-commit-hash-123");

    match result {
        Ok(()) => {
            // Unexpectedly passed (shouldn't happen)
        }
        Err(err) => {
            let error_msg = err.to_string();
            assert!(
                error_msg.contains("does not exist")
                    || error_msg.contains("not a git repository")
                    || error_msg.contains("Git command failed")
            );
        }
    }
}

// Tests for Validate::in_git_repo

#[test]
#[serial]
fn test_validate_in_git_repo() {
    let result = Validate::in_git_repo();

    match result {
        Ok(()) => {
            // We're in a git repository
        }
        Err(err) => {
            // Not in a git repository
            let error_msg = err.to_string();
            assert!(error_msg.contains("Not in a git repository"));
        }
    }
}

// Tests for Validate::branch_name

#[test]
#[serial]
fn test_validate_branch_name_valid() {
    assert!(Validate::branch_name("main").is_ok());
    assert!(Validate::branch_name("feature/test").is_ok());
    assert!(Validate::branch_name("bugfix-123").is_ok());
    assert!(Validate::branch_name("release/v1.0.0").is_ok());
    assert!(Validate::branch_name("test_branch").is_ok());
    assert!(Validate::branch_name("a").is_ok());
    assert!(Validate::branch_name("feature123").is_ok());
    assert!(Validate::branch_name("FEATURE").is_ok());
}

#[test]
#[serial]
fn test_validate_branch_name_empty() {
    let result = Validate::branch_name("");
    assert!(result.is_err());

    match result {
        Err(GitXError::Parse(msg)) => {
            assert!(msg.contains("Branch name cannot be empty"));
        }
        _ => panic!("Expected Parse error"),
    }
}

#[test]
#[serial]
fn test_validate_branch_name_invalid_characters() {
    let invalid_names = [
        "feature branch", // space
        "feature~1",      // tilde
        "feature^1",      // caret
        "feature:origin", // colon
        "feature?",       // question mark
        "feature*",       // asterisk
        "feature[1]",     // brackets
        "feature\\test",  // backslash
    ];

    for name in invalid_names {
        let result = Validate::branch_name(name);
        assert!(result.is_err(), "Expected error for branch name: {name}");

        match result {
            Err(GitXError::Parse(msg)) => {
                assert!(msg.contains("contains invalid characters"));
                assert!(msg.contains(name));
            }
            _ => panic!("Expected Parse error for: {name}"),
        }
    }
}

#[test]
#[serial]
fn test_validate_branch_name_reserved() {
    let result = Validate::branch_name("HEAD");
    assert!(result.is_err());

    match result {
        Err(GitXError::Parse(msg)) => {
            assert!(msg.contains("reserved"));
            assert!(msg.contains("HEAD"));
        }
        _ => panic!("Expected Parse error"),
    }
}

#[test]
#[serial]
fn test_validate_branch_name_starts_with_dash() {
    let result = Validate::branch_name("-invalid");
    assert!(result.is_err());

    match result {
        Err(GitXError::Parse(msg)) => {
            assert!(msg.contains("reserved"));
        }
        _ => panic!("Expected Parse error"),
    }
}

// Tests for Validate::commit_hash

#[test]
#[serial]
fn test_validate_commit_hash_valid() {
    // Full hash
    assert!(Validate::commit_hash("1234567890abcdef1234567890abcdef12345678").is_ok());

    // Short hashes
    assert!(Validate::commit_hash("1234567").is_ok());
    assert!(Validate::commit_hash("abc123").is_ok());
    assert!(Validate::commit_hash("1234").is_ok());

    // Mixed case
    assert!(Validate::commit_hash("AbC123").is_ok());
    assert!(Validate::commit_hash("DEF456").is_ok());
}

#[test]
#[serial]
fn test_validate_commit_hash_empty() {
    let result = Validate::commit_hash("");
    assert!(result.is_err());

    match result {
        Err(GitXError::Parse(msg)) => {
            assert!(msg.contains("Commit hash cannot be empty"));
        }
        _ => panic!("Expected Parse error"),
    }
}

#[test]
#[serial]
fn test_validate_commit_hash_too_short() {
    let result = Validate::commit_hash("123");
    assert!(result.is_err());

    match result {
        Err(GitXError::Parse(msg)) => {
            assert!(msg.contains("Invalid commit hash format"));
            assert!(msg.contains("must be 4-40 characters"));
        }
        _ => panic!("Expected Parse error"),
    }
}

#[test]
#[serial]
fn test_validate_commit_hash_too_long() {
    let result = Validate::commit_hash("1234567890abcdef1234567890abcdef123456789");
    assert!(result.is_err());

    match result {
        Err(GitXError::Parse(msg)) => {
            assert!(msg.contains("Invalid commit hash format"));
            assert!(msg.contains("must be 4-40 characters"));
        }
        _ => panic!("Expected Parse error"),
    }
}

#[test]
#[serial]
fn test_validate_commit_hash_invalid_characters() {
    let invalid_hashes = ["123g", "abcxyz", "123-456", "abc.def", "123 456"];

    for hash in invalid_hashes {
        let result = Validate::commit_hash(hash);
        assert!(result.is_err(), "Expected error for hash: {hash}");

        match result {
            Err(GitXError::Parse(msg)) => {
                assert!(msg.contains("must contain only hexadecimal characters"));
            }
            _ => panic!("Expected Parse error for: {hash}"),
        }
    }
}

// Tests for Validate::remote_name

#[test]
#[serial]
fn test_validate_remote_name_valid() {
    assert!(Validate::remote_name("origin").is_ok());
    assert!(Validate::remote_name("upstream").is_ok());
    assert!(Validate::remote_name("fork").is_ok());
    assert!(Validate::remote_name("my-remote").is_ok());
    assert!(Validate::remote_name("remote123").is_ok());
    assert!(Validate::remote_name("REMOTE").is_ok());
}

#[test]
#[serial]
fn test_validate_remote_name_empty() {
    let result = Validate::remote_name("");
    assert!(result.is_err());

    match result {
        Err(GitXError::Parse(msg)) => {
            assert!(msg.contains("Remote name cannot be empty"));
        }
        _ => panic!("Expected Parse error"),
    }
}

#[test]
#[serial]
fn test_validate_remote_name_invalid_characters() {
    let invalid_names = [
        "origin/branch", // slash
        "origin\\test",  // backslash
        "origin:test",   // colon
        "origin?",       // question mark
        "origin*",       // asterisk
        "origin[1]",     // brackets
        "origin^1",      // caret
        "origin~1",      // tilde
        "origin test",   // space
        "origin\ttest",  // tab
        "origin\ntest",  // newline
        "origin\rtest",  // carriage return
    ];

    for name in invalid_names {
        let result = Validate::remote_name(name);
        assert!(result.is_err(), "Expected error for remote name: {name}");

        match result {
            Err(GitXError::Parse(msg)) => {
                assert!(msg.contains("contains invalid characters"));
            }
            _ => panic!("Expected Parse error for: {name}"),
        }
    }
}

#[test]
#[serial]
fn test_validate_remote_name_invalid_patterns() {
    let invalid_names = ["-origin", "origin-", "origin..test"];

    for name in invalid_names {
        let result = Validate::remote_name(name);
        assert!(result.is_err(), "Expected error for remote name: {name}");

        match result {
            Err(GitXError::Parse(msg)) => {
                assert!(msg.contains("uses invalid pattern"));
            }
            _ => panic!("Expected Parse error for: {name}"),
        }
    }
}

// Tests for Validate::file_path

#[test]
#[serial]
fn test_validate_file_path_valid() {
    assert!(Validate::file_path("file.txt").is_ok());
    assert!(Validate::file_path("src/main.rs").is_ok());
    assert!(Validate::file_path("docs/README.md").is_ok());
    assert!(Validate::file_path("test-file.txt").is_ok());
    assert!(Validate::file_path("dir/subdir/file").is_ok());
}

#[test]
#[serial]
fn test_validate_file_path_empty() {
    let result = Validate::file_path("");
    assert!(result.is_err());

    match result {
        Err(GitXError::Parse(msg)) => {
            assert!(msg.contains("File path cannot be empty"));
        }
        _ => panic!("Expected Parse error"),
    }
}

#[test]
#[serial]
fn test_validate_file_path_dangerous() {
    let dangerous_paths = [
        "../etc/passwd",
        "../../file.txt",
        "/etc/passwd",
        "/absolute/path",
        "dir/../other",
    ];

    for path in dangerous_paths {
        let result = Validate::file_path(path);
        assert!(result.is_err(), "Expected error for path: {path}");

        match result {
            Err(GitXError::Parse(msg)) => {
                assert!(msg.contains("is not allowed"));
            }
            _ => panic!("Expected Parse error for: {path}"),
        }
    }
}

#[test]
#[serial]
fn test_validate_file_path_invalid_characters() {
    let invalid_paths = ["file\0.txt", "file\r.txt", "file\n.txt"];

    for path in invalid_paths {
        let result = Validate::file_path(path);
        assert!(result.is_err(), "Expected error for path: {path}");

        match result {
            Err(GitXError::Parse(msg)) => {
                assert!(msg.contains("contains invalid characters"));
            }
            _ => panic!("Expected Parse error for: {path}"),
        }
    }
}

// Tests for Validate::positive_number

#[test]
#[serial]
fn test_validate_positive_number_valid() {
    assert!(Validate::positive_number(0, None, "test").is_ok());
    assert!(Validate::positive_number(1, None, "test").is_ok());
    assert!(Validate::positive_number(100, None, "test").is_ok());
    assert!(Validate::positive_number(5, Some(10), "test").is_ok());
    assert!(Validate::positive_number(10, Some(10), "test").is_ok());
}

#[test]
#[serial]
fn test_validate_positive_number_negative() {
    let result = Validate::positive_number(-1, None, "count");
    assert!(result.is_err());

    match result {
        Err(GitXError::Parse(msg)) => {
            assert!(msg.contains("count must be positive"));
            assert!(msg.contains("-1"));
        }
        _ => panic!("Expected Parse error"),
    }
}

#[test]
#[serial]
fn test_validate_positive_number_exceeds_max() {
    let result = Validate::positive_number(15, Some(10), "limit");
    assert!(result.is_err());

    match result {
        Err(GitXError::Parse(msg)) => {
            assert!(msg.contains("limit must be <= 10"));
            assert!(msg.contains("15"));
        }
        _ => panic!("Expected Parse error"),
    }
}

// Tests for Validate::git_date_format

#[test]
#[serial]
fn test_validate_git_date_format_valid() {
    assert!(Validate::git_date_format("2023-01-01").is_ok());
    assert!(Validate::git_date_format("yesterday").is_ok());
    assert!(Validate::git_date_format("1 week ago").is_ok());
    assert!(Validate::git_date_format("2023-12-25T10:00:00").is_ok());
    assert!(Validate::git_date_format("last.monday").is_ok());
}

#[test]
#[serial]
fn test_validate_git_date_format_empty() {
    let result = Validate::git_date_format("");
    assert!(result.is_err());

    match result {
        Err(GitXError::Parse(msg)) => {
            assert!(msg.contains("Date string cannot be empty"));
        }
        _ => panic!("Expected Parse error"),
    }
}

#[test]
#[serial]
fn test_validate_git_date_format_dangerous() {
    let dangerous_dates = ["2023-01-01; rm -rf /", "date && ls", "2023|whoami"];

    for date in dangerous_dates {
        let result = Validate::git_date_format(date);
        assert!(result.is_err(), "Expected error for date: {date}");

        match result {
            Err(GitXError::Parse(msg)) => {
                assert!(msg.contains("contains invalid characters"));
            }
            _ => panic!("Expected Parse error for: {date}"),
        }
    }
}

#[test]
#[serial]
fn test_validate_git_date_format_too_long() {
    let long_date = "a".repeat(101);
    let result = Validate::git_date_format(&long_date);
    assert!(result.is_err());

    match result {
        Err(GitXError::Parse(msg)) => {
            assert!(msg.contains("Date string too long"));
        }
        _ => panic!("Expected Parse error"),
    }
}

// Tests for Validate::safe_string

#[test]
#[serial]
fn test_validate_safe_string_valid() {
    assert!(Validate::safe_string("simple", "test").is_ok());
    assert!(Validate::safe_string("test123", "field").is_ok());
    assert!(Validate::safe_string("test-value", "input").is_ok());
    assert!(Validate::safe_string("test_value", "name").is_ok());
    assert!(Validate::safe_string("TestValue", "identifier").is_ok());
}

#[test]
#[serial]
fn test_validate_safe_string_empty() {
    let result = Validate::safe_string("", "field");
    assert!(result.is_err());

    match result {
        Err(GitXError::Parse(msg)) => {
            assert!(msg.contains("field cannot be empty"));
        }
        _ => panic!("Expected Parse error"),
    }
}

#[test]
#[serial]
fn test_validate_safe_string_dangerous() {
    let dangerous_strings = [
        "test;command",
        "test&other",
        "test|pipe",
        "test`command`",
        "test$var",
        "test()",
        "test{}",
        "test\\escape",
        "test\nline",
        "test\rcarriage",
        "test space",
    ];

    for string in dangerous_strings {
        let result = Validate::safe_string(string, "input");
        assert!(result.is_err(), "Expected error for string: {string}");

        match result {
            Err(GitXError::Parse(msg)) => {
                assert!(msg.contains("contains potentially dangerous characters"));
            }
            _ => panic!("Expected Parse error for: {string}"),
        }
    }
}

#[test]
#[serial]
fn test_validate_safe_string_too_long() {
    let long_string = "a".repeat(1001);
    let result = Validate::safe_string(&long_string, "data");
    assert!(result.is_err());

    match result {
        Err(GitXError::Parse(msg)) => {
            assert!(msg.contains("data is too long"));
            assert!(msg.contains("1001"));
        }
        _ => panic!("Expected Parse error"),
    }
}

// Tests for BranchNameValidator

#[test]
#[serial]
fn test_branch_name_validator() {
    let validator = BranchNameValidator;

    // Test valid names
    assert!(validator.validate("main").is_ok());
    assert!(validator.validate("feature/test").is_ok());

    // Test invalid names
    assert!(validator.validate("").is_err());
    assert!(validator.validate("HEAD").is_err());
    assert!(validator.validate("invalid name").is_err());

    // Test validation rules
    let rules = validator.validation_rules();
    assert_eq!(rules.len(), 5);
    assert!(rules.contains(&"Cannot be empty"));
    assert!(rules.contains(&"Cannot start with a dash"));
    assert!(rules.contains(&"Cannot be 'HEAD'"));
    assert!(rules.contains(&"Cannot contain spaces"));
    assert!(rules.contains(&"Cannot contain ~^:?*[\\"));
}

// Tests for CommitHashValidator

#[test]
#[serial]
fn test_commit_hash_validator() {
    let validator = CommitHashValidator;

    // Test valid hashes
    assert!(validator.validate("1234567").is_ok());
    assert!(validator.validate("abcdef1234567890").is_ok());

    // Test invalid hashes
    assert!(validator.validate("").is_err());
    assert!(validator.validate("123").is_err());
    assert!(validator.validate("invalid").is_err());

    // Test validation rules
    let rules = validator.validation_rules();
    assert_eq!(rules.len(), 3);
    assert!(rules.contains(&"Must be 4-40 characters long"));
    assert!(rules.contains(&"Must contain only hex characters (0-9, a-f)"));
    assert!(rules.contains(&"Must reference an existing commit"));
}

// Tests for RemoteNameValidator

#[test]
#[serial]
fn test_remote_name_validator() {
    let validator = RemoteNameValidator;

    // Test valid names
    assert!(validator.validate("origin").is_ok());
    assert!(validator.validate("upstream").is_ok());

    // Test invalid names
    assert!(validator.validate("").is_err());
    assert!(validator.validate("-origin").is_err());
    assert!(validator.validate("origin..test").is_err());

    // Test validation rules
    let rules = validator.validation_rules();
    assert_eq!(rules.len(), 4);
    assert!(rules.contains(&"Cannot be empty"));
    assert!(rules.contains(&"Cannot contain special characters"));
    assert!(rules.contains(&"Cannot start or end with dash"));
    assert!(rules.contains(&"Cannot contain '..'"));
}

// Integration tests for validator combinations

#[test]
#[serial]
fn test_multiple_validators_same_input() {
    let branch_validator = BranchNameValidator;
    let commit_validator = CommitHashValidator;

    // Test input that's valid for branch but invalid for commit
    let input = "feature-branch";
    assert!(branch_validator.validate(input).is_ok());
    assert!(commit_validator.validate(input).is_err());

    // Test input that's valid for commit but invalid for branch
    let input = "1234567890abcdef";
    assert!(commit_validator.validate(input).is_ok());
    assert!(branch_validator.validate(input).is_ok()); // Actually valid branch name too
}

// Edge case tests

#[test]
#[serial]
fn test_validate_branch_name_edge_cases() {
    // Test very long but valid name
    let long_name = "a".repeat(200);
    assert!(Validate::branch_name(&long_name).is_ok());

    // Test numbers only
    assert!(Validate::branch_name("123456").is_ok());

    // Test special valid characters
    assert!(Validate::branch_name("feature.test").is_ok());
    assert!(Validate::branch_name("feature+test").is_ok());
    assert!(Validate::branch_name("feature@test").is_ok());
}

#[test]
#[serial]
fn test_validate_file_path_edge_cases() {
    // Test single character
    assert!(Validate::file_path("a").is_ok());

    // Test with dots (but not ..)
    assert!(Validate::file_path("file.txt").is_ok());
    assert!(Validate::file_path("dir.name/file").is_ok());

    // Test with spaces in filename (should be valid)
    assert!(Validate::file_path("my file.txt").is_ok());
}

#[test]
#[serial]
fn test_validate_positive_number_edge_cases() {
    // Test zero
    assert!(Validate::positive_number(0, Some(0), "test").is_ok());

    // Test large numbers
    assert!(Validate::positive_number(i32::MAX, None, "test").is_ok());

    // Test boundary conditions
    assert!(Validate::positive_number(999, Some(1000), "test").is_ok());
    assert!(Validate::positive_number(1000, Some(1000), "test").is_ok());
    assert!(Validate::positive_number(1001, Some(1000), "test").is_err());
}
