use assert_cmd::Command;
use git_x::cli::UpstreamAction;
use git_x::upstream::*;
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

// Direct run() function tests for maximum coverage

#[test]
fn test_upstream_run_set_function() {
    let (_temp_dir, repo_path) = create_test_repo();

    std::env::set_current_dir(&repo_path).expect("Failed to change directory");

    // Test set action through run function
    let action = UpstreamAction::Set {
        upstream: "origin/main".to_string(),
    };

    git_x::upstream::run(action);

    std::env::set_current_dir("/").expect("Failed to reset directory");
}

#[test]
fn test_upstream_run_set_function_invalid_format() {
    let (_temp_dir, repo_path) = create_test_repo();

    std::env::set_current_dir(&repo_path).expect("Failed to change directory");

    // Test set action with invalid upstream format
    let action = UpstreamAction::Set {
        upstream: "invalid_format".to_string(),
    };

    git_x::upstream::run(action);

    std::env::set_current_dir("/").expect("Failed to reset directory");
}

#[test]
fn test_upstream_run_status_function() {
    let (_temp_dir, repo_path) = create_test_repo();

    std::env::set_current_dir(&repo_path).expect("Failed to change directory");

    // Test status action through run function
    let action = UpstreamAction::Status;

    git_x::upstream::run(action);

    std::env::set_current_dir("/").expect("Failed to reset directory");
}

#[test]
fn test_upstream_run_sync_all_function() {
    let (_temp_dir, repo_path) = create_test_repo();

    std::env::set_current_dir(&repo_path).expect("Failed to change directory");

    // Test sync-all action through run function
    let action = UpstreamAction::SyncAll {
        dry_run: true,
        merge: false,
    };

    git_x::upstream::run(action);

    std::env::set_current_dir("/").expect("Failed to reset directory");
}

#[test]
fn test_upstream_run_sync_all_function_with_merge() {
    let (_temp_dir, repo_path) = create_test_repo();

    std::env::set_current_dir(&repo_path).expect("Failed to change directory");

    // Test sync-all action with merge option
    let action = UpstreamAction::SyncAll {
        dry_run: false,
        merge: true,
    };

    git_x::upstream::run(action);

    std::env::set_current_dir("/").expect("Failed to reset directory");
}

// Helper function to create a remote repository
fn create_remote_repo(name: &str, repo_path: &PathBuf) -> (PathBuf, String) {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let unique_name = format!("{name}_{timestamp}");
    let remote_dir = repo_path
        .parent()
        .unwrap()
        .join(format!("{unique_name}.git"));

    // Initialize bare remote repo
    Command::new("git")
        .args(["init", "--bare"])
        .current_dir(remote_dir.parent().unwrap())
        .arg(&remote_dir)
        .assert()
        .success();

    // Add remote to main repo
    Command::new("git")
        .args(["remote", "add", name, remote_dir.to_str().unwrap()])
        .current_dir(repo_path)
        .assert()
        .success();

    // Get current branch name
    let branch_output = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to get current branch");
    let current_branch = String::from_utf8_lossy(&branch_output.stdout)
        .trim()
        .to_string();

    // Push initial branch to remote with set-upstream and force
    Command::new("git")
        .args(["push", "-u", name, &current_branch, "--force"])
        .current_dir(repo_path)
        .assert()
        .success();

    (remote_dir, current_branch)
}

#[test]
fn test_get_git_branch_set_upstream_args() {
    assert_eq!(
        get_git_branch_set_upstream_args(),
        ["branch", "--set-upstream-to"]
    );
}

#[test]
fn test_format_error_message() {
    assert_eq!(format_error_message("Test error"), "❌ Test error");
    assert_eq!(
        format_error_message("Upstream validation failed"),
        "❌ Upstream validation failed"
    );
}

#[test]
fn test_format_setting_upstream_message() {
    assert_eq!(
        format_setting_upstream_message("feature", "origin/main"),
        "🔗 Setting upstream for 'feature' to 'origin/main'..."
    );
    assert_eq!(
        format_setting_upstream_message("develop", "upstream/develop"),
        "🔗 Setting upstream for 'develop' to 'upstream/develop'..."
    );
}

#[test]
fn test_format_upstream_set_message() {
    assert_eq!(
        format_upstream_set_message("feature", "origin/feature"),
        "✅ Upstream for 'feature' set to 'origin/feature'"
    );
    assert_eq!(
        format_upstream_set_message("main", "origin/main"),
        "✅ Upstream for 'main' set to 'origin/main'"
    );
}

#[test]
fn test_format_no_branches_message() {
    assert_eq!(format_no_branches_message(), "ℹ️ No local branches found");
}

#[test]
fn test_format_upstream_status_header() {
    assert_eq!(
        format_upstream_status_header(),
        "🔗 Upstream status for all branches:\n"
    );
}

#[test]
fn test_format_branch_without_upstream() {
    assert_eq!(
        format_branch_without_upstream("feature", false),
        "  feature -> (no upstream)"
    );
    assert_eq!(
        format_branch_without_upstream("main", true),
        "* main -> (no upstream)"
    );
}

#[test]
fn test_format_no_upstream_branches_message() {
    assert_eq!(
        format_no_upstream_branches_message(),
        "ℹ️ No branches with upstream configuration found"
    );
}

#[test]
fn test_format_sync_all_start_message() {
    assert_eq!(
        format_sync_all_start_message(3, true, false),
        "🧪 (dry run) Would sync 3 branch(es) with upstream using rebase:"
    );
    assert_eq!(
        format_sync_all_start_message(2, false, true),
        "🔄 Syncing 2 branch(es) with upstream using merge:"
    );
    assert_eq!(
        format_sync_all_start_message(1, false, false),
        "🔄 Syncing 1 branch(es) with upstream using rebase:"
    );
}

#[test]
fn test_format_sync_results_header() {
    assert_eq!(format_sync_results_header(), "\n📊 Sync results:");
}

#[test]
fn test_format_sync_summary() {
    assert_eq!(
        format_sync_summary(3, true),
        "\n💡 Would sync 3 branch(es). Run without --dry-run to apply changes."
    );
    assert_eq!(
        format_sync_summary(2, false),
        "\n✅ Synced 2 branch(es) successfully."
    );
}

#[test]
fn test_upstream_set_command_help() {
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["upstream", "set", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Set upstream for current branch"));
}

#[test]
fn test_upstream_status_command_help() {
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["upstream", "status", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Show upstream status for all branches",
        ));
}

#[test]
fn test_upstream_sync_all_command_help() {
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["upstream", "sync-all", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Sync all local branches with their upstreams",
        ));
}

#[test]
fn test_upstream_set_invalid_format() {
    let (_temp_dir, repo_path) = create_test_repo();

    // Test empty upstream
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["upstream", "set", ""])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stderr(predicate::str::contains("Upstream cannot be empty"));

    // Test upstream without slash
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["upstream", "set", "origin"])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stderr(predicate::str::contains(
            "must be in format 'remote/branch'",
        ));

    // Test upstream with empty parts
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["upstream", "set", "/main"])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stderr(predicate::str::contains("Invalid upstream format"));

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["upstream", "set", "origin/"])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stderr(predicate::str::contains("Invalid upstream format"));
}

#[test]
fn test_upstream_set_nonexistent_upstream() {
    let (_temp_dir, repo_path) = create_test_repo();

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["upstream", "set", "nonexistent/main"])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stderr(predicate::str::contains("Upstream branch does not exist"));
}

#[test]
fn test_upstream_set_success() {
    let (_temp_dir, repo_path) = create_test_repo();
    let (_remote_dir, branch_name) = create_remote_repo("origin", &repo_path);

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["upstream", "set", &format!("origin/{branch_name}")])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "Setting upstream for '{branch_name}' to 'origin/{branch_name}'"
        )))
        .stdout(predicate::str::contains(format!(
            "Upstream for '{branch_name}' set to 'origin/{branch_name}'"
        )));
}

#[test]
fn test_upstream_status_no_branches() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let repo_path = temp_dir.path().to_path_buf();

    // Initialize empty git repo with no commits
    Command::new("git")
        .args(["init"])
        .current_dir(&repo_path)
        .assert()
        .success();

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

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["upstream", "status"])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("No local branches found"));
}

#[test]
fn test_upstream_status_with_branches() {
    let (_temp_dir, repo_path) = create_test_repo();
    let (_remote_dir, branch_name) = create_remote_repo("origin", &repo_path);

    // Create a feature branch
    Command::new("git")
        .args(["checkout", "-b", "feature"])
        .current_dir(&repo_path)
        .assert()
        .success();

    // Set upstream for the main branch
    Command::new("git")
        .args(["checkout", &branch_name])
        .current_dir(&repo_path)
        .assert()
        .success();

    Command::new("git")
        .args([
            "branch",
            "--set-upstream-to",
            &format!("origin/{branch_name}"),
        ])
        .current_dir(&repo_path)
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["upstream", "status"])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Upstream status for all branches"))
        .stdout(predicate::str::contains(format!(
            "{branch_name} -> origin/{branch_name}"
        )))
        .stdout(predicate::str::contains("feature -> (no upstream)"));
}

#[test]
fn test_upstream_sync_all_no_upstreams() {
    let (_temp_dir, repo_path) = create_test_repo();

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["upstream", "sync-all"])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "No branches with upstream configuration found",
        ));
}

#[test]
fn test_upstream_sync_all_dry_run() {
    let (_temp_dir, repo_path) = create_test_repo();
    let (_remote_dir, branch_name) = create_remote_repo("origin", &repo_path);

    // Set upstream for master
    Command::new("git")
        .args([
            "branch",
            "--set-upstream-to",
            &format!("origin/{branch_name}"),
        ])
        .current_dir(&repo_path)
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["upstream", "sync-all", "--dry-run"])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("(dry run) Would sync"))
        .stdout(predicate::str::contains("using rebase"));
}

#[test]
fn test_upstream_sync_all_with_merge() {
    let (_temp_dir, repo_path) = create_test_repo();
    let (_remote_dir, branch_name) = create_remote_repo("origin", &repo_path);

    // Set upstream for master
    Command::new("git")
        .args([
            "branch",
            "--set-upstream-to",
            &format!("origin/{branch_name}"),
        ])
        .current_dir(&repo_path)
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["upstream", "sync-all", "--merge", "--dry-run"])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("using merge"));
}

#[test]
fn test_upstream_command_outside_git_repo() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["upstream", "status"])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("Failed to list local branches"));
}

#[test]
fn test_upstream_set_outside_git_repo() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["upstream", "set", "origin/main"])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("Upstream branch does not exist"));
}

#[test]
fn test_upstream_main_command_help() {
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["upstream", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Manage upstream branch relationships",
        ));
}

// Unit tests for core logic functions

#[test]
fn test_validate_upstream_format_valid() {
    assert!(validate_upstream_format("origin/main").is_ok());
    assert!(validate_upstream_format("upstream/develop").is_ok());
    assert!(validate_upstream_format("fork/feature-branch").is_ok());
    assert!(validate_upstream_format("remote/branch-name").is_ok());
}

#[test]
fn test_validate_upstream_format_invalid() {
    assert!(validate_upstream_format("").is_err());
    assert!(validate_upstream_format("origin").is_err());
    assert!(validate_upstream_format("main").is_err());
    assert!(validate_upstream_format("origin/").is_err());
    assert!(validate_upstream_format("/main").is_err());
    assert!(validate_upstream_format("origin//main").is_err());
}

#[test]
fn test_get_current_branch_success() {
    let (_temp_dir, repo_path) = create_test_repo();

    std::env::set_current_dir(&repo_path).expect("Failed to change directory");

    let result = get_current_branch();
    std::env::set_current_dir("/").expect("Failed to reset directory");

    assert!(result.is_ok());
    let branch = result.unwrap();
    assert!(!branch.is_empty());
}

#[test]
fn test_get_all_local_branches_success() {
    let (_temp_dir, repo_path) = create_test_repo();

    // Create additional branches
    Command::new("git")
        .args(["checkout", "-b", "feature-branch"])
        .current_dir(&repo_path)
        .assert()
        .success();

    Command::new("git")
        .args(["checkout", "-b", "another-branch"])
        .current_dir(&repo_path)
        .assert()
        .success();

    std::env::set_current_dir(&repo_path).expect("Failed to change directory");

    let result = get_all_local_branches();
    std::env::set_current_dir("/").expect("Failed to reset directory");

    assert!(result.is_ok());
    let branches = result.unwrap();
    assert!(branches.len() >= 3); // main/master + feature-branch + another-branch
    assert!(branches.contains(&"feature-branch".to_string()));
    assert!(branches.contains(&"another-branch".to_string()));
}

#[test]
fn test_get_all_local_branches_not_git_repo() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    std::env::set_current_dir(temp_dir.path()).expect("Failed to change directory");

    let result = get_all_local_branches();
    std::env::set_current_dir("/").expect("Failed to reset directory");

    assert!(result.is_err());
}

#[test]
fn test_get_branch_upstream_no_upstream() {
    let (_temp_dir, repo_path) = create_test_repo();

    std::env::set_current_dir(&repo_path).expect("Failed to change directory");

    let result = get_branch_upstream("main");
    std::env::set_current_dir("/").expect("Failed to reset directory");

    assert!(result.is_err());
}

#[test]
fn test_get_branch_sync_status_no_upstream() {
    let (_temp_dir, repo_path) = create_test_repo();

    std::env::set_current_dir(&repo_path).expect("Failed to change directory");

    let result = get_branch_sync_status("main", "origin/main");
    std::env::set_current_dir("/").expect("Failed to reset directory");

    assert!(result.is_err());
}

#[test]
fn test_get_branches_with_upstreams_no_upstreams() {
    let (_temp_dir, repo_path) = create_test_repo();

    std::env::set_current_dir(&repo_path).expect("Failed to change directory");

    let result = get_branches_with_upstreams();
    std::env::set_current_dir("/").expect("Failed to reset directory");

    assert!(result.is_ok());
    let branches = result.unwrap();
    assert!(branches.is_empty());
}

#[test]
fn test_validate_upstream_exists_invalid() {
    let (_temp_dir, repo_path) = create_test_repo();

    std::env::set_current_dir(&repo_path).expect("Failed to change directory");

    let result = validate_upstream_exists("nonexistent/branch");
    std::env::set_current_dir("/").expect("Failed to reset directory");

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Upstream branch does not exist");
}

#[test]
fn test_sync_status_enum_debug() {
    let status = SyncStatus::Diverged(3, 5);
    let debug_str = format!("{status:?}");
    assert!(debug_str.contains("Diverged"));
    assert!(debug_str.contains("3"));
    assert!(debug_str.contains("5"));
}

#[test]
fn test_sync_result_enum_debug() {
    let result = SyncResult::Error("Test error".to_string());
    let debug_str = format!("{result:?}");
    assert!(debug_str.contains("Error"));
    assert!(debug_str.contains("Test error"));
}

// Additional tests for better coverage of upstream.rs functions

#[test]
fn test_format_branch_with_upstream_all_statuses() {
    // Test all sync status variants
    assert_eq!(
        format_branch_with_upstream("main", "origin/main", &SyncStatus::UpToDate, false),
        "  main -> origin/main (✅ up-to-date)"
    );
    assert_eq!(
        format_branch_with_upstream("feature", "origin/feature", &SyncStatus::Behind(3), true),
        "* feature -> origin/feature (⬇️ 3 behind)"
    );
    assert_eq!(
        format_branch_with_upstream("hotfix", "origin/hotfix", &SyncStatus::Ahead(2), false),
        "  hotfix -> origin/hotfix (⬆️ 2 ahead)"
    );
    assert_eq!(
        format_branch_with_upstream(
            "develop",
            "origin/develop",
            &SyncStatus::Diverged(1, 4),
            true
        ),
        "* develop -> origin/develop (🔀 1 behind, 4 ahead)"
    );
    assert_eq!(
        format_branch_with_upstream("test", "origin/test", &SyncStatus::Unknown, false),
        "  test -> origin/test (❓ unknown)"
    );
}

#[test]
fn test_get_branches_with_upstreams_success() {
    let (_temp_dir, repo_path) = create_test_repo();

    std::env::set_current_dir(&repo_path).expect("Failed to change directory");

    // Test the function works even when no upstreams are configured
    let result = get_branches_with_upstreams();
    std::env::set_current_dir("/").expect("Failed to reset directory");

    assert!(result.is_ok());
    let branches = result.unwrap();
    // Should return empty vector when no upstreams are configured
    assert!(branches.is_empty());
}

#[test]
fn test_validate_upstream_exists_valid() {
    let (_temp_dir, repo_path) = create_test_repo();
    let (_remote_dir, branch_name) = create_remote_repo("origin", &repo_path);

    std::env::set_current_dir(&repo_path).expect("Failed to change directory");

    let result = validate_upstream_exists(&format!("origin/{branch_name}"));
    std::env::set_current_dir("/").expect("Failed to reset directory");

    assert!(result.is_ok());
}

#[test]
fn test_get_branch_upstream_with_upstream() {
    let (_temp_dir, repo_path) = create_test_repo();
    let (_remote_dir, branch_name) = create_remote_repo("origin", &repo_path);

    // Set upstream for the main branch
    Command::new("git")
        .args([
            "branch",
            "--set-upstream-to",
            &format!("origin/{branch_name}"),
        ])
        .current_dir(&repo_path)
        .assert()
        .success();

    std::env::set_current_dir(&repo_path).expect("Failed to change directory");

    let result = get_branch_upstream(&branch_name);
    std::env::set_current_dir("/").expect("Failed to reset directory");

    assert!(result.is_ok());
    let upstream = result.unwrap();
    assert_eq!(upstream, format!("origin/{branch_name}"));
}

#[test]
fn test_get_branch_sync_status_up_to_date() {
    let (_temp_dir, repo_path) = create_test_repo();
    let (_remote_dir, branch_name) = create_remote_repo("origin", &repo_path);

    // Set upstream for the main branch
    Command::new("git")
        .args([
            "branch",
            "--set-upstream-to",
            &format!("origin/{branch_name}"),
        ])
        .current_dir(&repo_path)
        .assert()
        .success();

    std::env::set_current_dir(&repo_path).expect("Failed to change directory");

    let result = get_branch_sync_status(&branch_name, &format!("origin/{branch_name}"));
    std::env::set_current_dir("/").expect("Failed to reset directory");

    assert!(result.is_ok());
    let status = result.unwrap();
    assert!(matches!(status, SyncStatus::UpToDate));
}

#[test]
fn test_sync_status_enum_clone() {
    let status = SyncStatus::Diverged(2, 3);
    let cloned = status.clone();
    assert!(matches!(cloned, SyncStatus::Diverged(2, 3)));

    let status2 = SyncStatus::Behind(5);
    let cloned2 = status2.clone();
    assert!(matches!(cloned2, SyncStatus::Behind(5)));
}

#[test]
fn test_sync_result_enum_clone() {
    let result = SyncResult::Error("Test".to_string());
    let cloned = result.clone();
    assert!(matches!(cloned, SyncResult::Error(_)));

    let result2 = SyncResult::UpToDate;
    let cloned2 = result2.clone();
    assert!(matches!(cloned2, SyncResult::UpToDate));
}

// Additional comprehensive tests for edge cases and error paths

#[test]
fn test_validate_upstream_format_edge_cases() {
    // Test various invalid formats
    assert!(validate_upstream_format("origin/main/extra").is_err());
    assert!(validate_upstream_format("//").is_err());
    assert!(validate_upstream_format("origin//branch").is_err());
    assert!(validate_upstream_format("origin/").is_err());
    assert!(validate_upstream_format("/branch").is_err());

    // Test valid formats
    assert!(validate_upstream_format("upstream/develop").is_ok());
    assert!(validate_upstream_format("fork/feature-branch").is_ok());
    assert!(validate_upstream_format("remote123/branch-name_test").is_ok());
}

#[test]
fn test_format_sync_result_line_all_variants() {
    // Test all SyncResult variants
    assert_eq!(
        format_sync_result_line("main", &SyncResult::UpToDate),
        "  ✅ main: already up-to-date"
    );
    assert_eq!(
        format_sync_result_line("feature", &SyncResult::Synced),
        "  ✅ feature: synced successfully"
    );
    assert_eq!(
        format_sync_result_line("develop", &SyncResult::WouldSync),
        "  🔄 develop: would be synced"
    );
    assert_eq!(
        format_sync_result_line("hotfix", &SyncResult::Ahead),
        "  ⬆️ hotfix: ahead of upstream (skipped)"
    );
    assert_eq!(
        format_sync_result_line("test", &SyncResult::Error("merge conflict".to_string())),
        "  ❌ test: merge conflict"
    );
}

#[test]
fn test_upstream_set_with_different_branches() {
    let (_temp_dir, repo_path) = create_test_repo();
    let (_remote_dir, _branch_name) = create_remote_repo("origin", &repo_path);

    // Create a feature branch
    Command::new("git")
        .args(["checkout", "-b", "feature-test"])
        .current_dir(&repo_path)
        .assert()
        .success();

    // Create a commit in feature branch
    std::fs::write(repo_path.join("feature.txt"), "feature content").expect("Failed to write");
    Command::new("git")
        .args(["add", "feature.txt"])
        .current_dir(&repo_path)
        .assert()
        .success();
    Command::new("git")
        .args(["commit", "-m", "Add feature"])
        .current_dir(&repo_path)
        .assert()
        .success();

    // Push feature branch to origin
    Command::new("git")
        .args(["push", "-u", "origin", "feature-test", "--force"])
        .current_dir(&repo_path)
        .assert()
        .success();

    // Test setting upstream for feature branch
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["upstream", "set", "origin/feature-test"])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Setting upstream for 'feature-test'",
        ))
        .stdout(predicate::str::contains("Upstream for 'feature-test' set"));
}

#[test]
fn test_upstream_status_multiple_branches_with_mixed_upstreams() {
    let (_temp_dir, repo_path) = create_test_repo();
    let (_remote_dir, branch_name) = create_remote_repo("origin", &repo_path);

    // Create multiple branches
    Command::new("git")
        .args(["checkout", "-b", "feature-1"])
        .current_dir(&repo_path)
        .assert()
        .success();

    Command::new("git")
        .args(["checkout", "-b", "feature-2"])
        .current_dir(&repo_path)
        .assert()
        .success();

    // Go back to main and set upstream
    Command::new("git")
        .args(["checkout", &branch_name])
        .current_dir(&repo_path)
        .assert()
        .success();

    Command::new("git")
        .args([
            "branch",
            "--set-upstream-to",
            &format!("origin/{branch_name}"),
        ])
        .current_dir(&repo_path)
        .assert()
        .success();

    // Test upstream status with mixed scenarios
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["upstream", "status"])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Upstream status for all branches"))
        .stdout(predicate::str::contains(format!(
            "{branch_name} -> origin/{branch_name}"
        )))
        .stdout(predicate::str::contains("feature-1 -> (no upstream)"))
        .stdout(predicate::str::contains("feature-2 -> (no upstream)"));
}

#[test]
fn test_upstream_sync_all_error_scenarios() {
    let (_temp_dir, _repo_path) = create_test_repo();

    // Test sync-all when not in git repo (move to non-git directory)
    let temp_dir2 = TempDir::new().expect("Failed to create temp directory");

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["upstream", "sync-all"])
        .current_dir(temp_dir2.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("Failed to list local branches"));
}

#[test]
fn test_validate_upstream_exists_git_error() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    std::env::set_current_dir(temp_dir.path()).expect("Failed to change directory");

    // This should fail because we're not in a git repository
    let result = validate_upstream_exists("origin/main");
    std::env::set_current_dir("/").expect("Failed to reset directory");

    assert!(result.is_err());
}

#[test]
fn test_get_current_branch_edge_case() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    std::env::set_current_dir(temp_dir.path()).expect("Failed to change directory");

    let result = get_current_branch();
    std::env::set_current_dir("/").expect("Failed to reset directory");

    // In CI environments, there might be a git repo in parent directories
    // So we test that either it fails with the expected error, or succeeds with a valid branch name
    if result.is_err() {
        assert_eq!(result.unwrap_err(), "Not in a git repository");
    } else {
        // If it succeeds, it should return a non-empty string
        let branch = result.unwrap();
        assert!(!branch.is_empty());
    }
}

#[test]
fn test_get_all_local_branches_error_case() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    std::env::set_current_dir(temp_dir.path()).expect("Failed to change directory");

    let result = get_all_local_branches();
    std::env::set_current_dir("/").expect("Failed to reset directory");

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Failed to list local branches");
}

#[test]
fn test_get_branch_sync_status_error_case() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    std::env::set_current_dir(temp_dir.path()).expect("Failed to change directory");

    let result = get_branch_sync_status("main", "origin/main");
    std::env::set_current_dir("/").expect("Failed to reset directory");

    assert!(result.is_err());
}

#[test]
fn test_enum_debug_formatting() {
    // Test Debug trait implementation for comprehensive coverage
    let status_variants = vec![
        SyncStatus::UpToDate,
        SyncStatus::Behind(5),
        SyncStatus::Ahead(3),
        SyncStatus::Diverged(2, 4),
        SyncStatus::Unknown,
    ];

    for status in status_variants {
        let debug_str = format!("{status:?}");
        assert!(!debug_str.is_empty());
    }

    let result_variants = vec![
        SyncResult::UpToDate,
        SyncResult::Synced,
        SyncResult::WouldSync,
        SyncResult::Ahead,
        SyncResult::Error("test error".to_string()),
    ];

    for result in result_variants {
        let debug_str = format!("{result:?}");
        assert!(!debug_str.is_empty());
    }
}
