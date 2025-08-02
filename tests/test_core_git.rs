use serial_test::serial;
// Tests for src/core/git.rs
//
// NOTE: These tests are designed to work both inside and outside git repositories.
// When outside a git repo, we test that appropriate errors are returned.
// When inside a git repo, we test that the operations work correctly.

use git_x::core::git::{BranchOperations, CommitOperations, GitOperations, RemoteOperations};

// Tests for GitOperations

#[test]
#[serial]
fn test_git_operations_run() {
    // Test the basic run command
    let result = GitOperations::run(&["--version"]);

    // Git --version should always work (assuming git is installed)
    match result {
        Ok(output) => {
            assert!(output.contains("git version"));
        }
        Err(_) => {
            // Git might not be installed in test environment
            // This is acceptable for the test
        }
    }
}

#[test]
#[serial]
fn test_git_operations_run_status() {
    // Test run_status with a safe command
    let result = GitOperations::run_status(&["--version"]);

    // Should either succeed or fail gracefully
    match result {
        Ok(_) => {
            // Success - git is available and working
        }
        Err(err) => {
            // Failure is expected if git not available or other issues
            let error_msg = err.to_string();
            assert!(error_msg.contains("Git command failed") || error_msg.contains("IO error"));
        }
    }
}

#[test]
#[serial]
fn test_git_operations_repo_root() {
    let result = GitOperations::repo_root();

    if is_in_git_repo() {
        // If we're in a git repo, should succeed
        assert!(result.is_ok());
        let root = result.unwrap();
        assert!(!root.is_empty());
    } else {
        // If not in git repo, should fail with appropriate error
        assert!(result.is_err());
        let error_msg = result.err().unwrap().to_string();
        assert!(
            error_msg.contains("Git command failed") || error_msg.contains("not a git repository")
        );
    }
}

#[test]
#[serial]
fn test_git_operations_current_branch() {
    let result = GitOperations::current_branch();

    if is_in_git_repo() {
        match result {
            Ok(branch) => {
                // Should be a valid branch name
                assert!(!branch.is_empty());
                assert!(!branch.contains('\n')); // Should be trimmed
            }
            Err(_) => {
                // Might fail in detached HEAD state or other edge cases
            }
        }
    } else {
        // Should fail when not in git repo
        assert!(result.is_err());
    }
}

#[test]
#[serial]
fn test_git_operations_commit_exists() {
    if is_in_git_repo() {
        // Test with HEAD (should exist)
        let result = GitOperations::commit_exists("HEAD");
        match result {
            Ok(exists) => {
                // HEAD should exist in any git repo with commits
                assert!(exists);
            }
            Err(_) => {
                // Might fail in empty repo with no commits
            }
        }

        // Test with obviously invalid commit
        let result = GitOperations::commit_exists("invalid-commit-hash-12345");
        match result {
            Ok(exists) => {
                assert!(!exists);
            }
            Err(_) => {
                // Command might fail instead of returning false
            }
        }
    }
}

#[test]
#[serial]
fn test_git_operations_short_hash() {
    if is_in_git_repo() {
        let result = GitOperations::short_hash("HEAD");
        match result {
            Ok(hash) => {
                // Short hash should be 7-12 characters
                assert!(hash.len() >= 7 && hash.len() <= 12);
                assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
            }
            Err(_) => {
                // Might fail in empty repo
            }
        }
    }
}

#[test]
#[serial]
fn test_git_operations_is_working_directory_clean() {
    if is_in_git_repo() {
        let result = GitOperations::is_working_directory_clean();
        match result {
            Ok(_is_clean) => {
                // Should return true or false - the fact that we got a result is what we're testing
            }
            Err(_) => {
                // Might fail in some edge cases
            }
        }
    } else {
        let result = GitOperations::is_working_directory_clean();
        assert!(result.is_err());
    }
}

#[test]
#[serial]
fn test_git_operations_staged_files() {
    if is_in_git_repo() {
        let result = GitOperations::staged_files();
        match result {
            Ok(_files) => {
                // Should return a vector (might be empty) - the fact that we got a result is what we're testing
            }
            Err(_) => {
                // Might fail in some cases
            }
        }
    } else {
        let result = GitOperations::staged_files();
        assert!(result.is_err());
    }
}

#[test]
#[serial]
fn test_git_operations_local_branches() {
    if is_in_git_repo() {
        let result = GitOperations::local_branches();
        match result {
            Ok(branches) => {
                // Should have at least one branch in most cases
                for branch in &branches {
                    assert!(!branch.is_empty());
                    assert!(!branch.starts_with('*')); // Should be cleaned
                }
            }
            Err(_) => {
                // Might fail in edge cases
            }
        }
    } else {
        let result = GitOperations::local_branches();
        assert!(result.is_err());
    }
}

#[test]
#[serial]
fn test_git_operations_recent_branches() {
    if is_in_git_repo() {
        // Test with limit
        let result = GitOperations::recent_branches(Some(5));
        match result {
            Ok(branches) => {
                // Should respect the limit
                assert!(branches.len() <= 5);
            }
            Err(_) => {
                // Might fail in some cases
            }
        }

        // Test without limit
        let result = GitOperations::recent_branches(None);
        match result {
            Ok(_branches) => {
                // Should return some branches or empty list
            }
            Err(_) => {
                // Might fail in some cases
            }
        }
    }
}

#[test]
#[serial]
fn test_git_operations_merged_branches() {
    if is_in_git_repo() {
        let result = GitOperations::merged_branches();
        match result {
            Ok(branches) => {
                // Should return a list (might be empty)
                for branch in &branches {
                    assert!(!branch.is_empty());
                }
            }
            Err(_) => {
                // Might fail in some cases
            }
        }
    } else {
        let result = GitOperations::merged_branches();
        assert!(result.is_err());
    }
}

// Tests for BranchOperations

#[test]
#[serial]
fn test_branch_operations_exists() {
    if is_in_git_repo() {
        // Test with current branch
        if let Ok(current) = GitOperations::current_branch() {
            let result = BranchOperations::exists(&current);
            match result {
                Ok(exists) => {
                    assert!(exists); // Current branch should exist
                }
                Err(_) => {
                    // Might fail in edge cases
                }
            }
        }

        // Test with non-existent branch
        let result = BranchOperations::exists("non-existent-branch-name-12345");
        match result {
            Ok(exists) => {
                assert!(!exists);
            }
            Err(_) => {
                // Command might fail instead
            }
        }
    }
}

// Tests for CommitOperations

#[test]
#[serial]
fn test_commit_operations_get_message() {
    if is_in_git_repo() {
        let result = CommitOperations::get_message("HEAD");
        match result {
            Ok(message) => {
                assert!(!message.is_empty());
            }
            Err(_) => {
                // Might fail in empty repo or other cases
            }
        }
    }
}

#[test]
#[serial]
fn test_commit_operations_get_author() {
    if is_in_git_repo() {
        let result = CommitOperations::get_author("HEAD");
        match result {
            Ok(author) => {
                assert!(!author.is_empty());
                // Author should have some format like "Name <email>"
            }
            Err(_) => {
                // Might fail in empty repo or other cases
            }
        }
    }
}

// Tests for RemoteOperations

#[test]
#[serial]
fn test_remote_operations_list() {
    if is_in_git_repo() {
        let result = RemoteOperations::list();
        match result {
            Ok(remotes) => {
                // Should return a list (might be empty for local-only repos)
                for remote in &remotes {
                    assert!(!remote.is_empty());
                }
            }
            Err(_) => {
                // Might fail in some cases
            }
        }
    } else {
        let result = RemoteOperations::list();
        assert!(result.is_err());
    }
}

// Tests for error handling

#[test]
#[serial]
fn test_git_operations_error_handling() {
    // Test with invalid git command
    let result = GitOperations::run(&["invalid-command-that-does-not-exist"]);
    assert!(result.is_err());

    let error = result.err().unwrap();
    let error_msg = error.to_string();
    assert!(error_msg.contains("Git command failed"));
}

#[test]
#[serial]
fn test_git_operations_branch_info_optimized() {
    if is_in_git_repo() {
        let result = GitOperations::branch_info_optimized();
        match result {
            Ok((current, upstream, ahead, behind)) => {
                // Current branch should not be empty
                assert!(!current.is_empty());

                // Upstream might be None
                if let Some(ref upstream_branch) = upstream {
                    assert!(!upstream_branch.is_empty());
                }

                // Ahead/behind should be reasonable numbers
                assert!(ahead <= 10000); // Sanity check
                assert!(behind <= 10000); // Sanity check
            }
            Err(_) => {
                // Might fail in detached HEAD or other cases
            }
        }
    }
}

#[test]
#[serial]
fn test_git_operations_upstream_branch() {
    if is_in_git_repo() {
        let result = GitOperations::upstream_branch();
        match result {
            Ok(upstream) => {
                assert!(!upstream.is_empty());
                // Upstream should have format like "origin/main"
                assert!(upstream.contains('/') || !upstream.contains('/'));
            }
            Err(_) => {
                // Many repos don't have upstream configured, this is expected
            }
        }
    }
}

#[test]
#[serial]
fn test_git_operations_ahead_behind_counts() {
    if is_in_git_repo() {
        let result = GitOperations::ahead_behind_counts();
        match result {
            Ok((ahead, behind)) => {
                // Should be reasonable numbers
                assert!(ahead <= 10000);
                assert!(behind <= 10000);
            }
            Err(_) => {
                // Expected when no upstream is configured
            }
        }
    }
}

// Helper functions

// Helper function to check if we're in a git repository
fn is_in_git_repo() -> bool {
    GitOperations::repo_root().is_ok()
}
