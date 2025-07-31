// Clippy thinks methods are unused because they're not used in prod
#![allow(dead_code)]

use assert_cmd::Command;
use assert_cmd::assert::OutputAssertExt;
use std::fs;
use std::path::Path;
use std::process::Command as StdCommand;
use tempfile::{TempDir, tempdir};

pub struct TestRepo {
    pub _temp_dir: TempDir,
    pub path: std::path::PathBuf,
}

impl TestRepo {
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Configure Git user identity for this repository
    fn configure_git_identity(&self) {
        StdCommand::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(&self.path)
            .assert()
            .success();
        StdCommand::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(&self.path)
            .assert()
            .success();
    }

    /// Run a git-x command in this repo and return the Command for assertions
    pub fn run_git_x(&self, args: &[&str]) -> assert_cmd::assert::Assert {
        Command::cargo_bin("git-x")
            .unwrap()
            .args(args)
            .current_dir(self.path())
            .env("GIT_X_NON_INTERACTIVE", "1")
            .assert()
    }

    /// Add a commit with specified file content and message
    pub fn add_commit(&self, file_name: &str, content: &str, message: &str) {
        fs::write(self.path.join(file_name), content).unwrap();
        StdCommand::new("git")
            .args(["add", "."])
            .current_dir(&self.path)
            .assert()
            .success();
        StdCommand::new("git")
            .args(["commit", "-m", message])
            .current_dir(&self.path)
            .assert()
            .success();
    }

    /// Create and checkout a new branch
    pub fn create_branch(&self, branch_name: &str) {
        StdCommand::new("git")
            .args(["checkout", "-b", branch_name])
            .current_dir(&self.path)
            .assert()
            .success();
    }

    /// Checkout an existing branch
    pub fn checkout_branch(&self, branch_name: &str) {
        StdCommand::new("git")
            .args(["checkout", branch_name])
            .current_dir(&self.path)
            .assert()
            .success();
    }

    /// Merge a branch into the current branch
    pub fn merge_branch(&self, branch_name: &str) {
        StdCommand::new("git")
            .args(["merge", branch_name])
            .current_dir(&self.path)
            .assert()
            .success();
    }

    /// Set up a remote repository and push to it
    pub fn setup_remote(&self, branch_name: &str) -> TestRepo {
        let remote_temp = tempdir().unwrap();
        let remote_path = remote_temp.path().to_path_buf();

        StdCommand::new("git")
            .arg("init")
            .arg("--bare")
            .current_dir(&remote_path)
            .assert()
            .success();

        StdCommand::new("git")
            .args(["remote", "add", "origin", remote_path.to_str().unwrap()])
            .current_dir(&self.path)
            .assert()
            .success();

        StdCommand::new("git")
            .args(["push", "-u", "origin", branch_name])
            .current_dir(&self.path)
            .assert()
            .success();

        TestRepo {
            _temp_dir: remote_temp,
            path: remote_path,
        }
    }
}

/// Create a basic Git repository with a single commit
pub fn basic_repo() -> TestRepo {
    let temp = tempdir().unwrap();
    let path = temp.path().to_path_buf();

    StdCommand::new("git")
        .arg("init")
        .current_dir(&path)
        .assert()
        .success();

    let repo = TestRepo {
        _temp_dir: temp,
        path: path.clone(),
    };
    repo.configure_git_identity();

    fs::write(path.join("README.md"), "# test").unwrap();
    StdCommand::new("git")
        .args(["add", "."])
        .current_dir(&path)
        .assert()
        .success();
    StdCommand::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(&path)
        .assert()
        .success();

    repo
}

/// Create a repo with a specific branch name and initial commit
pub fn repo_with_branch(branch_name: &str) -> TestRepo {
    let temp = tempdir().unwrap();
    let path = temp.path().to_path_buf();

    StdCommand::new("git")
        .arg("init")
        .current_dir(&path)
        .assert()
        .success();

    let repo = TestRepo {
        _temp_dir: temp,
        path: path.clone(),
    };
    repo.configure_git_identity();

    StdCommand::new("git")
        .args(["checkout", "-b", branch_name])
        .current_dir(&path)
        .assert()
        .success();

    fs::write(path.join("README.md"), "# test repo").unwrap();
    StdCommand::new("git")
        .args(["add", "."])
        .current_dir(&path)
        .assert()
        .success();
    StdCommand::new("git")
        .args(["commit", "-m", "initial commit"])
        .current_dir(&path)
        .assert()
        .success();

    repo
}

/// Create a repo with multiple commits for testing commit-related commands
pub fn repo_with_commits(count: usize) -> TestRepo {
    let repo = basic_repo();

    for i in 1..count {
        repo.add_commit(
            "file.txt",
            &format!("content {}", i + 1),
            &format!("commit {}", i + 1),
        );
    }

    repo
}

/// Create a repo with a merged feature branch
pub fn repo_with_merged_branch(feature_branch: &str, main_branch: &str) -> TestRepo {
    let temp = tempdir().unwrap();
    let path = temp.path().to_path_buf();

    StdCommand::new("git")
        .args(["init", &format!("--initial-branch={main_branch}")])
        .current_dir(&path)
        .assert()
        .success();

    let repo = TestRepo {
        _temp_dir: temp,
        path: path.clone(),
    };
    repo.configure_git_identity();

    // Initial commit
    fs::write(path.join("README.md"), "# test").unwrap();
    StdCommand::new("git")
        .args(["add", "."])
        .current_dir(&path)
        .assert()
        .success();
    StdCommand::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(&path)
        .assert()
        .success();

    // Create feature branch and add a commit
    StdCommand::new("git")
        .args(["checkout", "-b", feature_branch])
        .current_dir(&path)
        .assert()
        .success();
    fs::write(path.join("feature.txt"), "feature content").unwrap();
    StdCommand::new("git")
        .args(["add", "."])
        .current_dir(&path)
        .assert()
        .success();
    StdCommand::new("git")
        .args(["commit", "-m", "add feature"])
        .current_dir(&path)
        .assert()
        .success();

    // Switch to main branch and merge
    StdCommand::new("git")
        .args(["checkout", main_branch])
        .current_dir(&path)
        .assert()
        .success();
    StdCommand::new("git")
        .args(["merge", feature_branch])
        .current_dir(&path)
        .assert()
        .success();

    // Verify the repo was created successfully with a merged branch

    // 1. Assert we're on the main branch
    let current_branch = TestAssertions::get_git_output(&repo, &["branch", "--show-current"]);
    assert_eq!(
        current_branch, main_branch,
        "Should be on the main branch after merge"
    );

    // 2. Assert the feature branch exists
    let branches = TestAssertions::get_git_output(&repo, &["branch"]);
    assert!(
        branches.contains(feature_branch),
        "Feature branch '{feature_branch}' should exist"
    );

    // 3. Assert the feature branch is merged (shows up in git branch --merged)
    let merged_branches = TestAssertions::get_git_output(&repo, &["branch", "--merged"]);
    assert!(
        merged_branches.contains(feature_branch),
        "Feature branch '{feature_branch}' should be merged"
    );

    // 4. Assert both files from main and feature branch exist
    assert!(
        path.join("README.md").exists(),
        "README.md should exist from main branch"
    );
    assert!(
        path.join("feature.txt").exists(),
        "feature.txt should exist from merged feature branch"
    );

    // 5. Assert we have at least 2 commits (initial + feature)
    let commit_count = TestAssertions::get_git_output(&repo, &["rev-list", "--count", "HEAD"]);
    let count: u32 = commit_count
        .parse()
        .expect("Should be able to parse commit count");
    assert!(
        count >= 2,
        "Should have at least 2 commits (initial + feature), but found {count}"
    );

    repo
}

/// Create a repo with feature branch ahead of main
pub fn repo_with_feature_ahead(feature_branch: &str, _main_branch: &str) -> TestRepo {
    let temp = tempdir().unwrap();
    let path = temp.path().to_path_buf();

    StdCommand::new("git")
        .args(["init", "--initial-branch=main"])
        .current_dir(&path)
        .assert()
        .success();

    let repo = TestRepo {
        _temp_dir: temp,
        path: path.clone(),
    };
    repo.configure_git_identity();

    fs::write(path.join("file.txt"), "initial").unwrap();
    StdCommand::new("git")
        .args(["add", "."])
        .current_dir(&path)
        .assert()
        .success();
    StdCommand::new("git")
        .args(["commit", "-m", "initial commit"])
        .current_dir(&path)
        .assert()
        .success();

    StdCommand::new("git")
        .args(["checkout", "-b", feature_branch])
        .current_dir(&path)
        .assert()
        .success();

    fs::write(path.join("file.txt"), "modified").unwrap();
    StdCommand::new("git")
        .args(["add", "."])
        .current_dir(&path)
        .assert()
        .success();
    StdCommand::new("git")
        .args(["commit", "-m", "modified file"])
        .current_dir(&path)
        .assert()
        .success();

    repo
}

/// Create a repo with conventional commits for summary testing
pub fn repo_with_conventional_commits() -> TestRepo {
    let temp = tempdir().unwrap();
    let path = temp.path().to_path_buf();

    StdCommand::new("git")
        .args(["init"])
        .current_dir(&path)
        .assert()
        .success();

    let repo = TestRepo {
        _temp_dir: temp,
        path: path.clone(),
    };
    repo.configure_git_identity();

    fs::write(path.join("file1.txt"), "Initial").unwrap();
    StdCommand::new("git")
        .args(["add", "."])
        .current_dir(&path)
        .assert()
        .success();
    StdCommand::new("git")
        .args([
            "commit",
            "-m",
            "feat: initial commit",
            "--author=Alice <alice@example.com>",
        ])
        .current_dir(&path)
        .assert()
        .success();

    fs::write(path.join("file2.txt"), "Fix").unwrap();
    StdCommand::new("git")
        .args(["add", "."])
        .current_dir(&path)
        .assert()
        .success();
    StdCommand::new("git")
        .args([
            "commit",
            "-m",
            "fix: bug fix",
            "--author=Bob <bob@example.com>",
        ])
        .current_dir(&path)
        .assert()
        .success();

    repo
}

/// Create repo with remote that has commits ahead (for testing behind status)
pub fn repo_with_remote_ahead(branch_name: &str) -> (TestRepo, TestRepo) {
    let repo = repo_with_branch(branch_name);
    let remote = repo.setup_remote(branch_name);

    // Clone remote to another location and add a commit
    let clone_temp = tempdir().unwrap();
    let clone_path = clone_temp.path();

    StdCommand::new("git")
        .args(["clone", remote.path.to_str().unwrap(), "."])
        .current_dir(clone_path)
        .assert()
        .success();

    StdCommand::new("git")
        .args(["checkout", branch_name])
        .current_dir(clone_path)
        .assert()
        .success();

    // Configure git identity in clone
    StdCommand::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(clone_path)
        .assert()
        .success();
    StdCommand::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(clone_path)
        .assert()
        .success();

    fs::write(clone_path.join("remote_file.txt"), "remote content").unwrap();
    StdCommand::new("git")
        .args(["add", "."])
        .current_dir(clone_path)
        .assert()
        .success();

    StdCommand::new("git")
        .args(["commit", "-m", "remote commit"])
        .current_dir(clone_path)
        .assert()
        .success();

    StdCommand::new("git")
        .args(["push"])
        .current_dir(clone_path)
        .assert()
        .success();

    // Fetch in original repo to see behind status
    StdCommand::new("git")
        .args(["fetch"])
        .current_dir(&repo.path)
        .assert()
        .success();

    (repo, remote)
}

/// Test utilities for common testing patterns
pub struct TestUtils;

impl TestUtils {
    /// Run a test function in a specific directory, automatically restoring the original directory
    pub fn with_current_dir<F, R>(dir: &Path, test_fn: F) -> R
    where
        F: FnOnce() -> R,
    {
        let original_dir = std::env::current_dir().expect("Failed to get current directory");
        std::env::set_current_dir(dir).expect("Failed to change directory");

        let result = test_fn();

        // Always restore directory, even if the test panics
        let _ = std::env::set_current_dir(&original_dir);
        result
    }

    /// Run a git-x command and return the Command for assertions
    pub fn run_git_x_cmd(_args: &[&str]) -> Command {
        let mut cmd = Command::cargo_bin("git-x").expect("Failed to find git-x binary");
        cmd.env("GIT_X_NON_INTERACTIVE", "1");
        cmd
    }

    /// Create a git-x command with args for testing
    pub fn git_x_with_args(args: &[&str]) -> Command {
        let mut cmd = Self::run_git_x_cmd(args);
        cmd.args(args);
        cmd
    }

    /// Test that a command runs without panicking in a repo directory
    pub fn test_command_in_repo<F>(repo: &TestRepo, test_fn: F)
    where
        F: FnOnce(),
    {
        Self::with_current_dir(repo.path(), test_fn)
    }

    /// Test that a command handles non-git directories gracefully
    pub fn test_command_outside_repo<F>(test_fn: F)
    where
        F: FnOnce(),
    {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        Self::with_current_dir(temp_dir.path(), test_fn)
    }
}

/// Enhanced test repository creation with more options
pub struct TestRepoBuilder {
    branch_name: Option<String>,
    commit_count: usize,
    with_remote: bool,
    conventional_commits: bool,
}

impl TestRepoBuilder {
    pub fn new() -> Self {
        Self {
            branch_name: None,
            commit_count: 1,
            with_remote: false,
            conventional_commits: false,
        }
    }

    pub fn with_branch(mut self, branch_name: &str) -> Self {
        self.branch_name = Some(branch_name.to_string());
        self
    }

    pub fn with_commits(mut self, count: usize) -> Self {
        self.commit_count = count;
        self
    }

    pub fn with_remote(mut self) -> Self {
        self.with_remote = true;
        self
    }

    pub fn with_conventional_commits(mut self) -> Self {
        self.conventional_commits = true;
        self
    }

    pub fn build(self) -> TestRepo {
        if self.conventional_commits {
            return repo_with_conventional_commits();
        }

        let repo = if let Some(branch_name) = &self.branch_name {
            repo_with_branch(branch_name)
        } else {
            basic_repo()
        };

        // Add additional commits if requested
        for i in 1..self.commit_count {
            repo.add_commit(
                &format!("file{}.txt", i + 1),
                &format!("content {}", i + 1),
                &format!("commit {}", i + 1),
            );
        }

        if self.with_remote {
            let branch = self.branch_name.as_deref().unwrap_or("main");
            repo.setup_remote(branch);
        }

        repo
    }
}

impl Default for TestRepoBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Assertion helpers for common test patterns
pub struct TestAssertions;

impl TestAssertions {
    /// Assert that output contains expected success indicators
    pub fn assert_success_output(output: &str) {
        assert!(
            output.contains("✅") || output.contains("success"),
            "Expected success indicators in output: {output}"
        );
    }

    /// Assert that output contains expected error indicators
    pub fn assert_error_output(output: &str) {
        assert!(
            output.contains("❌") || output.contains("error") || output.contains("failed"),
            "Expected error indicators in output: {output}"
        );
    }

    /// Assert that a git command was successful in a repo
    pub fn assert_git_command_success(repo: &TestRepo, args: &[&str]) {
        StdCommand::new("git")
            .args(args)
            .current_dir(repo.path())
            .assert()
            .success();
    }

    /// Get git command output as string
    pub fn get_git_output(repo: &TestRepo, args: &[&str]) -> String {
        let output = StdCommand::new("git")
            .args(args)
            .current_dir(repo.path())
            .output()
            .expect("Failed to run git command");

        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }
}

/// Additional repository helper functions
impl TestRepo {
    /// Create a commit and return its hash
    pub fn create_commit_with_hash(&self, filename: &str, content: &str, message: &str) -> String {
        self.add_commit(filename, content, message);
        TestAssertions::get_git_output(self, &["rev-parse", "HEAD"])
    }

    /// Stage specific files (as opposed to add_commit which stages all files)
    pub fn stage_files(&self, files: &[&str]) {
        for file in files {
            StdCommand::new("git")
                .args(["add", file])
                .current_dir(&self.path)
                .assert()
                .success();
        }
    }

    /// Get current commit hash
    pub fn get_current_commit_hash(&self) -> String {
        TestAssertions::get_git_output(self, &["rev-parse", "HEAD"])
    }

    /// Check if there are staged changes
    pub fn has_staged_changes(&self) -> bool {
        let output = TestAssertions::get_git_output(self, &["diff", "--cached", "--name-only"]);
        !output.is_empty()
    }
}
