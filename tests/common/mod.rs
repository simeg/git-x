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
// Clippy seems to think this method
#[allow(dead_code)]
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
        .arg("init")
        .current_dir(&path)
        .assert()
        .success();

    let repo = TestRepo {
        _temp_dir: temp,
        path: path.clone(),
    };
    repo.configure_git_identity();

    // Rename default branch to the requested main branch if it's not "master"
    if main_branch != "master" {
        StdCommand::new("git")
            .args(["checkout", "-b", main_branch])
            .current_dir(&path)
            .assert()
            .success();
    }

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
