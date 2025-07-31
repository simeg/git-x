use git_x::domain::{GitRepository, HealthLevel, RepositoryInfo};

// Helper function to get a repository instance
fn get_test_repository() -> Result<GitRepository, git_x::GitXError> {
    GitRepository::open()
}

#[test]
fn test_git_repository_open() {
    // Test that we can open a repository
    // This will fail if not in a git repo, which is expected
    match get_test_repository() {
        Ok(_repository) => {
            // Successfully opened repository in a git repo
        }
        Err(_) => {
            // Expected when not in a git repo - this tests the validation
        }
    }
}

#[test]
fn test_git_repository_info() {
    if let Ok(repository) = get_test_repository() {
        let result = repository.info();

        match result {
            Ok(info) => {
                // Verify RepositoryInfo structure is valid
                assert!(!info.name.is_empty());
                assert!(!info.root_path.is_empty());
                assert!(!info.current_branch.is_empty());

                // Test the derived methods
                let is_in_sync = info.is_in_sync();
                assert_eq!(is_in_sync, info.ahead_count == 0 && info.behind_count == 0);

                let has_local_changes = info.has_local_changes();
                assert_eq!(
                    has_local_changes,
                    !info.is_clean || info.staged_files_count > 0
                );

                let status_desc = info.status_description();
                assert!(!status_desc.is_empty());

                // Test specific status description content
                if info.upstream_branch.is_some() {
                    if info.is_in_sync() {
                        assert!(status_desc.contains("up to date"));
                    } else {
                        if info.ahead_count > 0 {
                            assert!(status_desc.contains("ahead"));
                        }
                        if info.behind_count > 0 {
                            assert!(status_desc.contains("behind"));
                        }
                    }
                } else {
                    assert!(status_desc.contains("no upstream configured"));
                }

                if info.has_local_changes() {
                    assert!(status_desc.contains("has local changes"));
                } else {
                    assert!(status_desc.contains("clean"));
                }
            }
            Err(_) => {
                // Could fail if git commands fail, which is acceptable in test environment
            }
        }
    }
}

#[test]
fn test_git_repository_health() {
    if let Ok(repository) = get_test_repository() {
        let result = repository.health();

        match result {
            Ok(health_status) => {
                // Test HealthStatus structure and methods
                let is_healthy = health_status.is_healthy();
                assert_eq!(is_healthy, health_status.level == HealthLevel::Healthy);

                let summary = health_status.summary();
                assert!(!summary.is_empty());

                // Test summary content based on health level
                match health_status.level {
                    HealthLevel::Healthy => {
                        assert!(summary.contains("healthy"));
                        assert!(health_status.issues.is_empty());
                        assert!(health_status.warnings.is_empty());
                    }
                    HealthLevel::Warning => {
                        assert!(summary.contains("warning"));
                        assert!(health_status.issues.is_empty());
                        assert!(!health_status.warnings.is_empty());
                    }
                    HealthLevel::Unhealthy => {
                        assert!(summary.contains("issue"));
                        assert!(!health_status.issues.is_empty());
                    }
                }

                // Test all_problems method
                let all_problems = health_status.all_problems();
                let expected_count = health_status.issues.len() + health_status.warnings.len();
                assert_eq!(all_problems.len(), expected_count);

                // Verify all_problems contains both issues and warnings
                for issue in &health_status.issues {
                    assert!(all_problems.contains(issue));
                }
                for warning in &health_status.warnings {
                    assert!(all_problems.contains(warning));
                }
            }
            Err(_) => {
                // Could fail if git commands fail, which is acceptable in test environment
            }
        }
    }
}

#[test]
fn test_git_repository_validate_for_operation_general() {
    if let Ok(repository) = get_test_repository() {
        // Test general operation validation (should pass basic git repo check)
        let result = repository.validate_for_operation("general");

        // Should either succeed (if in valid git repo) or fail (if not in git repo)
        match result {
            Ok(_) => {
                // Validation passed
            }
            Err(err) => {
                // Should be a validation error about not being in a git repo
                assert!(err.to_string().contains("git"));
            }
        }
    }
}

#[test]
fn test_git_repository_validate_for_operation_destructive() {
    if let Ok(repository) = get_test_repository() {
        // Test destructive operation validation
        let result = repository.validate_for_operation("destructive");

        match result {
            Ok(_) => {
                // Validation passed - working directory is clean
            }
            Err(err) => {
                // Could fail if working directory is not clean or not in git repo
                let error_msg = err.to_string();
                assert!(
                    error_msg.contains("clean") || error_msg.contains("git"),
                    "Error should be about cleanliness or git repo: {error_msg}"
                );
            }
        }
    }
}

#[test]
fn test_git_repository_validate_for_operation_commit() {
    if let Ok(repository) = get_test_repository() {
        // Test commit operation validation
        let result = repository.validate_for_operation("commit");

        match result {
            Ok(_) => {
                // Validation passed - there are staged changes
            }
            Err(err) => {
                // Could fail if no staged changes or not in git repo
                let error_msg = err.to_string();
                assert!(
                    error_msg.contains("staged") || error_msg.contains("git"),
                    "Error should be about staged changes or git repo: {error_msg}"
                );
            }
        }
    }
}

#[test]
fn test_git_repository_root_path() {
    if let Ok(repository) = get_test_repository() {
        let root_path = repository.root_path();
        assert!(!root_path.is_empty());

        // Root path should be an absolute path
        assert!(std::path::Path::new(root_path).is_absolute());
    }
}

// Test RepositoryInfo helper methods with constructed data

#[test]
fn test_repository_info_is_in_sync_true() {
    let info = RepositoryInfo {
        name: "test-repo".to_string(),
        root_path: "/test/path".to_string(),
        current_branch: "main".to_string(),
        upstream_branch: Some("origin/main".to_string()),
        ahead_count: 0,
        behind_count: 0,
        is_clean: true,
        staged_files_count: 0,
    };

    assert!(info.is_in_sync());
}

#[test]
fn test_repository_info_is_in_sync_false_ahead() {
    let info = RepositoryInfo {
        name: "test-repo".to_string(),
        root_path: "/test/path".to_string(),
        current_branch: "main".to_string(),
        upstream_branch: Some("origin/main".to_string()),
        ahead_count: 2,
        behind_count: 0,
        is_clean: true,
        staged_files_count: 0,
    };

    assert!(!info.is_in_sync());
}

#[test]
fn test_repository_info_is_in_sync_false_behind() {
    let info = RepositoryInfo {
        name: "test-repo".to_string(),
        root_path: "/test/path".to_string(),
        current_branch: "main".to_string(),
        upstream_branch: Some("origin/main".to_string()),
        ahead_count: 0,
        behind_count: 3,
        is_clean: true,
        staged_files_count: 0,
    };

    assert!(!info.is_in_sync());
}

#[test]
fn test_repository_info_has_local_changes_clean() {
    let info = RepositoryInfo {
        name: "test-repo".to_string(),
        root_path: "/test/path".to_string(),
        current_branch: "main".to_string(),
        upstream_branch: Some("origin/main".to_string()),
        ahead_count: 0,
        behind_count: 0,
        is_clean: true,
        staged_files_count: 0,
    };

    assert!(!info.has_local_changes());
}

#[test]
fn test_repository_info_has_local_changes_dirty() {
    let info = RepositoryInfo {
        name: "test-repo".to_string(),
        root_path: "/test/path".to_string(),
        current_branch: "main".to_string(),
        upstream_branch: Some("origin/main".to_string()),
        ahead_count: 0,
        behind_count: 0,
        is_clean: false,
        staged_files_count: 0,
    };

    assert!(info.has_local_changes());
}

#[test]
fn test_repository_info_has_local_changes_staged() {
    let info = RepositoryInfo {
        name: "test-repo".to_string(),
        root_path: "/test/path".to_string(),
        current_branch: "main".to_string(),
        upstream_branch: Some("origin/main".to_string()),
        ahead_count: 0,
        behind_count: 0,
        is_clean: true,
        staged_files_count: 2,
    };

    assert!(info.has_local_changes());
}

#[test]
fn test_repository_info_status_description_in_sync_clean() {
    let info = RepositoryInfo {
        name: "test-repo".to_string(),
        root_path: "/test/path".to_string(),
        current_branch: "main".to_string(),
        upstream_branch: Some("origin/main".to_string()),
        ahead_count: 0,
        behind_count: 0,
        is_clean: true,
        staged_files_count: 0,
    };

    let status = info.status_description();
    assert!(status.contains("up to date with origin/main"));
    assert!(status.contains("clean"));
}

#[test]
fn test_repository_info_status_description_ahead_behind() {
    let info = RepositoryInfo {
        name: "test-repo".to_string(),
        root_path: "/test/path".to_string(),
        current_branch: "feature".to_string(),
        upstream_branch: Some("origin/feature".to_string()),
        ahead_count: 3,
        behind_count: 2,
        is_clean: false,
        staged_files_count: 1,
    };

    let status = info.status_description();
    assert!(status.contains("3 ahead"));
    assert!(status.contains("2 behind"));
    assert!(status.contains("has local changes"));
}

#[test]
fn test_repository_info_status_description_no_upstream() {
    let info = RepositoryInfo {
        name: "test-repo".to_string(),
        root_path: "/test/path".to_string(),
        current_branch: "feature".to_string(),
        upstream_branch: None,
        ahead_count: 0,
        behind_count: 0,
        is_clean: true,
        staged_files_count: 0,
    };

    let status = info.status_description();
    assert!(status.contains("no upstream configured"));
    assert!(status.contains("clean"));
}

#[test]
fn test_repository_info_status_description_only_ahead() {
    let info = RepositoryInfo {
        name: "test-repo".to_string(),
        root_path: "/test/path".to_string(),
        current_branch: "feature".to_string(),
        upstream_branch: Some("origin/feature".to_string()),
        ahead_count: 5,
        behind_count: 0,
        is_clean: true,
        staged_files_count: 0,
    };

    let status = info.status_description();
    assert!(status.contains("5 ahead"));
    assert!(!status.contains("behind"));
    assert!(status.contains("clean"));
}

#[test]
fn test_repository_info_status_description_only_behind() {
    let info = RepositoryInfo {
        name: "test-repo".to_string(),
        root_path: "/test/path".to_string(),
        current_branch: "feature".to_string(),
        upstream_branch: Some("origin/feature".to_string()),
        ahead_count: 0,
        behind_count: 4,
        is_clean: true,
        staged_files_count: 0,
    };

    let status = info.status_description();
    assert!(status.contains("4 behind"));
    assert!(!status.contains("ahead"));
    assert!(status.contains("clean"));
}

// Test HealthStatus helper methods with constructed data

#[test]
fn test_health_status_is_healthy_true() {
    let health = git_x::domain::HealthStatus {
        level: HealthLevel::Healthy,
        issues: vec![],
        warnings: vec![],
    };

    assert!(health.is_healthy());
}

#[test]
fn test_health_status_is_healthy_false_warning() {
    let health = git_x::domain::HealthStatus {
        level: HealthLevel::Warning,
        issues: vec![],
        warnings: vec!["Some warning".to_string()],
    };

    assert!(!health.is_healthy());
}

#[test]
fn test_health_status_is_healthy_false_unhealthy() {
    let health = git_x::domain::HealthStatus {
        level: HealthLevel::Unhealthy,
        issues: vec!["Some issue".to_string()],
        warnings: vec![],
    };

    assert!(!health.is_healthy());
}

#[test]
fn test_health_status_summary_healthy() {
    let health = git_x::domain::HealthStatus {
        level: HealthLevel::Healthy,
        issues: vec![],
        warnings: vec![],
    };

    assert_eq!(health.summary(), "Repository is healthy");
}

#[test]
fn test_health_status_summary_warning() {
    let health = git_x::domain::HealthStatus {
        level: HealthLevel::Warning,
        issues: vec![],
        warnings: vec!["Warning 1".to_string(), "Warning 2".to_string()],
    };

    assert_eq!(health.summary(), "Repository has 2 warning(s)");
}

#[test]
fn test_health_status_summary_unhealthy() {
    let health = git_x::domain::HealthStatus {
        level: HealthLevel::Unhealthy,
        issues: vec![
            "Issue 1".to_string(),
            "Issue 2".to_string(),
            "Issue 3".to_string(),
        ],
        warnings: vec!["Warning 1".to_string()],
    };

    assert_eq!(health.summary(), "Repository has 3 issue(s)");
}

#[test]
fn test_health_status_all_problems() {
    let health = git_x::domain::HealthStatus {
        level: HealthLevel::Unhealthy,
        issues: vec!["Issue 1".to_string(), "Issue 2".to_string()],
        warnings: vec![
            "Warning 1".to_string(),
            "Warning 2".to_string(),
            "Warning 3".to_string(),
        ],
    };

    let all_problems = health.all_problems();
    assert_eq!(all_problems.len(), 5);

    // Check that all issues and warnings are included
    assert!(all_problems.contains(&"Issue 1".to_string()));
    assert!(all_problems.contains(&"Issue 2".to_string()));
    assert!(all_problems.contains(&"Warning 1".to_string()));
    assert!(all_problems.contains(&"Warning 2".to_string()));
    assert!(all_problems.contains(&"Warning 3".to_string()));
}

#[test]
fn test_health_status_all_problems_empty() {
    let health = git_x::domain::HealthStatus {
        level: HealthLevel::Healthy,
        issues: vec![],
        warnings: vec![],
    };

    let all_problems = health.all_problems();
    assert!(all_problems.is_empty());
}

#[test]
fn test_health_level_partial_eq() {
    // Test PartialEq implementation for HealthLevel
    assert_eq!(HealthLevel::Healthy, HealthLevel::Healthy);
    assert_eq!(HealthLevel::Warning, HealthLevel::Warning);
    assert_eq!(HealthLevel::Unhealthy, HealthLevel::Unhealthy);

    assert_ne!(HealthLevel::Healthy, HealthLevel::Warning);
    assert_ne!(HealthLevel::Warning, HealthLevel::Unhealthy);
    assert_ne!(HealthLevel::Healthy, HealthLevel::Unhealthy);
}

// Integration test that exercises multiple repository methods
#[test]
fn test_git_repository_integration() {
    if let Ok(repository) = get_test_repository() {
        // Test that we can call multiple methods on the same repository instance

        // Get repository info
        if let Ok(info) = repository.info() {
            assert!(!info.name.is_empty());
            assert!(!info.current_branch.is_empty());
        }

        // Get health status
        if let Ok(health) = repository.health() {
            // Health check should return some result
            let _ = health.is_healthy();
            let _ = health.summary();
        }

        // Test validation for different operations
        let _ = repository.validate_for_operation("general");
        let _ = repository.validate_for_operation("destructive");
        let _ = repository.validate_for_operation("commit");

        // Get root path
        let root_path = repository.root_path();
        assert!(!root_path.is_empty());
    }
}
