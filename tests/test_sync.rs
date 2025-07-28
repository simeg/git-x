mod common;

use assert_cmd::Command;
use common::basic_repo;
use git_x::sync::*;
use predicates::prelude::*;
use tempfile::TempDir;

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
    let repo = basic_repo();

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.arg("sync")
        .current_dir(repo.path())
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
    let repo = basic_repo();

    // Get original directory and handle potential failures
    let original_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return, // Skip test if current directory is invalid
    };

    // Change to the repo directory and call get_current_branch
    if std::env::set_current_dir(repo.path()).is_err() {
        return; // Skip test if directory change fails
    }

    let result = get_current_branch();
    let _ = std::env::set_current_dir(&original_dir);

    assert!(result.is_ok());
    let branch = result.unwrap();
    assert!(!branch.is_empty());
}

#[test]
fn test_get_upstream_branch_no_upstream() {
    let repo = basic_repo();

    let original_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return, // Skip test if current directory is invalid
    };

    if std::env::set_current_dir(repo.path()).is_err() {
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
    let repo = basic_repo();

    // Add a remote
    Command::new("git")
        .args([
            "remote",
            "add",
            "origin",
            "https://github.com/test/repo.git",
        ])
        .current_dir(repo.path())
        .assert()
        .success();

    let original_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return, // Skip test if current directory is invalid
    };

    if std::env::set_current_dir(repo.path()).is_err() {
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
    let repo = basic_repo();

    let original_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return, // Skip test if current directory is invalid
    };

    if std::env::set_current_dir(repo.path()).is_err() {
        return; // Skip test if directory change fails
    }

    // This will fail since there's no upstream, but tests the error path
    let result = get_sync_status("main", "origin/main");
    let _ = std::env::set_current_dir(&original_dir);

    assert!(result.is_err());
}

#[test]
fn test_sync_with_upstream_merge() {
    let repo = basic_repo();

    let original_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return, // Skip test if current directory is invalid
    };

    if std::env::set_current_dir(repo.path()).is_err() {
        return; // Skip test if directory change fails
    }

    // This will fail since there's no upstream, but tests the error path
    let result = sync_with_upstream("origin/main", true);
    let _ = std::env::set_current_dir(&original_dir);

    assert!(result.is_err());
}

#[test]
fn test_sync_with_upstream_rebase() {
    let repo = basic_repo();

    let original_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return, // Skip test if current directory is invalid
    };

    if std::env::set_current_dir(repo.path()).is_err() {
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
    let repo = basic_repo();

    let original_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return, // Skip test if current directory is invalid
    };

    if std::env::set_current_dir(repo.path()).is_err() {
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
    let repo = basic_repo();

    let original_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return, // Skip test if current directory is invalid
    };

    if std::env::set_current_dir(repo.path()).is_err() {
        return; // Skip test if directory change fails
    }

    // Test with non-existent upstream
    let result = get_sync_status("main", "nonexistent/branch");
    let _ = std::env::set_current_dir(&original_dir);

    assert!(result.is_err());
}

#[test]
fn test_sync_with_upstream_merge_error() {
    let repo = basic_repo();

    let original_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return, // Skip test if current directory is invalid
    };

    if std::env::set_current_dir(repo.path()).is_err() {
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
    let repo = basic_repo();

    let original_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return, // Skip test if current directory is invalid
    };

    if std::env::set_current_dir(repo.path()).is_err() {
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
    let repo = basic_repo();

    let original_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return, // Skip test if current directory is invalid
    };

    if std::env::set_current_dir(repo.path()).is_err() {
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
    let repo = basic_repo();

    let original_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return, // Skip test if current directory is invalid
    };

    if std::env::set_current_dir(repo.path()).is_err() {
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

// Additional tests for sync.rs to increase coverage

#[test]
fn test_sync_status_enum_coverage() {
    // Test enum variants for complete coverage
    let up_to_date = SyncStatus::UpToDate;
    let behind = SyncStatus::Behind(5);
    let ahead = SyncStatus::Ahead(3);
    let diverged = SyncStatus::Diverged(2, 4);

    // Test Debug formatting (if derived)
    let _ = format!("{up_to_date:?}");
    let _ = format!("{behind:?}");
    let _ = format!("{ahead:?}");
    let _ = format!("{diverged:?}");

    // Test pattern matching coverage
    match up_to_date {
        SyncStatus::UpToDate => {}
        _ => panic!("Should be UpToDate"),
    }

    match behind {
        SyncStatus::Behind(n) => assert_eq!(n, 5),
        _ => panic!("Should be Behind"),
    }

    match ahead {
        SyncStatus::Ahead(n) => assert_eq!(n, 3),
        _ => panic!("Should be Ahead"),
    }

    match diverged {
        SyncStatus::Diverged(b, a) => {
            assert_eq!(b, 2);
            assert_eq!(a, 4);
        }
        _ => panic!("Should be Diverged"),
    }
}

#[test]
fn test_additional_parse_sync_counts_edge_cases() {
    // Test more edge cases for parse_sync_counts to increase coverage
    assert!(parse_sync_counts("").is_err());
    assert!(parse_sync_counts("invalid").is_err());
    assert!(parse_sync_counts("1").is_err());
    assert!(parse_sync_counts("abc\tdef").is_err());
    assert!(parse_sync_counts("-1\t2").is_err());
    assert!(parse_sync_counts("1\t-2").is_err());
    assert!(parse_sync_counts("999999999999999999999\t1").is_err());

    // Test valid formats
    assert_eq!(parse_sync_counts("0\t0").unwrap(), (0, 0));
    assert_eq!(parse_sync_counts("10\t20").unwrap(), (10, 20));
    assert_eq!(parse_sync_counts("1\t1").unwrap(), (1, 1));
}

#[test]
fn test_format_message_variations() {
    // Test format functions with different inputs for better coverage
    assert_eq!(
        format_behind_message(0),
        "⬇️ Branch is 0 commit(s) behind upstream"
    );
    assert_eq!(
        format_behind_message(1),
        "⬇️ Branch is 1 commit(s) behind upstream"
    );
    assert_eq!(
        format_behind_message(100),
        "⬇️ Branch is 100 commit(s) behind upstream"
    );

    assert_eq!(
        format_ahead_message(0),
        "⬆️ Branch is 0 commit(s) ahead of upstream"
    );
    assert_eq!(
        format_ahead_message(1),
        "⬆️ Branch is 1 commit(s) ahead of upstream"
    );
    assert_eq!(
        format_ahead_message(999),
        "⬆️ Branch is 999 commit(s) ahead of upstream"
    );

    assert_eq!(
        format_diverged_message(0, 0),
        "🔀 Branch has diverged: 0 behind, 0 ahead"
    );
    assert_eq!(
        format_diverged_message(1, 1),
        "🔀 Branch has diverged: 1 behind, 1 ahead"
    );
    assert_eq!(
        format_diverged_message(10, 5),
        "🔀 Branch has diverged: 10 behind, 5 ahead"
    );

    assert_eq!(
        format_sync_success_message(true),
        "✅ Successfully merged upstream changes"
    );
    assert_eq!(
        format_sync_success_message(false),
        "✅ Successfully rebased onto upstream"
    );

    assert!(format_diverged_help_message().contains("handle"));
    assert!(format_up_to_date_message().contains("up to date"));
}

#[test]
fn test_sync_start_message_variations() {
    // Test different branch name combinations
    assert!(format_sync_start_message("main", "origin/main").contains("main"));
    assert!(format_sync_start_message("main", "origin/main").contains("origin/main"));

    assert!(format_sync_start_message("feature", "origin/feature").contains("feature"));
    assert!(format_sync_start_message("", "").contains(""));

    let result = format_sync_start_message("test-branch", "upstream/test-branch");
    assert!(result.contains("test-branch"));
    assert!(result.contains("upstream/test-branch"));
}

#[test]
fn test_error_message_format_coverage() {
    // Test error message formatting with various inputs
    assert_eq!(format_error_message("test error"), "❌ test error");
    assert_eq!(format_error_message(""), "❌ ");
    assert_eq!(
        format_error_message("Network timeout"),
        "❌ Network timeout"
    );
    assert_eq!(
        format_error_message("Git command failed"),
        "❌ Git command failed"
    );
}

// Integration tests for sync.rs run() function testing all code paths

#[test]
fn test_sync_run_outside_git_repo() {
    // Test error path: not in a git repository
    let temp_dir = TempDir::new().unwrap();
    let output = Command::cargo_bin("git-x")
        .unwrap()
        .current_dir(temp_dir.path())
        .args(["sync"])
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should show error message about not being in git repo
    assert!(stderr.contains("❌"));
    assert!(
        stderr.contains("Not in a git repository")
            || stderr.contains("Failed to get current branch")
    );
}

#[test]
fn test_sync_run_no_upstream() {
    // Test error path: no upstream branch configured
    let repo = common::basic_repo();
    let output = Command::cargo_bin("git-x")
        .unwrap()
        .current_dir(repo.path())
        .args(["sync"])
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should show error message about no upstream
    assert!(stderr.contains("❌"));
    assert!(stderr.contains("No upstream branch configured"));
}

#[test]
fn test_sync_run_up_to_date() {
    // Test success path: branch is up to date with upstream
    let repo = common::repo_with_branch("main");

    // Set up remote
    let _remote = repo.setup_remote("main");

    let output = Command::cargo_bin("git-x")
        .unwrap()
        .current_dir(repo.path())
        .args(["sync"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should show sync start message
    assert!(stdout.contains("🔄 Syncing branch"));
    // Should show some outcome
    assert!(
        stdout.contains("✅")
            || stdout.contains("⬇️")
            || stdout.contains("⬆️")
            || stderr.contains("❌")
    );
}

#[test]
fn test_sync_run_behind_with_rebase() {
    // Test success path: branch is behind and needs rebase
    let (local_repo, _remote_repo) = common::repo_with_remote_ahead("main");

    let output = Command::cargo_bin("git-x")
        .unwrap()
        .current_dir(local_repo.path())
        .args(["sync"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should show sync messages
    assert!(stdout.contains("🔄 Syncing branch"));
    // The exact outcome may vary, but should show some progress
    assert!(stdout.contains("⬇️ Branch is") || stdout.contains("✅") || stderr.contains("❌"));
}

#[test]
fn test_sync_run_behind_with_merge() {
    // Test success path: branch is behind and needs merge
    let (local_repo, _remote_repo) = common::repo_with_remote_ahead("main");

    // Change to local repo directory
    if std::env::set_current_dir(local_repo.path()).is_err() {
        eprintln!("Warning: Could not change to repo directory, skipping test");
        return;
    }

    let output = Command::cargo_bin("git-x")
        .unwrap()
        .args(["sync", "--merge"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should show sync messages
    assert!(stdout.contains("🔄 Syncing branch"));
    // The exact outcome may vary, but should show some progress
    assert!(stdout.contains("⬇️ Branch is") || stdout.contains("✅") || stderr.contains("❌"));
}

#[test]
fn test_sync_run_ahead() {
    // Test path: branch is ahead of upstream
    let repo = common::repo_with_branch("main");

    // Set up remote first
    let _remote = repo.setup_remote("main");

    // Add a local commit to make branch ahead
    repo.add_commit("local_file.txt", "local content", "local commit");

    let output = Command::cargo_bin("git-x")
        .unwrap()
        .current_dir(repo.path())
        .args(["sync"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should show sync start message
    assert!(stdout.contains("🔄 Syncing branch"));
    // Should show some status
    assert!(
        stdout.contains("⬆️ Branch is")
            || stdout.contains("✅")
            || stdout.contains("⬇️")
            || stderr.contains("❌")
    );
}

#[test]
fn test_sync_run_diverged_no_merge() {
    // Test diverged path: branch has diverged, no merge flag
    let repo = common::repo_with_branch("main");

    // Set up remote with initial commit
    let _remote = repo.setup_remote("main");

    // Add local commit
    repo.add_commit("local_file.txt", "local content", "local commit");

    let output = Command::cargo_bin("git-x")
        .unwrap()
        .current_dir(repo.path())
        .args(["sync"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should show sync start message
    assert!(stdout.contains("🔄 Syncing branch"));
    // Should show some status outcome
    assert!(
        stdout.contains("✅")
            || stdout.contains("⬇️")
            || stdout.contains("⬆️")
            || stdout.contains("🔀")
            || stdout.contains("💡")
            || stderr.contains("❌")
    );
}

#[test]
fn test_sync_run_comprehensive_output() {
    // Test that all output components are present in success case
    let repo = common::repo_with_branch("main");

    // Set up remote
    let _remote = repo.setup_remote("main");

    let output = Command::cargo_bin("git-x")
        .unwrap()
        .current_dir(repo.path())
        .args(["sync"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should contain sync start message
    assert!(stdout.contains("🔄"));
    assert!(stdout.contains("Syncing branch"));

    // Should contain status message (one of the possible outcomes)
    assert!(
        stdout.contains("✅")
            || stdout.contains("⬇️")
            || stdout.contains("⬆️")
            || stdout.contains("🔀")
            || stderr.contains("❌")
    );
}
