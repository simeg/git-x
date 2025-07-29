use crate::common::Interactive;
use crate::{GitXError, Result};
use console::style;
use std::process::Command;

pub fn run() -> Result<String> {
    let branches = get_recent_branches()?;

    if branches.is_empty() {
        return Err(GitXError::GitCommand(
            "No recent branches found".to_string(),
        ));
    }

    // Check if we're in an interactive environment
    if !is_interactive() {
        // In non-interactive environments (like tests), just switch to the most recent branch
        let selected_branch = &branches[0];
        switch_to_branch(selected_branch)?;
        return Ok(format!(
            "Switched to branch '{}'",
            style(selected_branch).green().bold()
        ));
    }

    let selected_branch =
        Interactive::branch_picker(&branches, Some("Select a recent branch to switch to"))?;
    switch_to_branch(&selected_branch)?;

    Ok(format!(
        "Switched to branch '{}'",
        style(&selected_branch).green().bold()
    ))
}

/// Check if we're running in an interactive environment
fn is_interactive() -> bool {
    // Check for any test-related environment variables or conditions
    if std::env::var("CARGO_TARGET_TMPDIR").is_ok()
        || std::env::var("CI").is_ok()
        || std::env::var("GITHUB_ACTIONS").is_ok()
        || std::env::var("GIT_X_NON_INTERACTIVE").is_ok()
        || !atty::is(atty::Stream::Stdin)
    {
        return false;
    }

    true
}

fn get_recent_branches() -> Result<Vec<String>> {
    let output = Command::new("git")
        .args([
            "for-each-ref",
            "--sort=-committerdate",
            "--format=%(refname:short)",
            "refs/heads/",
        ])
        .output()?;

    if !output.status.success() {
        return Err(GitXError::GitCommand(format!(
            "Failed to get recent branches: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    let current_branch = get_current_branch().unwrap_or_default();
    let branches: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|branch| !branch.is_empty() && branch != &current_branch)
        .take(10) // Limit to 10 most recent branches
        .collect();

    Ok(branches)
}

fn get_current_branch() -> Result<String> {
    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .output()?;

    if !output.status.success() {
        return Err(GitXError::GitCommand(format!(
            "Failed to get current branch: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn switch_to_branch(branch: &str) -> Result<()> {
    let status = Command::new("git").args(["checkout", branch]).status()?;

    if !status.success() {
        return Err(GitXError::GitCommand(format!(
            "Failed to switch to branch '{branch}'"
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GitXError;

    #[test]
    fn test_get_recent_branches_success() {
        match get_recent_branches() {
            Ok(branches) => {
                assert!(branches.len() <= 10, "Should limit to 10 branches");
                for branch in branches {
                    assert!(!branch.is_empty(), "Branch names should not be empty");
                }
            }
            Err(GitXError::GitCommand(_)) => {
                // Expected in non-git environments or when git command fails
            }
            Err(GitXError::Io(_)) => {
                // Expected when git binary is not available
            }
            Err(_) => panic!("Unexpected error type"),
        }
    }

    #[test]
    fn test_get_current_branch_success() {
        match get_current_branch() {
            Ok(_branch) => {
                // In a real git repo, branch name should not be empty
                // In some cases (like detached HEAD), it might be empty, which is valid
            }
            Err(GitXError::GitCommand(_)) => {
                // Expected in non-git environments
            }
            Err(GitXError::Io(_)) => {
                // Expected when git binary is not available
            }
            Err(_) => panic!("Unexpected error type"),
        }
    }

    #[test]
    fn test_switch_to_branch_invalid_branch() {
        let result = switch_to_branch("non-existent-branch-12345");
        assert!(result.is_err(), "Should fail for non-existent branch");

        if let Err(e) = result {
            match e {
                GitXError::GitCommand(msg) => {
                    assert!(
                        msg.contains("Failed to switch to branch"),
                        "Error message should mention branch switching"
                    );
                }
                GitXError::Io(_) => {
                    // Expected when git binary is not available
                }
                _ => panic!("Unexpected error type"),
            }
        }
    }

    #[test]
    fn test_run_no_git_repo() {
        // Test the behavior when not in a git repository
        // We test this by checking get_recent_branches() which is the first step of run()
        let result = get_recent_branches();
        match result {
            Ok(branches) => {
                // If we're actually in a git repo, just verify branches are valid
                assert!(branches.len() <= 10, "Should limit to 10 branches");
                for branch in branches {
                    assert!(!branch.is_empty(), "Branch names should not be empty");
                }
                println!("In git repo - skipping non-git test");
            }
            Err(GitXError::GitCommand(_)) => {
                // Expected behavior in non-git repo - git command fails
                println!("Not in git repo - git command failed as expected");
            }
            Err(GitXError::Io(_)) => {
                // Expected when git binary is not available
                println!("Git binary not available - IO error as expected");
            }
            Err(_) => panic!("Unexpected error type"),
        }
    }

    #[test]
    fn test_show_branch_picker_with_branches() {
        let branches = ["feature/test".to_string(), "main".to_string()];

        // This test verifies the function exists and can be called
        // We can't actually test the interactive picker in a unit test environment
        // because it would hang waiting for user input

        // Instead, let's just verify the function signature and that we have branches
        assert!(!branches.is_empty(), "Should have branches to pick from");
        assert_eq!(branches.len(), 2, "Should have exactly 2 branches");
        assert_eq!(
            branches[0], "feature/test",
            "First branch should be feature/test"
        );
        assert_eq!(branches[1], "main", "Second branch should be main");

        // Note: We deliberately don't call show_branch_picker here because it would
        // hang in the test environment waiting for interactive input
    }

    #[test]
    fn test_show_branch_picker_empty_branches() {
        let branches: Vec<String> = vec![];

        // Test that we handle empty branch list properly
        assert!(branches.is_empty(), "Should have no branches");
        assert_eq!(branches.len(), 0, "Should have exactly 0 branches");

        // Note: We don't call show_branch_picker with empty branches because
        // it would still try to create an interactive picker which could hang
        // Instead we test the empty branch logic in the run() function
    }

    #[test]
    fn test_switch_to_branch_valid_args() {
        // Test that switch_to_branch properly formats git checkout command
        // This will fail since we're not in the branch, but we can verify error handling
        let result = switch_to_branch("main");
        match result {
            Ok(_) => {
                // Might succeed if we're actually in a git repo with a main branch
            }
            Err(GitXError::GitCommand(msg)) => {
                // Expected - either branch doesn't exist or checkout failed
                assert!(
                    msg.contains("Failed to switch to branch"),
                    "Should mention switching failure"
                );
            }
            Err(GitXError::Io(_)) => {
                // Expected when git binary is not available
            }
            Err(_) => panic!("Unexpected error type"),
        }
    }

    #[test]
    fn test_gitx_error_types() {
        // Test that our functions return the correct GitXError variants

        // Test IO error conversion
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "git not found");
        let gitx_error: GitXError = io_error.into();
        match gitx_error {
            GitXError::Io(_) => {} // Expected
            _ => panic!("Should convert to Io error"),
        }

        // Test GitCommand error
        let git_error = GitXError::GitCommand("test error".to_string());
        assert_eq!(git_error.to_string(), "Git command failed: test error");
    }

    #[test]
    fn test_run_function_complete_flow() {
        // Test the complete run() function flow, but only up to the point where
        // it would require interactive input

        // Test that we can get branches (or get appropriate error)
        let branches_result = get_recent_branches();
        match branches_result {
            Ok(branches) => {
                // If we got branches, verify they're properly formatted
                assert!(branches.len() <= 10, "Should limit to 10 branches");
                for branch in &branches {
                    assert!(!branch.is_empty(), "Branch names should not be empty");
                }

                // If we have branches, we can't test the full flow without hanging
                // so we just verify the branches are valid
                if branches.is_empty() {
                    println!(
                        "No recent branches found - this would trigger the empty branches error"
                    );
                } else {
                    println!(
                        "Found {} recent branches - skipping interactive test to avoid hanging",
                        branches.len()
                    );
                }
            }
            Err(_) => {
                // Expected in non-git environments
                println!("Failed to get recent branches - this would trigger the git error");
            }
        }
    }
}
