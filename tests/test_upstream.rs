use assert_cmd::Command;
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
