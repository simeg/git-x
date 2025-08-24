use crate::core::git::GitOperations;
use crate::{GitXError, Result};

/// Common validation utilities
pub struct Validate;

impl Validate {
    /// Validate that a commit exists
    pub fn commit_exists(commit: &str) -> Result<()> {
        if GitOperations::commit_exists(commit)? {
            Ok(())
        } else {
            Err(GitXError::GitCommand(format!(
                "Commit '{commit}' does not exist"
            )))
        }
    }

    /// Validate that we're in a git repository
    pub fn in_git_repo() -> Result<()> {
        match GitOperations::repo_root() {
            Ok(_) => Ok(()),
            Err(_) => Err(GitXError::GitCommand("Not in a git repository".to_string())),
        }
    }

    /// Validate branch name format
    pub fn branch_name(name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(GitXError::Parse("Branch name cannot be empty".to_string()));
        }

        // Check for invalid characters
        let invalid_chars = [' ', '~', '^', ':', '?', '*', '[', '\\'];
        if name.chars().any(|c| invalid_chars.contains(&c)) {
            return Err(GitXError::Parse(format!(
                "Branch name '{name}' contains invalid characters"
            )));
        }

        // Check for reserved names
        if name == "HEAD" || name.starts_with('-') {
            return Err(GitXError::Parse(format!(
                "Branch name '{name}' is reserved"
            )));
        }

        Ok(())
    }

    /// Validate commit hash format (40 chars hex or 7+ chars for short hash)
    pub fn commit_hash(hash: &str) -> Result<()> {
        if hash.is_empty() {
            return Err(GitXError::Parse("Commit hash cannot be empty".to_string()));
        }

        // Check length - must be at least 4 chars for short hash, max 40 for full
        if hash.len() < 4 || hash.len() > 40 {
            return Err(GitXError::Parse(format!(
                "Invalid commit hash format: '{hash}' (must be 4-40 characters)"
            )));
        }

        // Check if all characters are hexadecimal
        if !hash.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(GitXError::Parse(format!(
                "Invalid commit hash format: '{hash}' (must contain only hexadecimal characters)"
            )));
        }

        Ok(())
    }

    /// Validate remote name format
    pub fn remote_name(name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(GitXError::Parse("Remote name cannot be empty".to_string()));
        }

        // Check for invalid characters
        let invalid_chars = [
            ' ', '\t', '\n', '\r', '/', '\\', ':', '?', '*', '[', ']', '^', '~',
        ];
        if name.chars().any(|c| invalid_chars.contains(&c)) {
            return Err(GitXError::Parse(format!(
                "Remote name '{name}' contains invalid characters"
            )));
        }

        // Check for reserved names and patterns
        if name.starts_with('-') || name.ends_with('-') || name.contains("..") {
            return Err(GitXError::Parse(format!(
                "Remote name '{name}' uses invalid pattern"
            )));
        }

        Ok(())
    }

    /// Validate file path for git operations
    pub fn file_path(path: &str) -> Result<()> {
        if path.is_empty() {
            return Err(GitXError::Parse("File path cannot be empty".to_string()));
        }

        // Check for dangerous paths
        if path.contains("..") || path.starts_with('/') {
            return Err(GitXError::Parse(format!(
                "File path '{path}' is not allowed (no .. or absolute paths)"
            )));
        }

        // Check for null bytes or other problematic characters
        if path.contains('\0') || path.contains('\r') || path.contains('\n') {
            return Err(GitXError::Parse(format!(
                "File path '{path}' contains invalid characters"
            )));
        }

        Ok(())
    }

    /// Validate that a number is within a reasonable range
    pub fn positive_number(value: i32, max: Option<i32>, field_name: &str) -> Result<()> {
        if value < 0 {
            return Err(GitXError::Parse(format!(
                "{field_name} must be positive, got {value}"
            )));
        }

        if let Some(max_val) = max
            && value > max_val
        {
            return Err(GitXError::Parse(format!(
                "{field_name} must be <= {max_val}, got {value}"
            )));
        }

        Ok(())
    }

    /// Validate date/time string format for git operations
    pub fn git_date_format(date_str: &str) -> Result<()> {
        if date_str.is_empty() {
            return Err(GitXError::Parse("Date string cannot be empty".to_string()));
        }

        // Check for obviously malicious input
        if date_str.contains(';') || date_str.contains('&') || date_str.contains('|') {
            return Err(GitXError::Parse(format!(
                "Date string '{date_str}' contains invalid characters"
            )));
        }

        // Basic length check to prevent excessively long input
        if date_str.len() > 100 {
            return Err(GitXError::Parse(format!(
                "Date string too long: {}",
                date_str.len()
            )));
        }

        Ok(())
    }

    /// Validate string doesn't contain shell injection patterns
    pub fn safe_string(input: &str, field_name: &str) -> Result<()> {
        if input.is_empty() {
            return Err(GitXError::Parse(format!("{field_name} cannot be empty")));
        }

        // Check for shell injection patterns
        let dangerous_patterns = [
            ";", "&", "|", "`", "$", "(", ")", "{", "}", "\\", "\n", "\r", " ",
        ];
        if dangerous_patterns
            .iter()
            .any(|&pattern| input.contains(pattern))
        {
            return Err(GitXError::Parse(format!(
                "{field_name} contains potentially dangerous characters"
            )));
        }

        // Basic length check
        if input.len() > 1000 {
            return Err(GitXError::Parse(format!(
                "{field_name} is too long: {} characters",
                input.len()
            )));
        }

        Ok(())
    }
}

/// Specific validators for different types of input
pub struct BranchNameValidator;

impl crate::core::traits::Validator<str> for BranchNameValidator {
    fn validate(&self, input: &str) -> Result<()> {
        Validate::branch_name(input)
    }

    fn validation_rules(&self) -> Vec<&'static str> {
        vec![
            "Cannot be empty",
            "Cannot start with a dash",
            "Cannot be 'HEAD'",
            "Cannot contain spaces",
            "Cannot contain ~^:?*[\\",
        ]
    }
}

pub struct CommitHashValidator;

impl crate::core::traits::Validator<str> for CommitHashValidator {
    fn validate(&self, input: &str) -> Result<()> {
        Validate::commit_hash(input)
    }

    fn validation_rules(&self) -> Vec<&'static str> {
        vec![
            "Must be 4-40 characters long",
            "Must contain only hex characters (0-9, a-f)",
            "Must reference an existing commit",
        ]
    }
}

pub struct RemoteNameValidator;

impl crate::core::traits::Validator<str> for RemoteNameValidator {
    fn validate(&self, input: &str) -> Result<()> {
        Validate::remote_name(input)
    }

    fn validation_rules(&self) -> Vec<&'static str> {
        vec![
            "Cannot be empty",
            "Cannot contain special characters",
            "Cannot start or end with dash",
            "Cannot contain '..'",
        ]
    }
}
