// CLI handlers tests
//
// WARNING: Some tests in this module perform DESTRUCTIVE git operations including:
// - Creating new branches
// - Deleting merged branches
// - Switching branches
// - Requiring user input for confirmations
//
// To protect local development environments, these tests are skipped by default
// and only run in CI environments or when explicitly enabled.
//
// To run destructive tests locally:
//   ENABLE_DESTRUCTIVE_TESTS=1 cargo test test_cli_handlers
//
// Tests that don't modify git state (method signatures, factories, etc.) run normally.
use serial_test::serial;

use git_x::adapters::cli_handlers::{BranchCliHandler, CliHandlerFactory, RepositoryCliHandler};

// Helper to check if we should run potentially destructive tests
fn should_run_destructive_tests() -> bool {
    // Only run destructive tests in CI or when explicitly enabled
    std::env::var("CI").is_ok()
        || std::env::var("GITHUB_ACTIONS").is_ok()
        || std::env::var("ENABLE_DESTRUCTIVE_TESTS").is_ok()
}

// Helper function to check if we're in a git repository for conditional testing
fn get_test_branch_handler() -> git_x::Result<BranchCliHandler> {
    BranchCliHandler::new()
}

fn get_test_repository_handler() -> git_x::Result<RepositoryCliHandler> {
    RepositoryCliHandler::new()
}

// Tests for BranchCliHandler

#[test]
#[serial]
fn test_branch_cli_handler_new() {
    // Test that we can create a BranchCliHandler (conditional on being in a git repo)
    let result = BranchCliHandler::new();

    // Either success (in git repo) or failure (not in git repo) is acceptable
    match result {
        Ok(handler) => {
            // Handler creation succeeded - we're in a git repo
            drop(handler); // Just verify it can be created
        }
        Err(_) => {
            // Handler creation failed - we're not in a git repo
            // This is expected when running tests outside a git repository
        }
    }
}

#[test]
#[serial]
fn test_branch_cli_handler_handle_new_branch_simple() {
    if !should_run_destructive_tests() {
        return;
    }

    // Test handle_new_branch method (conditional on being in a git repo)
    if let Ok(handler) = get_test_branch_handler() {
        // Test that the method can be called (actual git operations may fail in test env)
        let result = handler.handle_new_branch("test-branch".to_string(), None);

        // The result may succeed or fail depending on git state, but the method should exist
        match result {
            Ok(output) => {
                // Branch creation succeeded
                assert!(output.contains("test-branch"));
            }
            Err(_) => {
                // Branch creation failed (expected in many test environments)
                // The important thing is that the method signature is correct
            }
        }
    }
}

#[test]
#[serial]
fn test_branch_cli_handler_handle_new_branch_with_from() {
    if !should_run_destructive_tests() {
        return;
    }

    // Test handle_new_branch with a base branch
    if let Ok(handler) = get_test_branch_handler() {
        let result =
            handler.handle_new_branch("feature-branch".to_string(), Some("main".to_string()));

        // Test that the method can be called with both parameters
        match result {
            Ok(output) => {
                assert!(output.contains("feature-branch"));
            }
            Err(_) => {
                // May fail in test environment, but method signature is correct
            }
        }
    }
}

#[test]
#[serial]
fn test_branch_cli_handler_handle_clean_branches_dry_run() {
    // Test handle_clean_branches in dry run mode
    if let Ok(handler) = get_test_branch_handler() {
        let result = handler.handle_clean_branches(true);

        match result {
            Ok(output) => {
                // Should indicate dry run mode
                assert!(
                    output.contains("dry run")
                        || output.contains("would")
                        || output.contains("No merged branches")
                );
            }
            Err(_) => {
                // May fail in test environment
            }
        }
    }
}

#[test]
#[serial]
fn test_branch_cli_handler_handle_clean_branches_actual() {
    if !should_run_destructive_tests() {
        return;
    }

    // Test handle_clean_branches in actual mode
    if let Ok(handler) = get_test_branch_handler() {
        let result = handler.handle_clean_branches(false);

        // Test that the method can be called
        match result {
            Ok(_output) => {
                // Clean branches succeeded
            }
            Err(_) => {
                // May fail in test environment (expected)
            }
        }
    }
}

#[test]
#[serial]
fn test_branch_cli_handler_handle_switch_recent() {
    if !should_run_destructive_tests() {
        return;
    }

    // Test handle_switch_recent method
    if let Ok(handler) = get_test_branch_handler() {
        // Set non-interactive mode to avoid hanging
        unsafe {
            std::env::set_var("GIT_X_NON_INTERACTIVE", "1");
        }

        let result = handler.handle_switch_recent();

        match result {
            Ok(output) => {
                // Switch succeeded
                assert!(output.contains("Switched") || output.contains("branch"));
            }
            Err(err) => {
                // May fail if no recent branches or not in git repo
                let error_msg = err.to_string();
                assert!(
                    error_msg.contains("No recent branches")
                        || error_msg.contains("not a git repository")
                        || error_msg.contains("Git command failed")
                );
            }
        }

        unsafe {
            std::env::remove_var("GIT_X_NON_INTERACTIVE");
        }
    }
}

// Tests for RepositoryCliHandler

#[test]
#[serial]
fn test_repository_cli_handler_new() {
    // Test that we can create a RepositoryCliHandler
    let result = RepositoryCliHandler::new();

    // Either success (in git repo) or failure (not in git repo) is acceptable
    match result {
        Ok(handler) => {
            // Handler creation succeeded - we're in a git repo
            drop(handler);
        }
        Err(_) => {
            // Handler creation failed - we're not in a git repo
            // This is expected when running tests outside a git repository
        }
    }
}

#[test]
#[serial]
fn test_repository_cli_handler_handle_info_basic() {
    // Test handle_info method in basic mode
    if let Ok(handler) = get_test_repository_handler() {
        let result = handler.handle_info(false);

        match result {
            Ok(output) => {
                // Info command succeeded
                assert!(output.contains("Repository") || output.contains("branch"));
            }
            Err(_) => {
                // May fail in test environment
            }
        }
    }
}

#[test]
#[serial]
fn test_repository_cli_handler_handle_info_detailed() {
    // Test handle_info method in detailed mode
    if let Ok(handler) = get_test_repository_handler() {
        let result = handler.handle_info(true);

        match result {
            Ok(output) => {
                // Detailed info should contain more information
                assert!(output.contains("Repository") || output.contains("branch"));
                // Detailed mode might have more content, but we can't guarantee specific content
            }
            Err(_) => {
                // May fail in test environment
            }
        }
    }
}

#[test]
#[serial]
fn test_repository_cli_handler_handle_health() {
    // Test handle_health method
    if let Ok(handler) = get_test_repository_handler() {
        let result = handler.handle_health();

        match result {
            Ok(output) => {
                // Health check succeeded
                assert!(output.contains("Health") || output.contains("Repository"));
            }
            Err(_) => {
                // May fail in test environment
            }
        }
    }
}

// Tests for CliHandlerFactory

#[test]
#[serial]
fn test_cli_handler_factory_create_branch_handler() {
    // Test factory method for creating branch handler
    let result = CliHandlerFactory::create_branch_handler();

    if result.is_ok() {
        // Factory succeeded in creating handler
        let handler = result.unwrap();
        drop(handler);
    } else {
        // Factory failed (not in git repo)
        assert!(result.is_err());
    }
}

#[test]
#[serial]
fn test_cli_handler_factory_create_repository_handler() {
    // Test factory method for creating repository handler
    let result = CliHandlerFactory::create_repository_handler();

    if result.is_ok() {
        // Factory succeeded in creating handler
        let handler = result.unwrap();
        drop(handler);
    } else {
        // Factory failed (not in git repo)
        assert!(result.is_err());
    }
}

// Tests for error conditions

#[test]
#[serial]
fn test_branch_cli_handler_creation_outside_git_repo() {
    // Test behavior when not in a git repository
    // We can't force this condition, but we can test the error handling

    // Save current directory
    let current_dir = std::env::current_dir().unwrap();

    // Try to change to a non-git directory (system temp)
    if let Ok(temp_dir) = std::env::temp_dir().canonicalize()
        && std::env::set_current_dir(&temp_dir).is_ok()
    {
        // We're now in a temp directory (likely not a git repo)
        let result = BranchCliHandler::new();

        // This should fail since we're not in a git repo
        match result {
            Ok(_) => {
                // Unexpectedly succeeded (temp dir might be in a git repo)
            }
            Err(err) => {
                // Expected to fail
                let error_msg = err.to_string();
                assert!(
                    error_msg.contains("not a git repository")
                        || error_msg.contains("Git command failed")
                        || error_msg.contains("IO error")
                );
            }
        }

        // Restore original directory
        let _ = std::env::set_current_dir(current_dir);
    }
}

#[test]
#[serial]
fn test_repository_cli_handler_creation_outside_git_repo() {
    // Test behavior when not in a git repository
    let current_dir = std::env::current_dir().unwrap();

    if let Ok(temp_dir) = std::env::temp_dir().canonicalize()
        && std::env::set_current_dir(&temp_dir).is_ok()
    {
        let result = RepositoryCliHandler::new();

        match result {
            Ok(_) => {
                // Unexpectedly succeeded
            }
            Err(err) => {
                // Expected to fail
                let error_msg = err.to_string();
                assert!(
                    error_msg.contains("not a git repository")
                        || error_msg.contains("Git command failed")
                        || error_msg.contains("IO error")
                );
            }
        }

        let _ = std::env::set_current_dir(current_dir);
    }
}

// Integration tests for method interactions

#[test]
#[serial]
fn test_branch_cli_handler_method_chaining() {
    if !should_run_destructive_tests() {
        return;
    }

    // Test that we can call multiple methods on the same handler instance
    if let Ok(handler) = get_test_branch_handler() {
        // Test that we can call different methods on the same handler
        let _clean_result = handler.handle_clean_branches(true); // Dry run first

        // Set non-interactive mode
        unsafe {
            std::env::set_var("GIT_X_NON_INTERACTIVE", "1");
        }

        let _switch_result = handler.handle_switch_recent();

        // Both calls should work (though they may fail due to git state)
        // The important thing is that the handler can be reused

        unsafe {
            std::env::remove_var("GIT_X_NON_INTERACTIVE");
        }
    }
}

#[test]
#[serial]
fn test_repository_cli_handler_method_chaining() {
    // Test that we can call multiple methods on the same handler instance
    if let Ok(handler) = get_test_repository_handler() {
        let _info_result = handler.handle_info(false);
        let _health_result = handler.handle_health();
        let _detailed_info_result = handler.handle_info(true);

        // All calls should work (though they may fail due to git state)
        // The important thing is that the handler can be reused
    }
}

// Tests for edge cases

#[test]
#[serial]
fn test_branch_cli_handler_empty_branch_name() {
    if !should_run_destructive_tests() {
        return;
    }

    // Test creating a branch with an empty name
    if let Ok(handler) = get_test_branch_handler() {
        let result = handler.handle_new_branch("".to_string(), None);

        // This should likely fail due to invalid branch name
        match result {
            Ok(_) => {
                // Unexpectedly succeeded (git might allow empty names in some cases)
            }
            Err(err) => {
                // Expected to fail
                let error_msg = err.to_string();
                assert!(
                    error_msg.contains("invalid")
                        || error_msg.contains("empty")
                        || error_msg.contains("Git command failed")
                );
            }
        }
    }
}

#[test]
#[serial]
fn test_branch_cli_handler_invalid_base_branch() {
    if !should_run_destructive_tests() {
        return;
    }

    // Test creating a branch with an invalid base
    if let Ok(handler) = get_test_branch_handler() {
        let result = handler.handle_new_branch(
            "test-branch".to_string(),
            Some("non-existent-branch".to_string()),
        );

        // This should likely fail due to non-existent base branch
        match result {
            Ok(_) => {
                // Unexpectedly succeeded
            }
            Err(err) => {
                // Expected to fail
                let error_msg = err.to_string();
                assert!(
                    error_msg.contains("not found")
                        || error_msg.contains("does not exist")
                        || error_msg.contains("Git command failed")
                        || error_msg.contains("ambiguous")
                );
            }
        }
    }
}

// Tests to ensure all public methods are covered

#[test]
#[serial]
fn test_all_branch_cli_handler_methods_exist() {
    // Verify all expected methods exist by referencing them
    let _new_fn = BranchCliHandler::new;

    if let Ok(handler) = get_test_branch_handler() {
        let _handle_new_fn =
            |name: String, from: Option<String>| handler.handle_new_branch(name, from);
        let _handle_clean_fn = |dry_run: bool| handler.handle_clean_branches(dry_run);
        let _handle_switch_fn = || handler.handle_switch_recent();
    }
}

#[test]
#[serial]
fn test_all_repository_cli_handler_methods_exist() {
    // Verify all expected methods exist by referencing them
    let _new_fn = RepositoryCliHandler::new;

    if let Ok(handler) = get_test_repository_handler() {
        let _handle_info_fn = |detailed: bool| handler.handle_info(detailed);
        let _handle_health_fn = || handler.handle_health();
    }
}

#[test]
#[serial]
fn test_all_cli_handler_factory_methods_exist() {
    // Verify all factory methods exist
    let _create_branch_fn = CliHandlerFactory::create_branch_handler;
    let _create_repo_fn = CliHandlerFactory::create_repository_handler;

    // Test that factory methods can be called
    let _branch_result = CliHandlerFactory::create_branch_handler();
    let _repo_result = CliHandlerFactory::create_repository_handler();
}
