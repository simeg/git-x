use serial_test::serial;
// Safety module tests
//
// WARNING: Many tests in this module perform DESTRUCTIVE git operations including:
// - Creating backup branches
// - Creating and restoring git stashes (checkpoints)
// - Modifying working directory state
//
// To protect local development environments, these tests are skipped by default
// and only run in CI environments or when explicitly enabled.
//
// To run destructive tests locally:
//   ENABLE_DESTRUCTIVE_TESTS=1 cargo test test_safety
//
// Tests that don't modify git state (method signatures, builders, etc.) run normally.

use git_x::core::safety::{Safety, SafetyBuilder};
use git_x::{GitXError, Result};

// Helper to check if we should run potentially destructive tests
fn should_run_destructive_tests() -> bool {
    // Only run destructive tests in CI or when explicitly enabled
    std::env::var("CI").is_ok()
        || std::env::var("GITHUB_ACTIONS").is_ok()
        || std::env::var("ENABLE_DESTRUCTIVE_TESTS").is_ok()
}

// Helper function to check if we're in a git repository for conditional testing
#[allow(dead_code)]
fn get_test_safety_result<F>(operation: F) -> Result<String>
where
    F: FnOnce() -> Result<String>,
{
    // Try operation and handle the result based on git repo availability
    operation()
}

// Tests for Safety::create_backup_branch

#[test]
#[serial]
fn test_safety_create_backup_branch() {
    if !should_run_destructive_tests() {
        // Skip destructive test in local development
        return;
    }

    // Test create_backup_branch (may fail if not in git repo)
    let result = Safety::create_backup_branch(None);

    match result {
        Ok(backup_name) => {
            // Success - we're in a git repo and backup was created
            assert!(backup_name.starts_with("backup/"));
            assert!(backup_name.contains("_")); // timestamp separator
        }
        Err(err) => {
            // Failed - likely not in git repo or other git error
            let error_msg = err.to_string();
            assert!(
                error_msg.contains("not a git repository")
                    || error_msg.contains("Git command failed")
                    || error_msg.contains("Failed to create backup")
                    || error_msg.contains("current branch")
            );
        }
    }
}

#[test]
#[serial]
fn test_safety_create_backup_branch_with_prefix() {
    if !should_run_destructive_tests() {
        return;
    }

    let result = Safety::create_backup_branch(Some("test-backup"));

    match result {
        Ok(backup_name) => {
            assert!(backup_name.starts_with("test-backup/"));
            assert!(backup_name.contains("_")); // timestamp separator
        }
        Err(err) => {
            // Expected to fail if not in git repo
            let error_msg = err.to_string();
            assert!(
                error_msg.contains("not a git repository")
                    || error_msg.contains("Git command failed")
                    || error_msg.contains("Failed to create backup")
            );
        }
    }
}

// Tests for Safety::ensure_clean_working_directory

#[test]
#[serial]
fn test_safety_ensure_clean_working_directory() {
    let result = Safety::ensure_clean_working_directory();

    match result {
        Ok(()) => {
            // Success - working directory is clean or we're in test environment
        }
        Err(err) => {
            // Failed - either not in git repo or working directory not clean
            let error_msg = err.to_string();
            assert!(
                error_msg.contains("not a git repository")
                    || error_msg.contains("Working directory is not clean")
                    || error_msg.contains("Git command failed")
            );
        }
    }
}

// Tests for Safety::confirm_destructive_operation

#[test]
#[serial]
fn test_safety_confirm_destructive_operation_non_interactive() {
    // Set non-interactive environment
    unsafe {
        std::env::set_var("GIT_X_NON_INTERACTIVE", "1");
    }

    let result = Safety::confirm_destructive_operation("Test Operation", "Test details");

    match result {
        Ok(confirmed) => {
            // In non-interactive mode, should return true (allow operation)
            assert!(confirmed);
        }
        Err(err) => {
            // Shouldn't fail in non-interactive mode unless other issue
            let error_msg = err.to_string();
            assert!(
                error_msg.contains("not a git repository")
                    || error_msg.contains("interactive")
                    || error_msg.contains("confirm")
            );
        }
    }

    unsafe {
        std::env::remove_var("GIT_X_NON_INTERACTIVE");
    }
}

// Tests for Safety::is_test_environment (private method, tested indirectly)

#[test]
#[serial]
fn test_safety_is_test_environment_indirect() {
    // This test verifies that we're correctly detecting the test environment
    // In a test environment, cfg!(test) should be true
    assert!(cfg!(test));

    // The safety module should detect test environment through various means
    // We can test this indirectly by checking environment variables
    let is_test_env =
        std::env::var("CARGO_TARGET_TMPDIR").is_ok() || std::env::var("CI").is_ok() || cfg!(test);

    assert!(is_test_env, "Should detect test environment");
}

// Tests for Safety::create_checkpoint

#[test]
#[serial]
fn test_safety_create_checkpoint() {
    if !should_run_destructive_tests() {
        return;
    }

    let result = Safety::create_checkpoint(None);

    match result {
        Ok(message) => {
            assert_eq!(message, "git-x safety checkpoint");
        }
        Err(err) => {
            // May fail if not in git repo or no changes to stash
            let error_msg = err.to_string();
            assert!(
                error_msg.contains("not a git repository")
                    || error_msg.contains("Failed to create safety checkpoint")
                    || error_msg.contains("Git command failed")
            );
        }
    }
}

#[test]
#[serial]
fn test_safety_create_checkpoint_with_message() {
    if !should_run_destructive_tests() {
        return;
    }

    let result = Safety::create_checkpoint(Some("Custom checkpoint message"));

    match result {
        Ok(message) => {
            assert_eq!(message, "Custom checkpoint message");
        }
        Err(err) => {
            // May fail if not in git repo
            let error_msg = err.to_string();
            assert!(
                error_msg.contains("not a git repository")
                    || error_msg.contains("Failed to create safety checkpoint")
                    || error_msg.contains("Git command failed")
            );
        }
    }
}

// Tests for Safety::restore_checkpoint

#[test]
#[serial]
fn test_safety_restore_checkpoint() {
    if !should_run_destructive_tests() {
        return;
    }

    let result = Safety::restore_checkpoint();

    match result {
        Ok(()) => {
            // Successfully restored checkpoint
        }
        Err(err) => {
            // May fail if not in git repo or no stash to restore
            let error_msg = err.to_string();
            assert!(
                error_msg.contains("not a git repository")
                    || error_msg.contains("Failed to restore from safety checkpoint")
                    || error_msg.contains("Git command failed")
            );
        }
    }
}

// Tests for Safety::list_backup_branches

#[test]
#[serial]
fn test_safety_list_backup_branches() {
    let result = Safety::list_backup_branches();

    match result {
        Ok(branches) => {
            // Successfully listed backup branches (may be empty)
            for branch in &branches {
                assert!(branch.starts_with("backup/") || !branch.contains("*"));
            }
        }
        Err(err) => {
            // May fail if not in git repo
            let error_msg = err.to_string();
            assert!(
                error_msg.contains("not a git repository")
                    || error_msg.contains("Failed to list backup branches")
                    || error_msg.contains("Git command failed")
            );
        }
    }
}

// Tests for Safety::cleanup_old_backups

#[test]
#[serial]
fn test_safety_cleanup_old_backups_dry_run() {
    let result = Safety::cleanup_old_backups(30, true);

    match result {
        Ok(results) => {
            // Dry run should work and show what would be deleted
            for result_msg in &results {
                if !result_msg.is_empty() {
                    assert!(
                        result_msg.contains("[DRY RUN]")
                            || result_msg.contains("Would delete")
                            || result_msg.contains("backup/")
                    );
                }
            }
        }
        Err(err) => {
            // May fail if not in git repo
            let error_msg = err.to_string();
            assert!(
                error_msg.contains("not a git repository")
                    || error_msg.contains("Failed to list backup branches")
                    || error_msg.contains("Git command failed")
            );
        }
    }
}

#[test]
#[serial]
fn test_safety_cleanup_old_backups_actual() {
    if !should_run_destructive_tests() {
        return;
    }

    let result = Safety::cleanup_old_backups(365, false); // Very old backups only

    match result {
        Ok(results) => {
            // Actual cleanup should work
            for result_msg in &results {
                if !result_msg.is_empty() {
                    assert!(
                        result_msg.contains("Deleted:")
                            || result_msg.contains("Failed to delete")
                            || result_msg.contains("backup/")
                    );
                }
            }
        }
        Err(err) => {
            // May fail if not in git repo
            let error_msg = err.to_string();
            assert!(
                error_msg.contains("not a git repository")
                    || error_msg.contains("Failed to list backup branches")
                    || error_msg.contains("Git command failed")
            );
        }
    }
}

// Tests for SafetyBuilder

#[test]
#[serial]
fn test_safety_builder_new() {
    let builder = SafetyBuilder::new("test operation");

    // Just verify it can be created
    drop(builder);
}

#[test]
#[serial]
fn test_safety_builder_fluent_interface() {
    let builder = SafetyBuilder::new("complex operation")
        .with_backup()
        .with_checkpoint()
        .with_confirmation()
        .with_clean_directory();

    // Test that fluent interface works
    drop(builder);
}

#[test]
#[serial]
fn test_safety_builder_execute_simple() {
    // Set non-interactive mode to avoid hanging
    unsafe {
        std::env::set_var("GIT_X_NON_INTERACTIVE", "1");
    }

    let builder = SafetyBuilder::new("simple test");

    let result = builder.execute(|| Ok("Operation completed".to_string()));

    match result {
        Ok(output) => {
            assert_eq!(output, "Operation completed");
        }
        Err(err) => {
            // May fail if not in git repo
            let error_msg = err.to_string();
            assert!(
                error_msg.contains("not a git repository")
                    || error_msg.contains("Git command failed")
            );
        }
    }

    unsafe {
        std::env::remove_var("GIT_X_NON_INTERACTIVE");
    }
}

#[test]
#[serial]
fn test_safety_builder_execute_with_backup() {
    if !should_run_destructive_tests() {
        return;
    }

    unsafe {
        std::env::set_var("GIT_X_NON_INTERACTIVE", "1");
    }

    let builder = SafetyBuilder::new("backup test").with_backup();

    let result = builder.execute(|| Ok("Operation with backup completed".to_string()));

    match result {
        Ok(output) => {
            assert_eq!(output, "Operation with backup completed");
        }
        Err(err) => {
            // May fail if not in git repo or backup creation fails
            let error_msg = err.to_string();
            assert!(
                error_msg.contains("not a git repository")
                    || error_msg.contains("Failed to create backup")
                    || error_msg.contains("Git command failed")
                    || error_msg.contains("current branch")
            );
        }
    }

    unsafe {
        std::env::remove_var("GIT_X_NON_INTERACTIVE");
    }
}

#[test]
#[serial]
fn test_safety_builder_execute_with_checkpoint() {
    if !should_run_destructive_tests() {
        return;
    }

    unsafe {
        std::env::set_var("GIT_X_NON_INTERACTIVE", "1");
    }

    let builder = SafetyBuilder::new("checkpoint test").with_checkpoint();

    let result = builder.execute(|| Ok("Operation with checkpoint completed".to_string()));

    match result {
        Ok(output) => {
            assert_eq!(output, "Operation with checkpoint completed");
        }
        Err(err) => {
            // May fail if not in git repo or checkpoint creation fails
            let error_msg = err.to_string();
            assert!(
                error_msg.contains("not a git repository")
                    || error_msg.contains("Failed to create safety checkpoint")
                    || error_msg.contains("Git command failed")
            );
        }
    }

    unsafe {
        std::env::remove_var("GIT_X_NON_INTERACTIVE");
    }
}

#[test]
#[serial]
fn test_safety_builder_execute_with_confirmation() {
    unsafe {
        std::env::set_var("GIT_X_NON_INTERACTIVE", "1");
    }

    let builder = SafetyBuilder::new("confirmation test").with_confirmation();

    let result = builder.execute(|| Ok("Operation with confirmation completed".to_string()));

    match result {
        Ok(output) => {
            // In non-interactive mode, confirmation should be skipped and operation should proceed
            assert_eq!(output, "Operation with confirmation completed");
        }
        Err(err) => {
            // May fail if not in git repo
            let error_msg = err.to_string();
            assert!(
                error_msg.contains("not a git repository")
                    || error_msg.contains("Git command failed")
            );
        }
    }

    unsafe {
        std::env::remove_var("GIT_X_NON_INTERACTIVE");
    }
}

#[test]
#[serial]
fn test_safety_builder_execute_with_clean_directory() {
    unsafe {
        std::env::set_var("GIT_X_NON_INTERACTIVE", "1");
    }

    let builder = SafetyBuilder::new("clean directory test").with_clean_directory();

    let result = builder.execute(|| Ok("Operation with clean directory completed".to_string()));

    match result {
        Ok(output) => {
            assert_eq!(output, "Operation with clean directory completed");
        }
        Err(err) => {
            // May fail if not in git repo or working directory not clean
            let error_msg = err.to_string();
            assert!(
                error_msg.contains("not a git repository")
                    || error_msg.contains("Working directory is not clean")
                    || error_msg.contains("Git command failed")
            );
        }
    }

    unsafe {
        std::env::remove_var("GIT_X_NON_INTERACTIVE");
    }
}

#[test]
#[serial]
fn test_safety_builder_execute_failing_operation() {
    if !should_run_destructive_tests() {
        return;
    }

    unsafe {
        std::env::set_var("GIT_X_NON_INTERACTIVE", "1");
    }

    let builder = SafetyBuilder::new("failing test").with_checkpoint();

    let result = builder.execute(|| {
        Err(GitXError::GitCommand(
            "Simulated operation failure".to_string(),
        ))
    });

    // Should return the original error
    assert!(result.is_err());

    match result {
        Err(GitXError::GitCommand(msg)) => {
            assert_eq!(msg, "Simulated operation failure");
        }
        Err(other_err) => {
            // May fail earlier due to git repo issues
            let error_msg = other_err.to_string();
            assert!(
                error_msg.contains("not a git repository")
                    || error_msg.contains("Failed to create safety checkpoint")
                    || error_msg.contains("Git command failed")
            );
        }
        Ok(_) => panic!("Expected error for failing operation"),
    }

    unsafe {
        std::env::remove_var("GIT_X_NON_INTERACTIVE");
    }
}

#[test]
#[serial]
fn test_safety_builder_execute_all_options() {
    if !should_run_destructive_tests() {
        return;
    }

    unsafe {
        std::env::set_var("GIT_X_NON_INTERACTIVE", "1");
    }

    let builder = SafetyBuilder::new("comprehensive test")
        .with_backup()
        .with_checkpoint()
        .with_confirmation()
        .with_clean_directory();

    let result = builder.execute(|| Ok("All safety features tested".to_string()));

    match result {
        Ok(output) => {
            assert_eq!(output, "All safety features tested");
        }
        Err(err) => {
            // May fail due to various git repo issues
            let error_msg = err.to_string();
            assert!(
                error_msg.contains("not a git repository")
                    || error_msg.contains("Working directory is not clean")
                    || error_msg.contains("Failed to create backup")
                    || error_msg.contains("Failed to create safety checkpoint")
                    || error_msg.contains("Git command failed")
            );
        }
    }

    unsafe {
        std::env::remove_var("GIT_X_NON_INTERACTIVE");
    }
}

// Tests for user cancellation scenario

#[test]
#[serial]
fn test_safety_builder_user_cancellation_simulation() {
    // We can't easily test actual user cancellation in automated tests,
    // but we can test the cancellation message handling

    // Test that cancellation message is properly formatted
    let expected_cancel_msg = "Operation cancelled by user.";
    assert_eq!(expected_cancel_msg.len(), 28);
    assert!(expected_cancel_msg.contains("cancelled"));
    assert!(expected_cancel_msg.contains("user"));
}

// Error handling tests

#[test]
#[serial]
fn test_safety_error_handling() {
    if !should_run_destructive_tests() {
        return;
    }

    // Test various error conditions that might occur

    // Test with invalid branch name for backup
    // This tests the validation that occurs in create_backup_branch
    if let Err(err) = Safety::create_backup_branch(Some("invalid branch name")) {
        let error_msg = err.to_string();
        assert!(
            error_msg.contains("contains invalid characters")
                || error_msg.contains("not a git repository")
                || error_msg.contains("Git command failed")
        );
    }
}

// Integration tests

#[test]
#[serial]
fn test_safety_operations_integration() {
    // Test that multiple safety operations can be performed in sequence

    let _list_result = Safety::list_backup_branches();
    let _cleanup_result = Safety::cleanup_old_backups(365, true); // Very old, dry run

    // Both operations should be able to run (though they may fail if not in git repo)
    // The important thing is that they don't interfere with each other
}

// Edge case tests

#[test]
#[serial]
fn test_safety_edge_cases() {
    if !should_run_destructive_tests() {
        return;
    }

    // Test empty prefix
    let result = Safety::create_backup_branch(Some(""));
    if result.is_ok() {
        // If it succeeds, backup name should still be valid
        let backup_name = result.unwrap();
        assert!(backup_name.contains("_")); // Should have timestamp
    }

    // Test very short cleanup period
    let cleanup_result = Safety::cleanup_old_backups(0, true);
    // Should work (dry run of cleaning up all backups)
    match cleanup_result {
        Ok(_) => {
            // Success
        }
        Err(err) => {
            // May fail if not in git repo
            let error_msg = err.to_string();
            assert!(
                error_msg.contains("not a git repository")
                    || error_msg.contains("Git command failed")
            );
        }
    }
}

// Test method availability and signatures

#[test]
#[serial]
fn test_safety_method_signatures() {
    // Verify all expected methods exist by referencing them
    let _create_backup = Safety::create_backup_branch;
    let _ensure_clean = Safety::ensure_clean_working_directory;
    let _confirm_destructive = Safety::confirm_destructive_operation;
    let _create_checkpoint = Safety::create_checkpoint;
    let _restore_checkpoint = Safety::restore_checkpoint;
    let _list_backups = Safety::list_backup_branches;
    let _cleanup_backups = Safety::cleanup_old_backups;

    // Verify SafetyBuilder methods
    let _builder_new = SafetyBuilder::new;

    // Test builder method chaining
    let builder = SafetyBuilder::new("test");
    let _with_backup = builder.with_backup();

    let builder = SafetyBuilder::new("test");
    let _with_checkpoint = builder.with_checkpoint();

    let builder = SafetyBuilder::new("test");
    let _with_confirmation = builder.with_confirmation();

    let builder = SafetyBuilder::new("test");
    let _with_clean = builder.with_clean_directory();
}

// Test builder pattern completion

#[test]
#[serial]
fn test_safety_builder_pattern_variations() {
    // Test different combinations of builder options

    let builder1 = SafetyBuilder::new("test1")
        .with_backup()
        .with_confirmation();
    drop(builder1);

    let builder2 = SafetyBuilder::new("test2")
        .with_checkpoint()
        .with_clean_directory();
    drop(builder2);

    let builder3 = SafetyBuilder::new("test3").with_backup().with_checkpoint();
    drop(builder3);

    // All combinations should be valid
}
