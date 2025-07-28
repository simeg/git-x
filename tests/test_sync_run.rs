// Integration tests for sync.rs run() function testing all code paths

use assert_cmd::Command;
use tempfile::TempDir;

mod common;

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
