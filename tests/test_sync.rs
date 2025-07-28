use assert_cmd::Command;
use git_x::sync::*;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// Helper function to create a test git repository
fn create_test_repo() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let repo_path = temp_dir.path().to_path_buf();

    // Initialize git repo
    Command::new("git")
        .args(["init"])
        .current_dir(&repo_path)
        .assert()
        .success();

    // Configure git
    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(&repo_path)
        .assert()
        .success();

    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(&repo_path)
        .assert()
        .success();

    // Create initial commit
    fs::write(repo_path.join("README.md"), "Initial commit").expect("Failed to write file");
    Command::new("git")
        .args(["add", "README.md"])
        .current_dir(&repo_path)
        .assert()
        .success();

    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(&repo_path)
        .assert()
        .success();

    (temp_dir, repo_path)
}

#[test]
fn test_parse_sync_counts() {
    assert_eq!(parse_sync_counts("0\t0"), Ok((0, 0)));
    assert_eq!(parse_sync_counts("1\t0"), Ok((1, 0)));
    assert_eq!(parse_sync_counts("0\t2"), Ok((0, 2)));
    assert_eq!(parse_sync_counts("3\t5"), Ok((3, 5)));
    assert_eq!(parse_sync_counts("10\t20"), Ok((10, 20)));
}

#[test]
fn test_parse_sync_counts_invalid() {
    assert!(parse_sync_counts("").is_err());
    assert!(parse_sync_counts("abc").is_err());
    assert!(parse_sync_counts("1").is_err());
    assert!(parse_sync_counts("1\tabc").is_err());
}

#[test]
fn test_format_sync_start_message() {
    assert_eq!(
        format_sync_start_message("main", "origin/main"),
        "🔄 Syncing branch 'main' with 'origin/main'..."
    );
    assert_eq!(
        format_sync_start_message("feature", "upstream/develop"),
        "🔄 Syncing branch 'feature' with 'upstream/develop'..."
    );
}

#[test]
fn test_format_error_message() {
    assert_eq!(format_error_message("Test error"), "❌ Test error");
    assert_eq!(
        format_error_message("Connection failed"),
        "❌ Connection failed"
    );
}

#[test]
fn test_format_up_to_date_message() {
    assert_eq!(
        format_up_to_date_message(),
        "✅ Branch is up to date with upstream"
    );
}

#[test]
fn test_format_behind_message() {
    assert_eq!(
        format_behind_message(1),
        "⬇️ Branch is 1 commit(s) behind upstream"
    );
    assert_eq!(
        format_behind_message(5),
        "⬇️ Branch is 5 commit(s) behind upstream"
    );
}

#[test]
fn test_format_ahead_message() {
    assert_eq!(
        format_ahead_message(1),
        "⬆️ Branch is 1 commit(s) ahead of upstream"
    );
    assert_eq!(
        format_ahead_message(3),
        "⬆️ Branch is 3 commit(s) ahead of upstream"
    );
}

#[test]
fn test_format_diverged_message() {
    assert_eq!(
        format_diverged_message(2, 3),
        "🔀 Branch has diverged: 2 behind, 3 ahead"
    );
    assert_eq!(
        format_diverged_message(1, 1),
        "🔀 Branch has diverged: 1 behind, 1 ahead"
    );
}

#[test]
fn test_format_diverged_help_message() {
    assert_eq!(
        format_diverged_help_message(),
        "💡 Use --merge flag to merge changes, or handle manually"
    );
}

#[test]
fn test_format_sync_success_message() {
    assert_eq!(
        format_sync_success_message(true),
        "✅ Successfully merged upstream changes"
    );
    assert_eq!(
        format_sync_success_message(false),
        "✅ Successfully rebased onto upstream"
    );
}

#[test]
fn test_sync_run_function_outside_git_repo() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.arg("sync")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("Not in a git repository"));
}

#[test]
fn test_sync_run_function_no_upstream() {
    let (_temp_dir, repo_path) = create_test_repo();

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.arg("sync")
        .current_dir(&repo_path)
        .assert()
        .success()
        .stderr(predicate::str::contains("No upstream branch configured"));
}

#[test]
fn test_sync_command_help() {
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["sync", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Sync current branch with upstream",
        ));
}

#[test]
fn test_sync_merge_flag() {
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["sync", "--merge", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Use merge instead of rebase"));
}

#[test]
fn test_get_current_branch_success() {
    let (_temp_dir, repo_path) = create_test_repo();

    // Get original directory and handle potential failures
    let original_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return, // Skip test if current directory is invalid
    };

    // Change to the repo directory and call get_current_branch
    if std::env::set_current_dir(&repo_path).is_err() {
        return; // Skip test if directory change fails
    }

    let result = get_current_branch();
    let _ = std::env::set_current_dir(&original_dir);

    assert!(result.is_ok());
    let branch = result.unwrap();
    assert!(!branch.is_empty());
}

#[test]
fn test_get_current_branch_not_git_repo() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Create a completely isolated directory that definitely isn't a git repo
    let isolated_dir = temp_dir.path().join("isolated");
    std::fs::create_dir(&isolated_dir).expect("Failed to create isolated directory");

    // Use a wrapper to test the function in isolation without changing global state
    let result = std::thread::spawn(move || {
        // Unset GIT_DIR and GIT_WORK_TREE to ensure git doesn't find parent repos
        unsafe {
            std::env::remove_var("GIT_DIR");
            std::env::remove_var("GIT_WORK_TREE");
        }

        // Change directory in this isolated thread
        if std::env::set_current_dir(&isolated_dir).is_err() {
            return Err("Failed to change directory");
        }

        get_current_branch()
    })
    .join()
    .expect("Thread should not panic");

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Not in a git repository");
}

#[test]
fn test_get_upstream_branch_no_upstream() {
    let (_temp_dir, repo_path) = create_test_repo();

    let original_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return, // Skip test if current directory is invalid
    };

    if std::env::set_current_dir(&repo_path).is_err() {
        return; // Skip test if directory change fails
    }

    let result = get_upstream_branch("main");
    let _ = std::env::set_current_dir(&original_dir);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "No upstream branch configured");
}

#[test]
fn test_sync_status_enum_equality() {
    assert_eq!(SyncStatus::UpToDate, SyncStatus::UpToDate);
    assert_eq!(SyncStatus::Behind(5), SyncStatus::Behind(5));
    assert_eq!(SyncStatus::Ahead(3), SyncStatus::Ahead(3));
    assert_eq!(SyncStatus::Diverged(2, 4), SyncStatus::Diverged(2, 4));

    assert_ne!(SyncStatus::Behind(1), SyncStatus::Behind(2));
    assert_ne!(SyncStatus::Ahead(1), SyncStatus::Ahead(2));
    assert_ne!(SyncStatus::UpToDate, SyncStatus::Behind(1));
}

#[test]
fn test_sync_status_debug() {
    let status = SyncStatus::Diverged(3, 5);
    let debug_str = format!("{status:?}");
    assert!(debug_str.contains("Diverged"));
    assert!(debug_str.contains("3"));
    assert!(debug_str.contains("5"));
}

#[test]
fn test_fetch_upstream_success() {
    let (_temp_dir, repo_path) = create_test_repo();

    // Add a remote
    Command::new("git")
        .args([
            "remote",
            "add",
            "origin",
            "https://github.com/test/repo.git",
        ])
        .current_dir(&repo_path)
        .assert()
        .success();

    let original_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return, // Skip test if current directory is invalid
    };

    if std::env::set_current_dir(&repo_path).is_err() {
        return; // Skip test if directory change fails
    }

    // Test fetch with invalid remote (will fail but tests the error path)
    let result = fetch_upstream("origin/main");
    let _ = std::env::set_current_dir(&original_dir);

    // Should fail because remote doesn't exist, but tests the function
    assert!(result.is_err());
}

#[test]
fn test_get_sync_status_patterns() {
    let (_temp_dir, repo_path) = create_test_repo();

    let original_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return, // Skip test if current directory is invalid
    };

    if std::env::set_current_dir(&repo_path).is_err() {
        return; // Skip test if directory change fails
    }

    // This will fail since there's no upstream, but tests the error path
    let result = get_sync_status("main", "origin/main");
    let _ = std::env::set_current_dir(&original_dir);

    assert!(result.is_err());
}

#[test]
fn test_sync_with_upstream_merge() {
    let (_temp_dir, repo_path) = create_test_repo();

    let original_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return, // Skip test if current directory is invalid
    };

    if std::env::set_current_dir(&repo_path).is_err() {
        return; // Skip test if directory change fails
    }

    // This will fail since there's no upstream, but tests the error path
    let result = sync_with_upstream("origin/main", true);
    let _ = std::env::set_current_dir(&original_dir);

    assert!(result.is_err());
}

#[test]
fn test_sync_with_upstream_rebase() {
    let (_temp_dir, repo_path) = create_test_repo();

    let original_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return, // Skip test if current directory is invalid
    };

    if std::env::set_current_dir(&repo_path).is_err() {
        return; // Skip test if directory change fails
    }

    // This will fail since there's no upstream, but tests the error path
    let result = sync_with_upstream("origin/main", false);
    let _ = std::env::set_current_dir(&original_dir);

    assert!(result.is_err());
}

#[test]
fn test_run_function_complete_flow() {
    // Simple test that verifies the main run function executes without crashing
    // when called from outside a git repository (error path)
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.arg("sync")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("Not in a git repository"));
}

// Additional comprehensive tests for better coverage

#[test]
fn test_parse_sync_counts_comprehensive() {
    // Test various valid formats
    assert_eq!(parse_sync_counts("0\t0\n"), Ok((0, 0)));
    assert_eq!(parse_sync_counts("5\t0"), Ok((5, 0)));
    assert_eq!(parse_sync_counts("0\t3"), Ok((0, 3)));
    assert_eq!(parse_sync_counts("2\t4"), Ok((2, 4)));
    assert_eq!(parse_sync_counts("10\t20\n"), Ok((10, 20)));

    // Test with extra whitespace
    assert_eq!(parse_sync_counts("  1  \t  2  "), Ok((1, 2)));
    assert_eq!(parse_sync_counts("100\t200\n\n"), Ok((100, 200)));
}

#[test]
fn test_parse_sync_counts_edge_cases() {
    // Test various invalid formats
    assert!(parse_sync_counts("").is_err());
    assert!(parse_sync_counts("abc\tdef").is_err());
    assert!(parse_sync_counts("1").is_err());
    assert!(parse_sync_counts("1\t").is_err());
    assert!(parse_sync_counts("\t2").is_err());
    assert!(parse_sync_counts("1\tabc").is_err());
    assert!(parse_sync_counts("abc\t2").is_err());
    assert!(parse_sync_counts("1\t2\t3").is_ok()); // Should still work, extra ignored
}

#[test]
fn test_fetch_upstream_remote_parsing() {
    let (_temp_dir, repo_path) = create_test_repo();

    let original_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return, // Skip test if current directory is invalid
    };

    if std::env::set_current_dir(&repo_path).is_err() {
        return; // Skip test if directory change fails
    }

    // Test various upstream formats
    let result1 = fetch_upstream("origin/main");
    let result2 = fetch_upstream("upstream/develop");
    let result3 = fetch_upstream("fork/feature");

    let _ = std::env::set_current_dir(&original_dir);

    // All should fail since remotes don't exist, but tests the parsing logic
    assert!(result1.is_err());
    assert!(result2.is_err());
    assert!(result3.is_err());
}

#[test]
fn test_get_sync_status_error_scenarios() {
    let (_temp_dir, repo_path) = create_test_repo();

    let original_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return, // Skip test if current directory is invalid
    };

    if std::env::set_current_dir(&repo_path).is_err() {
        return; // Skip test if directory change fails
    }

    // Test with non-existent upstream
    let result = get_sync_status("main", "nonexistent/branch");
    let _ = std::env::set_current_dir(&original_dir);

    assert!(result.is_err());
}

#[test]
fn test_sync_with_upstream_merge_error() {
    let (_temp_dir, repo_path) = create_test_repo();

    let original_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return, // Skip test if current directory is invalid
    };

    if std::env::set_current_dir(&repo_path).is_err() {
        return; // Skip test if directory change fails
    }

    // Test merge with non-existent upstream (should fail)
    let result = sync_with_upstream("nonexistent/branch", true);
    let _ = std::env::set_current_dir(&original_dir);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Merge failed");
}

#[test]
fn test_sync_with_upstream_rebase_error() {
    let (_temp_dir, repo_path) = create_test_repo();

    let original_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return, // Skip test if current directory is invalid
    };

    if std::env::set_current_dir(&repo_path).is_err() {
        return; // Skip test if directory change fails
    }

    // Test rebase with non-existent upstream (should fail)
    let result = sync_with_upstream("nonexistent/branch", false);
    let _ = std::env::set_current_dir(&original_dir);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Rebase failed");
}

#[test]
fn test_sync_status_enum_all_variants() {
    // Test PartialEq for all variants
    assert_eq!(SyncStatus::UpToDate, SyncStatus::UpToDate);
    assert_eq!(SyncStatus::Behind(5), SyncStatus::Behind(5));
    assert_eq!(SyncStatus::Ahead(3), SyncStatus::Ahead(3));
    assert_eq!(SyncStatus::Diverged(2, 4), SyncStatus::Diverged(2, 4));

    // Test inequality
    assert_ne!(SyncStatus::Behind(1), SyncStatus::Behind(2));
    assert_ne!(SyncStatus::Ahead(1), SyncStatus::Ahead(2));
    assert_ne!(SyncStatus::UpToDate, SyncStatus::Behind(1));
    assert_ne!(SyncStatus::Diverged(1, 2), SyncStatus::Diverged(2, 1));
}

#[test]
fn test_sync_status_debug_all_variants() {
    let variants = vec![
        SyncStatus::UpToDate,
        SyncStatus::Behind(5),
        SyncStatus::Ahead(3),
        SyncStatus::Diverged(2, 4),
    ];

    for variant in variants {
        let debug_str = format!("{variant:?}");
        assert!(!debug_str.is_empty());
        // Ensure debug contains the variant name
        assert!(
            debug_str.contains("UpToDate")
                || debug_str.contains("Behind")
                || debug_str.contains("Ahead")
                || debug_str.contains("Diverged")
        );
    }
}

#[test]
fn test_comprehensive_formatting_functions() {
    // Test all formatting functions with various inputs
    assert_eq!(
        format_sync_start_message("feature-branch", "origin/main"),
        "🔄 Syncing branch 'feature-branch' with 'origin/main'..."
    );

    assert_eq!(
        format_error_message("Custom error message"),
        "❌ Custom error message"
    );

    assert_eq!(
        format_behind_message(0),
        "⬇️ Branch is 0 commit(s) behind upstream"
    );

    assert_eq!(
        format_ahead_message(0),
        "⬆️ Branch is 0 commit(s) ahead of upstream"
    );

    assert_eq!(
        format_diverged_message(0, 0),
        "🔀 Branch has diverged: 0 behind, 0 ahead"
    );

    assert_eq!(
        format_sync_success_message(true),
        "✅ Successfully merged upstream changes"
    );

    assert_eq!(
        format_sync_success_message(false),
        "✅ Successfully rebased onto upstream"
    );
}

#[test]
fn test_fetch_upstream_edge_cases() {
    let (_temp_dir, repo_path) = create_test_repo();

    let original_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return, // Skip test if current directory is invalid
    };

    if std::env::set_current_dir(&repo_path).is_err() {
        return; // Skip test if directory change fails
    }

    // Test edge cases for upstream parsing
    let result1 = fetch_upstream("/main"); // Should use "" as remote
    let result2 = fetch_upstream("main"); // No slash, should use "main" as remote

    let _ = std::env::set_current_dir(&original_dir);

    // These should fail since remotes don't exist, but tests the parsing logic
    assert!(result2.is_err()); // "main" remote doesn't exist

    // result1 behavior depends on git's handling of empty remote names - just test it doesn't panic
    let _ = result1;
}

#[test]
fn test_get_upstream_branch_error_path() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let original_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return, // Skip test if current directory is invalid
    };

    if std::env::set_current_dir(temp_dir.path()).is_err() {
        return; // Skip test if directory change fails
    }

    let result = get_upstream_branch("main");
    let _ = std::env::set_current_dir(&original_dir);

    assert!(result.is_err());
}

#[test]
fn test_get_current_branch_comprehensive() {
    // Test successful case
    let (_temp_dir, repo_path) = create_test_repo();

    let original_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return, // Skip test if current directory is invalid
    };

    if std::env::set_current_dir(&repo_path).is_err() {
        return; // Skip test if directory change fails
    }

    let result_success = get_current_branch();
    let _ = std::env::set_current_dir(&original_dir);

    assert!(result_success.is_ok());
    let branch = result_success.unwrap();
    assert!(!branch.is_empty());

    // Test error case
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Create a completely isolated directory that definitely isn't a git repo
    let isolated_dir = temp_dir.path().join("isolated");
    std::fs::create_dir(&isolated_dir).expect("Failed to create isolated directory");

    // Unset GIT_DIR and GIT_WORK_TREE to ensure git doesn't find parent repos
    let original_git_dir = std::env::var("GIT_DIR").ok();
    let original_git_work_tree = std::env::var("GIT_WORK_TREE").ok();
    unsafe {
        std::env::remove_var("GIT_DIR");
        std::env::remove_var("GIT_WORK_TREE");
    }

    let original_dir_2 = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return, // Skip test if current directory is invalid
    };

    if std::env::set_current_dir(&isolated_dir).is_err() {
        return; // Skip test if directory change fails
    }

    let result_error = get_current_branch();

    // Restore original directory and environment
    let _ = std::env::set_current_dir(&original_dir_2);
    unsafe {
        if let Some(git_dir) = original_git_dir {
            std::env::set_var("GIT_DIR", git_dir);
        }
        if let Some(git_work_tree) = original_git_work_tree {
            std::env::set_var("GIT_WORK_TREE", git_work_tree);
        }
    }

    assert!(result_error.is_err());
    assert_eq!(result_error.unwrap_err(), "Not in a git repository");
}
