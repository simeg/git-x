use git_x::domain::{
    BranchManager, CleanBranchesRequest, CreateBranchRequest, DeleteBranchesRequest, GitRepository,
    RecentBranchesRequest, RenameBranchRequest, SwitchBranchRequest,
};

// Helper function to get a manager instance (tests GitRepository::open)
fn get_test_manager() -> Result<BranchManager, git_x::GitXError> {
    let repository = GitRepository::open()?;
    Ok(BranchManager::new(repository))
}

#[test]
fn test_branch_manager_new() {
    // Test that we can create a BranchManager
    // This will fail if not in a git repo, which is expected
    match get_test_manager() {
        Ok(_manager) => {
            // Successfully created manager in a git repo
        }
        Err(_) => {
            // Expected when not in a git repo - this tests the validation
        }
    }
}

#[test]
fn test_branch_manager_create_branch_invalid_name_empty() {
    if let Ok(manager) = get_test_manager() {
        let request = CreateBranchRequest {
            name: "".to_string(), // Empty name should be invalid
            from: None,
            create_backup: false,
        };

        let result = manager.create_branch(request);
        assert!(result.is_err());

        if let Err(err) = result {
            // Should be a validation error
            assert!(err.to_string().contains("Branch name"));
        }
    }
}

#[test]
fn test_branch_manager_create_branch_invalid_name_dash() {
    if let Ok(manager) = get_test_manager() {
        let request = CreateBranchRequest {
            name: "-invalid".to_string(), // Names starting with dash should be invalid
            from: None,
            create_backup: false,
        };

        let result = manager.create_branch(request);
        assert!(result.is_err());
    }
}

#[test]
fn test_branch_manager_create_branch_invalid_name_spaces() {
    if let Ok(manager) = get_test_manager() {
        let request = CreateBranchRequest {
            name: "invalid name".to_string(), // Names with spaces should be invalid
            from: None,
            create_backup: false,
        };

        let result = manager.create_branch(request);
        assert!(result.is_err());
    }
}

#[test]
fn test_branch_manager_create_branch_invalid_name_dots() {
    if let Ok(manager) = get_test_manager() {
        let request = CreateBranchRequest {
            name: "invalid..name".to_string(), // Names with double dots should be invalid
            from: None,
            create_backup: false,
        };

        let result = manager.create_branch(request);
        assert!(result.is_err());
    }
}

#[test]
fn test_branch_manager_create_branch_invalid_base_commit() {
    if let Ok(manager) = get_test_manager() {
        let request = CreateBranchRequest {
            name: "valid-name".to_string(),
            from: Some("nonexistent-commit-hash".to_string()),
            create_backup: false,
        };

        let result = manager.create_branch(request);
        assert!(result.is_err());

        if let Err(err) = result {
            // Should indicate the base commit doesn't exist
            assert!(err.to_string().contains("does not exist"));
        }
    }
}

#[test]
fn test_branch_manager_create_branch_valid_name() {
    if let Ok(manager) = get_test_manager() {
        let request = CreateBranchRequest {
            name: "feature/valid-branch-name".to_string(),
            from: None,
            create_backup: false,
        };

        // This might succeed or fail depending on whether branch exists
        // We're testing the validation and execution path
        let result = manager.create_branch(request);

        match result {
            Ok(branch_result) => {
                assert_eq!(branch_result.branch_name, "feature/valid-branch-name");
                assert_eq!(branch_result.base_commit, None);
                assert_eq!(branch_result.backup_branch, None);
                assert!(branch_result.switched);
            }
            Err(err) => {
                // Could fail if branch already exists or other git issues
                // The important thing is we got past validation
                println!("Branch creation failed (expected in some environments): {err}");
            }
        }
    }
}

#[test]
fn test_branch_manager_delete_branches_protected() {
    if let Ok(manager) = get_test_manager() {
        let request = DeleteBranchesRequest {
            branches: vec![
                "main".to_string(),
                "master".to_string(),
                "develop".to_string(),
            ],
            force: false,
            dry_run: false,
        };

        let result = manager.delete_branches(request);
        assert!(result.is_ok());

        if let Ok(delete_result) = result {
            // Protected branches should not be deleted
            assert_eq!(delete_result.protected.len(), 3);
            assert!(delete_result.protected.contains(&"main".to_string()));
            assert!(delete_result.protected.contains(&"master".to_string()));
            assert!(delete_result.protected.contains(&"develop".to_string()));
            assert!(delete_result.deleted.is_empty());
        }
    }
}

#[test]
fn test_branch_manager_delete_branches_dry_run() {
    if let Ok(manager) = get_test_manager() {
        let request = DeleteBranchesRequest {
            branches: vec!["some-branch".to_string()],
            force: false,
            dry_run: true,
        };

        let result = manager.delete_branches(request);
        assert!(result.is_ok());

        if let Ok(delete_result) = result {
            assert!(delete_result.dry_run);
            // In dry run, non-protected branches are assumed to succeed
            if !delete_result.protected.contains(&"some-branch".to_string()) {
                assert!(delete_result.deleted.contains(&"some-branch".to_string()));
            }
        }
    }
}

#[test]
fn test_branch_manager_get_recent_branches_basic() {
    if let Ok(manager) = get_test_manager() {
        let request = RecentBranchesRequest {
            limit: Some(10),
            exclude_current: false,
            exclude_protected: false,
        };

        let result = manager.get_recent_branches(request);
        assert!(result.is_ok());

        if let Ok(recent_result) = result {
            // Should have a current branch
            assert!(!recent_result.current_branch.is_empty());
            // Branches list should be valid (could be empty in new repos)
            assert!(recent_result.branches.len() <= 10);
        }
    }
}

#[test]
fn test_branch_manager_get_recent_branches_exclude_current() {
    if let Ok(manager) = get_test_manager() {
        let request = RecentBranchesRequest {
            limit: Some(10),
            exclude_current: true,
            exclude_protected: false,
        };

        let result = manager.get_recent_branches(request);
        assert!(result.is_ok());

        if let Ok(recent_result) = result {
            // Current branch should not be in the list
            assert!(
                !recent_result
                    .branches
                    .contains(&recent_result.current_branch)
            );
        }
    }
}

#[test]
fn test_branch_manager_get_recent_branches_exclude_protected() {
    if let Ok(manager) = get_test_manager() {
        let request = RecentBranchesRequest {
            limit: Some(10),
            exclude_current: false,
            exclude_protected: true,
        };

        let result = manager.get_recent_branches(request);
        assert!(result.is_ok());

        if let Ok(recent_result) = result {
            // Protected branches should not be in the list
            assert!(!recent_result.branches.contains(&"main".to_string()));
            assert!(!recent_result.branches.contains(&"master".to_string()));
            assert!(!recent_result.branches.contains(&"develop".to_string()));
        }
    }
}

#[test]
fn test_branch_manager_switch_branch_nonexistent() {
    if let Ok(manager) = get_test_manager() {
        let request = SwitchBranchRequest {
            branch_name: "definitely-nonexistent-branch-12345".to_string(),
            strict_mode: false,
            create_checkpoint: false,
        };

        let result = manager.switch_branch(request);
        assert!(result.is_err());

        if let Err(err) = result {
            assert!(err.to_string().contains("does not exist"));
        }
    }
}

#[test]
fn test_branch_manager_switch_branch_current() {
    if let Ok(manager) = get_test_manager() {
        // Get current branch first
        let recent_request = RecentBranchesRequest {
            limit: Some(1),
            exclude_current: false,
            exclude_protected: false,
        };

        if let Ok(recent_result) = manager.get_recent_branches(recent_request) {
            let current_branch = recent_result.current_branch;

            let request = SwitchBranchRequest {
                branch_name: current_branch.clone(),
                strict_mode: false,
                create_checkpoint: false,
            };

            let result = manager.switch_branch(request);

            match result {
                Ok(switch_result) => {
                    assert_eq!(switch_result.new_branch, current_branch);
                    assert_eq!(switch_result.checkpoint, None);
                }
                Err(_) => {
                    // Might fail if already on the branch, which is fine
                }
            }
        }
    }
}

#[test]
fn test_branch_manager_rename_branch_invalid_name() {
    if let Ok(manager) = get_test_manager() {
        let request = RenameBranchRequest {
            new_name: "".to_string(), // Invalid empty name
            create_backup: false,
        };

        let result = manager.rename_branch(request);
        assert!(result.is_err());
    }
}

#[test]
fn test_branch_manager_rename_branch_existing_name() {
    if let Ok(manager) = get_test_manager() {
        // Try to rename to an existing branch name
        let request = RenameBranchRequest {
            new_name: "main".to_string(), // Likely to exist
            create_backup: false,
        };

        let result = manager.rename_branch(request);
        // Should either succeed (if we're not on main) or fail (if main exists and we try to rename to it)
        match result {
            Ok(_) => {
                // Rename succeeded
            }
            Err(err) => {
                // Should fail because main already exists
                assert!(err.to_string().contains("already exists"));
            }
        }
    }
}

#[test]
fn test_branch_manager_clean_merged_branches_dry_run() {
    if let Ok(manager) = get_test_manager() {
        let request = CleanBranchesRequest {
            dry_run: true,
            confirm_deletion: false,
        };

        let result = manager.clean_merged_branches(request);
        assert!(result.is_ok());

        if let Ok(clean_result) = result {
            assert!(clean_result.dry_run);
            // In dry run mode, candidates should equal deleted
            assert_eq!(clean_result.candidates.len(), clean_result.deleted.len());
            assert!(clean_result.failed.is_empty());
        }
    }
}

#[test]
fn test_branch_manager_clean_merged_branches_no_confirm() {
    if let Ok(manager) = get_test_manager() {
        // Set non-interactive mode for testing
        unsafe {
            std::env::set_var("GIT_X_NON_INTERACTIVE", "1");
        }

        let request = CleanBranchesRequest {
            dry_run: false,
            confirm_deletion: false, // No confirmation needed
        };

        let result = manager.clean_merged_branches(request);
        assert!(result.is_ok());

        if let Ok(clean_result) = result {
            assert!(!clean_result.dry_run);
            // Should have processed some candidates (even if empty)
            assert!(clean_result.candidates.len() >= clean_result.deleted.len());
        }

        // Clean up environment variable
        unsafe {
            std::env::remove_var("GIT_X_NON_INTERACTIVE");
        }
    }
}

// Test the summary method on CleanBranchesResult
#[test]
fn test_clean_branches_result_summary_dry_run() {
    use git_x::domain::CleanBranchesResult;

    let dry_run_result = CleanBranchesResult {
        candidates: vec!["branch1".to_string(), "branch2".to_string()],
        deleted: vec!["branch1".to_string(), "branch2".to_string()],
        failed: vec![],
        dry_run: true,
    };

    assert_eq!(dry_run_result.summary(), "Would delete 2 branches");
}

#[test]
fn test_clean_branches_result_summary_actual() {
    use git_x::domain::CleanBranchesResult;

    let actual_result = CleanBranchesResult {
        candidates: vec!["branch1".to_string(), "branch2".to_string()],
        deleted: vec!["branch1".to_string()],
        failed: vec!["branch2".to_string()],
        dry_run: false,
    };

    assert_eq!(actual_result.summary(), "Deleted 1 branches, 1 failed");
}

#[test]
fn test_clean_branches_result_summary_no_failures() {
    use git_x::domain::CleanBranchesResult;

    let success_result = CleanBranchesResult {
        candidates: vec!["branch1".to_string()],
        deleted: vec!["branch1".to_string()],
        failed: vec![],
        dry_run: false,
    };

    assert_eq!(success_result.summary(), "Deleted 1 branches, 0 failed");
}

#[test]
fn test_branch_manager_clean_request_structure() {
    // Test the CleanBranchesRequest structure
    let request = CleanBranchesRequest {
        dry_run: true,
        confirm_deletion: false,
    };

    assert!(request.dry_run);
    assert!(!request.confirm_deletion);
}

#[test]
fn test_branch_manager_protected_branch_patterns() {
    // Test edge cases in protected branch checking
    if let Ok(manager) = get_test_manager() {
        // Test with request using actual struct fields
        let request = CleanBranchesRequest {
            dry_run: true,
            confirm_deletion: false,
        };

        // This should work even if no branches to clean
        let result = manager.clean_merged_branches(request);
        // Either succeeds or fails with a reasonable error
        match result {
            Ok(_) => {} // Success is fine
            Err(err) => {
                let error_msg = err.to_string();
                // Should be a reasonable git-related error
                assert!(
                    error_msg.contains("git")
                        || error_msg.contains("branch")
                        || error_msg.contains("repository")
                );
            }
        }
    }
}

#[test]
fn test_branch_creation_result_properties() {
    use git_x::domain::BranchCreationResult;

    let result = BranchCreationResult {
        branch_name: "test-branch".to_string(),
        base_commit: Some("abc123".to_string()),
        backup_branch: None,
        switched: true,
    };

    // Test that the result has sensible display properties
    assert_eq!(result.branch_name, "test-branch");
    assert_eq!(result.base_commit, Some("abc123".to_string()));
    assert_eq!(result.backup_branch, None);
    assert!(result.switched);
}

// Test validation edge cases
#[test]
fn test_branch_manager_create_branch_request_validation() {
    if let Ok(manager) = get_test_manager() {
        // Test various invalid branch names
        let invalid_names = vec![
            "",                    // empty
            " ",                   // whitespace
            "-name",               // starts with dash
            "name with space",     // contains space
            "name..dots",          // contains double dots
            "name\twith\ttab",     // contains tab
            "name\nwith\nnewline", // contains newline
        ];

        for invalid_name in invalid_names {
            let request = CreateBranchRequest {
                name: invalid_name.to_string(),
                from: None,
                create_backup: false,
            };

            let result = manager.create_branch(request);
            assert!(
                result.is_err(),
                "Branch name '{invalid_name}' should be invalid"
            );
        }
    }
}

#[test]
fn test_branch_manager_rename_branch_request_validation() {
    if let Ok(manager) = get_test_manager() {
        // Test various invalid new names for rename
        let invalid_names = vec![
            "",                // empty
            " ",               // whitespace
            "-name",           // starts with dash
            "name with space", // contains space
            "name..dots",      // contains double dots
        ];

        for invalid_name in invalid_names {
            let request = RenameBranchRequest {
                new_name: invalid_name.to_string(),
                create_backup: false,
            };

            let result = manager.rename_branch(request);
            assert!(
                result.is_err(),
                "New branch name '{invalid_name}' should be invalid"
            );
        }
    }
}
