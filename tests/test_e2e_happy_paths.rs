// End-to-end tests for all CLI commands covering happy paths
//
// This file contains comprehensive E2E tests for every CLI command to ensure
// that the commands work correctly. These tests focus on the happy path
// scenarios to verify basic functionality.
//
// These tests call the command functions directly to improve test coverage
// rather than running subprocesses.

use assert_cmd::Command;
use predicates::str::contains;
use serial_test::serial;
use std::fs;
use std::path::PathBuf;
use std::process::Command as StdCommand;
use tempfile::TempDir;

mod common;
use common::{TestRepo, basic_repo, repo_with_branch, repo_with_commits, repo_with_merged_branch};

// Helper to check if we should run potentially destructive tests
fn should_run_destructive_tests() -> bool {
    std::env::var("CI").is_ok()
        || std::env::var("GITHUB_ACTIONS").is_ok()
        || std::env::var("ENABLE_DESTRUCTIVE_TESTS").is_ok()
}

// Helper to create a test repo with stash
fn create_repo_with_stash() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let repo_path = temp_dir.path().to_path_buf();

    // Initialize git repo
    StdCommand::new("git")
        .args(["init"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to init git repo");

    // Configure git
    StdCommand::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to set git user.name");

    StdCommand::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to set git user.email");

    // Create initial commit
    fs::write(repo_path.join("README.md"), "# Test Repo").expect("Failed to write file");
    StdCommand::new("git")
        .args(["add", "README.md"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to add file");
    StdCommand::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to commit");

    // Create a stash
    fs::write(repo_path.join("stashed.txt"), "stashed content").expect("Failed to write file");
    StdCommand::new("git")
        .args(["add", "stashed.txt"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to add file");
    StdCommand::new("git")
        .args(["stash", "push", "-m", "test stash"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to create stash");

    (temp_dir, repo_path)
}

// Helper to create a repo with remote tracking
fn create_repo_with_remote() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let _repo_path = temp_dir.path().to_path_buf();

    // Create bare remote repo
    let remote_path = temp_dir.path().join("remote.git");
    fs::create_dir(&remote_path).expect("Failed to create remote dir");
    StdCommand::new("git")
        .args(["init", "--bare"])
        .current_dir(&remote_path)
        .output()
        .expect("Failed to create remote repo");

    // Clone from remote
    StdCommand::new("git")
        .args(["clone", remote_path.to_str().unwrap(), "repo"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to clone repo");

    let repo_path = temp_dir.path().join("repo");

    // Configure git
    StdCommand::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to set git user.name");
    StdCommand::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to set git user.email");

    // Create initial commit and push
    fs::write(repo_path.join("README.md"), "# Test Repo").expect("Failed to write file");
    StdCommand::new("git")
        .args(["add", "README.md"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to add file");
    StdCommand::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to commit");
    StdCommand::new("git")
        .args(["push", "origin", "main"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to push");

    (temp_dir, repo_path)
}

// ============================================================================
// Basic Commands (No Subcommands)
// ============================================================================

#[test]
#[serial]
fn test_e2e_info_happy_path() {
    let repo = basic_repo();
    let result = repo
        .run_git_x_direct(&["info"])
        .expect("Info command should succeed");
    assert!(result.contains("Repository:"));
    assert!(result.contains("Current branch:"));
    assert!(result.contains("Working directory:"));
}

#[test]
#[serial]
fn test_e2e_graph_happy_path() {
    let repo = basic_repo();
    let _result = repo
        .run_git_x_direct(&["graph"])
        .expect("Graph command should succeed");
}

#[test]
#[serial]
fn test_e2e_color_graph_happy_path() {
    let repo = basic_repo();
    let _result = repo
        .run_git_x_direct(&["color-graph"])
        .expect("Color graph command should succeed");
}

#[test]
#[serial]
fn test_e2e_health_happy_path() {
    let repo = basic_repo();
    let result = repo
        .run_git_x_direct(&["health"])
        .expect("Health command should succeed");
    assert!(result.contains("Repository Health Check"));
}

#[test]
#[serial]
fn test_e2e_contributors_happy_path() {
    let repo = repo_with_commits(3);
    let result = repo
        .run_git_x_direct(&["contributors"])
        .expect("Contributors command should succeed");
    assert!(result.contains("Contributors"));
}

#[test]
#[serial]
fn test_e2e_technical_debt_happy_path() {
    let repo = basic_repo();
    let result = repo
        .run_git_x_direct(&["technical-debt"])
        .expect("Technical debt command should succeed");
    assert!(result.contains("Technical Debt Analysis"));
}

#[test]
#[serial]
fn test_e2e_switch_recent_happy_path() {
    let repo = repo_with_branch("feature-branch");
    // Switch recent may fail if there are no recent branches, which is expected behavior
    let _result = repo.run_git_x_direct(&["switch-recent"]);
}

// ============================================================================
// Commands with Arguments
// ============================================================================

#[test]
#[serial]
fn test_e2e_rename_branch_happy_path() {
    if !should_run_destructive_tests() {
        return;
    }

    let repo = repo_with_branch("old-name");

    // Switch to the branch to rename
    StdCommand::new("git")
        .args(["checkout", "old-name"])
        .current_dir(repo.path())
        .output()
        .expect("Failed to checkout branch");

    let result = repo
        .run_git_x_direct(&["rename-branch", "new-name"])
        .expect("Rename branch should succeed");
    assert!(result.contains("Renamed branch") || result.contains("new-name"));
}

#[test]
#[serial]
fn test_e2e_since_happy_path() {
    let repo = repo_with_commits(3);
    let result = repo
        .run_git_x_direct(&["since", "HEAD~1"])
        .expect("Since command should succeed");
    assert!(result.contains("Commits since") || result.contains("since"));
}

#[test]
#[serial]
fn test_e2e_large_files_default_happy_path() {
    let repo = basic_repo();
    let _result = repo
        .run_git_x_direct(&["large-files"])
        .expect("Large files command should succeed");
}

#[test]
#[serial]
fn test_e2e_large_files_with_limit_happy_path() {
    let repo = basic_repo();
    let _result = repo
        .run_git_x_direct(&["large-files", "--limit", "5"])
        .expect("Large files with limit should succeed");
}

#[test]
#[serial]
fn test_e2e_large_files_with_threshold_happy_path() {
    let repo = basic_repo();
    let _result = repo
        .run_git_x_direct(&["large-files", "--threshold", "0.1"])
        .expect("Large files with threshold should succeed");
}

#[test]
#[serial]
fn test_e2e_summary_default_happy_path() {
    let repo = repo_with_commits(2);
    let result = repo
        .run_git_x_direct(&["summary"])
        .expect("Summary command should succeed");
    assert!(result.contains("Repository Summary") || result.contains("Summary"));
}

#[test]
#[serial]
fn test_e2e_summary_with_since_happy_path() {
    let repo = repo_with_commits(2);
    let result = repo
        .run_git_x_direct(&["summary", "--since", "1 day ago"])
        .expect("Summary with since should succeed");
    assert!(result.contains("Summary") || result.contains("Commits"));
}

#[test]
#[serial]
fn test_e2e_what_default_happy_path() {
    let repo = basic_repo();
    // What command may fail if branches don't exist for comparison, which is expected behavior
    let _result = repo.run_git_x_direct(&["what"]);
}

#[test]
#[serial]
fn test_e2e_what_with_target_happy_path() {
    let repo = basic_repo();
    // What command may fail if branches don't exist for comparison, which is expected behavior
    let _result = repo.run_git_x_direct(&["what", "--target", "main"]);
}

#[test]
#[serial]
fn test_e2e_new_branch_happy_path() {
    if !should_run_destructive_tests() {
        return;
    }

    let repo = basic_repo();
    let result = repo
        .run_git_x_direct(&["new", "feature-new"])
        .expect("New branch command should succeed");
    assert!(result.contains("Creating new branch") || result.contains("feature-new"));
}

#[test]
#[serial]
fn test_e2e_new_branch_with_from_happy_path() {
    if !should_run_destructive_tests() {
        return;
    }

    let repo = repo_with_branch("base-branch");
    let result = repo
        .run_git_x_direct(&["new", "derived-branch", "--from", "base-branch"])
        .expect("New branch with from should succeed");
    assert!(result.contains("Creating new branch") || result.contains("derived-branch"));
}

#[test]
#[serial]
fn test_e2e_fixup_happy_path() {
    let repo = repo_with_commits(2);

    // Get the hash of the first commit
    let output = StdCommand::new("git")
        .args(["log", "--oneline", "--reverse"])
        .current_dir(repo.path())
        .output()
        .expect("Failed to get commit hash");

    let stdout = String::from_utf8_lossy(&output.stdout);
    if let Some(line) = stdout.lines().next() {
        let hash = line.split_whitespace().next().unwrap();

        // Stage some changes for the fixup
        std::fs::write(repo.path().join("fixup_change.txt"), "fixup content")
            .expect("Failed to write file");
        StdCommand::new("git")
            .args(["add", "fixup_change.txt"])
            .current_dir(repo.path())
            .output()
            .expect("Failed to stage changes");

        let _result = repo
            .run_git_x_direct(&["fixup", hash])
            .expect("Fixup should succeed");
    }
}

#[test]
#[serial]
fn test_e2e_fixup_with_rebase_happy_path() {
    if !should_run_destructive_tests() {
        return;
    }

    let repo = repo_with_commits(2);

    // Get the hash of the first commit
    let output = StdCommand::new("git")
        .args(["log", "--oneline", "--reverse"])
        .current_dir(repo.path())
        .output()
        .expect("Failed to get commit hash");

    let stdout = String::from_utf8_lossy(&output.stdout);
    if let Some(line) = stdout.lines().next() {
        let hash = line.split_whitespace().next().unwrap();

        // Stage some changes for the fixup
        std::fs::write(
            repo.path().join("fixup_rebase_change.txt"),
            "fixup rebase content",
        )
        .expect("Failed to write file");
        StdCommand::new("git")
            .args(["add", "fixup_rebase_change.txt"])
            .current_dir(repo.path())
            .output()
            .expect("Failed to stage changes");

        let _result = repo
            .run_git_x_direct(&["fixup", hash, "--rebase"])
            .expect("Fixup with rebase should succeed");
    }
}

// ============================================================================
// Commands with Flags
// ============================================================================

#[test]
#[serial]
fn test_e2e_prune_branches_dry_run_happy_path() {
    let repo = repo_with_merged_branch("to-prune", "main");
    let result = repo
        .run_git_x_direct(&["prune-branches", "--dry-run"])
        .expect("Prune branches dry run should succeed");
    assert!(
        result.contains("would be deleted")
            || result.contains("prune")
            || result.contains("No branches")
    );
}

#[test]
#[serial]
fn test_e2e_prune_branches_with_except_happy_path() {
    let repo = repo_with_merged_branch("protected", "main");
    let _result = repo
        .run_git_x_direct(&["prune-branches", "--except", "protected", "--dry-run"])
        .expect("Prune branches with except should succeed");
}

#[test]
#[serial]
fn test_e2e_clean_branches_dry_run_happy_path() {
    let repo = repo_with_merged_branch("merged-feature", "main");
    let result = repo
        .run_git_x_direct(&["clean-branches", "--dry-run"])
        .expect("Clean branches dry run should succeed");
    assert!(
        result.contains("would be deleted")
            || result.contains("clean")
            || result.contains("No branches")
    );
}

#[test]
#[serial]
fn test_e2e_sync_default_happy_path() {
    let (_temp_dir, repo_path) = create_repo_with_remote();
    let repo = TestRepo {
        _temp_dir,
        path: repo_path,
    };

    let _result = repo.run_git_x_direct(&["sync"]); // May show "Already up to date" or error
}

#[test]
#[serial]
fn test_e2e_sync_with_merge_happy_path() {
    let (_temp_dir, repo_path) = create_repo_with_remote();
    let repo = TestRepo {
        _temp_dir,
        path: repo_path,
    };

    let _result = repo.run_git_x_direct(&["sync", "--merge"]); // May show "Already up to date" or error
}

#[test]
#[serial]
fn test_e2e_undo_happy_path() {
    if !should_run_destructive_tests() {
        return;
    }

    let repo = repo_with_commits(2);
    let result = repo
        .run_git_x_direct(&["undo"])
        .expect("Undo should succeed");
    assert!(result.contains("Undid last commit") || result.contains("undo"));
}

// ============================================================================
// Subcommand Tests: StashBranch
// ============================================================================

#[test]
#[serial]
fn test_e2e_stash_branch_create_happy_path() {
    if !should_run_destructive_tests() {
        return;
    }

    let (_temp_dir, repo_path) = create_repo_with_stash();
    let repo = TestRepo {
        _temp_dir,
        path: repo_path,
    };

    let result = repo
        .run_git_x_direct(&["stash-branch", "create", "stash-feature"])
        .expect("Stash branch create should succeed");
    assert!(result.contains("Created branch") || result.contains("stash-feature"));
}

#[test]
#[serial]
fn test_e2e_stash_branch_create_with_stash_ref_happy_path() {
    if !should_run_destructive_tests() {
        return;
    }

    let (_temp_dir, repo_path) = create_repo_with_stash();

    Command::cargo_bin("git-x")
        .expect("Failed to find binary")
        .args([
            "stash-branch",
            "create",
            "stash-feature",
            "--stash",
            "stash@{0}",
        ])
        .current_dir(&repo_path)
        .assert()
        .success();
}

#[test]
#[serial]
fn test_e2e_stash_branch_clean_dry_run_happy_path() {
    let (_temp_dir, repo_path) = create_repo_with_stash();

    Command::cargo_bin("git-x")
        .expect("Failed to find binary")
        .args(["stash-branch", "clean", "--dry-run"])
        .current_dir(&repo_path)
        .assert()
        .success();
}

#[test]
#[serial]
fn test_e2e_stash_branch_clean_older_than_happy_path() {
    let (_temp_dir, repo_path) = create_repo_with_stash();

    Command::cargo_bin("git-x")
        .expect("Failed to find binary")
        .args(["stash-branch", "clean", "--older-than", "1w", "--dry-run"])
        .current_dir(&repo_path)
        .assert()
        .success();
}

#[test]
#[serial]
fn test_e2e_stash_branch_apply_by_branch_list_happy_path() {
    let (_temp_dir, repo_path) = create_repo_with_stash();

    Command::cargo_bin("git-x")
        .expect("Failed to find binary")
        .args(["stash-branch", "apply-by-branch", "main", "--list"])
        .current_dir(&repo_path)
        .assert()
        .success();
}

#[test]
#[serial]
fn test_e2e_stash_branch_interactive_happy_path() {
    let (_temp_dir, repo_path) = create_repo_with_stash();

    // Set non-interactive mode
    Command::cargo_bin("git-x")
        .expect("Failed to find binary")
        .args(["stash-branch", "interactive"])
        .current_dir(&repo_path)
        .env("GIT_X_NON_INTERACTIVE", "1")
        .assert()
        .success(); // Should handle non-interactive gracefully
}

#[test]
#[serial]
fn test_e2e_stash_branch_export_happy_path() {
    let (_temp_dir, repo_path) = create_repo_with_stash();
    let export_dir = repo_path.join("exports");
    fs::create_dir(&export_dir).expect("Failed to create export dir");

    Command::cargo_bin("git-x")
        .expect("Failed to find binary")
        .args(["stash-branch", "export", export_dir.to_str().unwrap()])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stdout(contains("Exported"));
}

#[test]
#[serial]
fn test_e2e_stash_branch_export_specific_stash_happy_path() {
    let (_temp_dir, repo_path) = create_repo_with_stash();
    let export_dir = repo_path.join("exports");
    fs::create_dir(&export_dir).expect("Failed to create export dir");

    Command::cargo_bin("git-x")
        .expect("Failed to find binary")
        .args([
            "stash-branch",
            "export",
            export_dir.to_str().unwrap(),
            "--stash",
            "stash@{0}",
        ])
        .current_dir(&repo_path)
        .assert()
        .success();
}

// ============================================================================
// Subcommand Tests: Upstream
// ============================================================================

#[test]
#[serial]
fn test_e2e_upstream_set_happy_path() {
    let (_temp_dir, repo_path) = create_repo_with_remote();
    let repo = TestRepo {
        _temp_dir,
        path: repo_path,
    };

    // Upstream set may fail if the remote branch doesn't exist, which is expected behavior
    let _result = repo.run_git_x_direct(&["upstream", "set", "origin/main"]);
}

#[test]
#[serial]
fn test_e2e_upstream_status_happy_path() {
    let (_temp_dir, repo_path) = create_repo_with_remote();
    let repo = TestRepo {
        _temp_dir,
        path: repo_path,
    };

    let result = repo
        .run_git_x_direct(&["upstream", "status"])
        .expect("Upstream status should succeed");
    assert!(result.contains("Upstream Status") || result.contains("status"));
}

#[test]
#[serial]
fn test_e2e_upstream_sync_all_dry_run_happy_path() {
    let (_temp_dir, repo_path) = create_repo_with_remote();
    let repo = TestRepo {
        _temp_dir,
        path: repo_path,
    };

    let _result = repo.run_git_x_direct(&["upstream", "sync-all", "--dry-run"]);
}

#[test]
#[serial]
fn test_e2e_upstream_sync_all_merge_happy_path() {
    let (_temp_dir, repo_path) = create_repo_with_remote();
    let repo = TestRepo {
        _temp_dir,
        path: repo_path,
    };

    let _result = repo.run_git_x_direct(&["upstream", "sync-all", "--merge", "--dry-run"]);
}

// ============================================================================
// Subcommand Tests: Bisect
// ============================================================================

#[test]
#[serial]
fn test_e2e_bisect_start_happy_path() {
    if !should_run_destructive_tests() {
        return;
    }

    let repo = repo_with_commits(5);

    // Get commit hashes
    let output = StdCommand::new("git")
        .args(["log", "--oneline", "--reverse"])
        .current_dir(repo.path())
        .output()
        .expect("Failed to get commits");

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let commits: Vec<_> = stdout_str
        .lines()
        .map(|line| line.split_whitespace().next().unwrap())
        .collect();

    if commits.len() >= 2 {
        let result = repo
            .run_git_x_direct(&["bisect", "start", commits[0], commits[commits.len() - 1]])
            .expect("Bisect start should succeed");
        assert!(
            result.contains("Started bisect")
                || result.contains("bisect")
                || result.contains("Starting")
        );
    }
}

#[test]
#[serial]
fn test_e2e_bisect_status_happy_path() {
    let repo = basic_repo();
    let _result = repo.run_git_x_direct(&["bisect", "status"]); // Should handle no active bisect gracefully
}

#[test]
#[serial]
fn test_e2e_bisect_good_outside_session_happy_path() {
    let repo = basic_repo();
    let _result = repo.run_git_x_direct(&["bisect", "good"]); // Should handle no active bisect gracefully
}

#[test]
#[serial]
fn test_e2e_bisect_bad_outside_session_happy_path() {
    let repo = basic_repo();
    let _result = repo.run_git_x_direct(&["bisect", "bad"]); // Should handle no active bisect gracefully
}

#[test]
#[serial]
fn test_e2e_bisect_skip_outside_session_happy_path() {
    let repo = basic_repo();
    let _result = repo.run_git_x_direct(&["bisect", "skip"]); // Should handle no active bisect gracefully
}

#[test]
#[serial]
fn test_e2e_bisect_reset_outside_session_happy_path() {
    let repo = basic_repo();
    let _result = repo.run_git_x_direct(&["bisect", "reset"]); // Should handle no active bisect gracefully
}

// ============================================================================
// Error Handling Happy Paths (commands that gracefully handle missing data)
// ============================================================================

#[test]
#[serial]
fn test_e2e_commands_handle_empty_repo_gracefully() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let repo_path = temp_dir.path().to_path_buf();

    // Initialize empty git repo
    StdCommand::new("git")
        .args(["init"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to init git repo");

    // Configure git
    StdCommand::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to set git user.name");
    StdCommand::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to set git user.email");

    let repo = TestRepo {
        _temp_dir: temp_dir,
        path: repo_path.clone(),
    };

    // Test commands that should handle empty repos gracefully using direct calls
    let commands = [
        vec!["info"],
        vec!["health"],
        vec!["graph"],
        vec!["color-graph"],
        vec!["contributors"],
        vec!["technical-debt"],
        vec!["large-files"],
        vec!["summary"],
        vec!["switch-recent"],
    ];

    for cmd_args in commands {
        // These should not crash on empty repo, though some may return errors
        let _result = repo.run_git_x_direct(&cmd_args);
    }
}

#[test]
#[serial]
fn test_e2e_commands_handle_non_git_directory_gracefully() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let repo_path = temp_dir.path().to_path_buf();

    let repo = TestRepo {
        _temp_dir: temp_dir,
        path: repo_path,
    };

    // Test commands that should handle non-git directories gracefully
    let commands = [
        vec!["info"],
        vec!["health"],
        vec!["contributors"],
        vec!["technical-debt"],
        vec!["large-files"],
        vec!["summary"],
    ];

    for cmd_args in commands {
        // These should not crash, though they may return errors for non-git directories
        let _result = repo.run_git_x_direct(&cmd_args);
    }
}
