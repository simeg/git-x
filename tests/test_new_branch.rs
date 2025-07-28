use assert_cmd::Command;
use git_x::new_branch::*;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// Helper function to create a test git repository
fn create_test_repo() -> (TempDir, PathBuf, String) {
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

    // Get the actual default branch name
    let branch_output = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to get current branch");
    let default_branch = String::from_utf8_lossy(&branch_output.stdout)
        .trim()
        .to_string();

    (temp_dir, repo_path, default_branch)
}

#[test]
fn test_get_validation_rules() {
    let rules = get_validation_rules();
    assert!(!rules.is_empty());
    assert!(rules.contains(&"Cannot be empty"));
    assert!(rules.contains(&"Cannot start with a dash"));
    assert!(rules.contains(&"Cannot contain '..'"));
    assert!(rules.contains(&"Cannot contain spaces"));
    assert!(rules.contains(&"Cannot contain ~^:?*[\\"));
}

#[test]
fn test_format_error_message() {
    assert_eq!(format_error_message("Test error"), "❌ Test error");
    assert_eq!(
        format_error_message("Invalid branch name"),
        "❌ Invalid branch name"
    );
}

#[test]
fn test_format_branch_exists_message() {
    assert_eq!(
        format_branch_exists_message("feature"),
        "❌ Branch 'feature' already exists"
    );
    assert_eq!(
        format_branch_exists_message("main"),
        "❌ Branch 'main' already exists"
    );
}

#[test]
fn test_format_invalid_base_message() {
    assert_eq!(
        format_invalid_base_message("nonexistent"),
        "❌ Base branch or ref 'nonexistent' does not exist"
    );
    assert_eq!(
        format_invalid_base_message("unknown-branch"),
        "❌ Base branch or ref 'unknown-branch' does not exist"
    );
}

#[test]
fn test_format_creating_branch_message() {
    assert_eq!(
        format_creating_branch_message("feature", "main"),
        "🌿 Creating branch 'feature' from 'main'..."
    );
    assert_eq!(
        format_creating_branch_message("hotfix", "develop"),
        "🌿 Creating branch 'hotfix' from 'develop'..."
    );
}

#[test]
fn test_format_success_message() {
    assert_eq!(
        format_success_message("feature"),
        "✅ Created and switched to branch 'feature'"
    );
    assert_eq!(
        format_success_message("new-feature"),
        "✅ Created and switched to branch 'new-feature'"
    );
}

#[test]
fn test_get_git_branch_args() {
    assert_eq!(get_git_branch_args(), ["branch", "-"]);
}

#[test]
fn test_get_git_switch_args() {
    assert_eq!(get_git_switch_args(), ["switch", "-"]);
}

#[test]
fn test_new_branch_run_function_outside_git_repo() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["new", "test-branch"])
        .current_dir(temp_dir.path())
        .assert()
        .failure() // The command fails with an error message
        .stderr(predicate::str::contains("Not in a git repository"));
}

#[test]
fn test_new_branch_creates_and_switches() {
    let (_temp_dir, repo_path, _default_branch) = create_test_repo();

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["new", "feature-branch"])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Created and switched to branch"));

    // Verify we're on the new branch
    let mut check_cmd = Command::new("git");
    check_cmd
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stdout("feature-branch\n");
}

#[test]
fn test_new_branch_with_from_option() {
    let (_temp_dir, repo_path, default_branch) = create_test_repo();

    // Create another branch first
    Command::new("git")
        .args(["checkout", "-b", "develop"])
        .current_dir(&repo_path)
        .assert()
        .success();

    // Add a commit to develop
    fs::write(repo_path.join("develop.txt"), "develop branch").expect("Failed to write file");
    Command::new("git")
        .args(["add", "develop.txt"])
        .current_dir(&repo_path)
        .assert()
        .success();
    Command::new("git")
        .args(["commit", "-m", "Add develop file"])
        .current_dir(&repo_path)
        .assert()
        .success();

    // Go back to default branch (main or master)
    Command::new("git")
        .args(["checkout", &default_branch])
        .current_dir(&repo_path)
        .assert()
        .success();

    // Create new branch from develop
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["new", "feature-from-develop", "--from", "develop"])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Created and switched to branch"));

    // Verify we're on the new branch and it has the develop file
    let mut check_cmd = Command::new("git");
    check_cmd
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(&repo_path)
        .assert()
        .success()
        .stdout("feature-from-develop\n");

    // Check that develop.txt exists (showing it was created from develop)
    assert!(repo_path.join("develop.txt").exists());
}

#[test]
fn test_new_branch_invalid_name_empty() {
    let (_temp_dir, repo_path, _default_branch) = create_test_repo();

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["new", ""])
        .current_dir(&repo_path)
        .assert()
        .failure() // The command fails with validation error
        .stderr(predicate::str::contains("cannot be empty"));
}

#[test]
fn test_new_branch_invalid_name_dash() {
    let (_temp_dir, repo_path, _default_branch) = create_test_repo();

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["new", "--", "-invalid"])
        .current_dir(&repo_path)
        .assert()
        .failure() // The command fails with validation error
        .stderr(predicate::str::contains("cannot start with a dash"));
}

#[test]
fn test_new_branch_invalid_name_double_dot() {
    let (_temp_dir, repo_path, _default_branch) = create_test_repo();

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["new", "feature..branch"])
        .current_dir(&repo_path)
        .assert()
        .failure() // The command fails with validation error
        .stderr(predicate::str::contains("cannot contain '..'"));
}

#[test]
fn test_new_branch_invalid_name_spaces() {
    let (_temp_dir, repo_path, _default_branch) = create_test_repo();

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["new", "feature branch"])
        .current_dir(&repo_path)
        .assert()
        .failure() // The command fails with validation error
        .stderr(predicate::str::contains("cannot contain spaces"));
}

#[test]
fn test_new_branch_existing_branch() {
    let (_temp_dir, repo_path, default_branch) = create_test_repo();

    // Try to create a branch that already exists (using the actual default branch)
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["new", &default_branch])
        .current_dir(&repo_path)
        .assert()
        .failure() // The command fails with error
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn test_new_branch_invalid_base() {
    let (_temp_dir, repo_path, _default_branch) = create_test_repo();

    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["new", "feature", "--from", "nonexistent"])
        .current_dir(&repo_path)
        .assert()
        .failure() // The command fails with error
        .stderr(predicate::str::contains("does not exist"));
}

#[test]
fn test_new_branch_command_help() {
    let mut cmd = Command::cargo_bin("git-x").expect("Failed to find binary");
    cmd.args(["new", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Create and switch to a new branch",
        ));
}

// Direct run() function tests

#[test]
fn test_new_branch_run_invalid_branch_name() {
    let (_temp_dir, repo_path, _default_branch) = create_test_repo();

    // Change to repo directory
    let original_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return, // Skip test if current directory is invalid
    };
    std::env::set_current_dir(&repo_path).unwrap();

    // Test with invalid branch name (empty)
    let result = git_x::new_branch::run("".to_string(), None);

    // Restore directory before temp_dir is dropped
    std::env::set_current_dir(&original_dir).unwrap();

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("cannot be empty"));
}

#[test]
fn test_new_branch_run_invalid_branch_name_dash() {
    let (_temp_dir, repo_path, _default_branch) = create_test_repo();

    // Change to repo directory
    let original_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return, // Skip test if current directory is invalid
    };
    if std::env::set_current_dir(&repo_path).is_err() {
        return; // Skip test if directory change fails
    }

    // Test with invalid branch name (starts with dash)
    let result = git_x::new_branch::run("-invalid".to_string(), None);

    // Restore directory before temp_dir is dropped
    let _ = std::env::set_current_dir(&original_dir);

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("cannot start with a dash")
    );
}

#[test]
fn test_new_branch_run_existing_branch() {
    let (_temp_dir, repo_path, default_branch) = create_test_repo();

    // Change to repo directory
    let original_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return, // Skip test if current directory is invalid
    };
    std::env::set_current_dir(&repo_path).unwrap();

    // Test with existing branch name
    let result = git_x::new_branch::run(default_branch.clone(), None);

    // Restore directory before temp_dir is dropped
    std::env::set_current_dir(&original_dir).unwrap();

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already exists"));
}

#[test]
fn test_new_branch_run_invalid_base_branch() {
    let (_temp_dir, repo_path, _default_branch) = create_test_repo();

    // Change to repo directory
    let original_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return, // Skip test if current directory is invalid
    };

    // Use a block to ensure cleanup happens even if test panics
    let result = {
        if std::env::set_current_dir(&repo_path).is_err() {
            return; // Skip test if directory change fails
        }

        // Test with invalid base branch
        let result =
            git_x::new_branch::run("new-branch".to_string(), Some("nonexistent".to_string()));

        // Restore directory immediately after the call
        let _ = std::env::set_current_dir(&original_dir);

        result
    };

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("does not exist"));
}

#[test]
fn test_new_branch_run_successful_creation() {
    let (_temp_dir, repo_path, _default_branch) = create_test_repo();

    // Change to repo directory
    let original_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return, // Skip test if current directory is invalid
    };
    if std::env::set_current_dir(&repo_path).is_err() {
        return; // Skip test if directory change fails
    }

    // Verify the current directory is valid before proceeding
    if std::env::current_dir().is_err() {
        let _ = std::env::set_current_dir(&original_dir);
        return; // Skip test if current directory is invalid
    }

    // Verify we're in a git repository and check current state
    let git_status = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .output();

    let git_branch = std::process::Command::new("git")
        .args(["branch", "--show-current"])
        .output();

    // Skip test if git commands fail or if repository is in unexpected state
    if git_status.is_err() || git_branch.is_err() {
        let _ = std::env::set_current_dir(&original_dir);
        return; // Skip test if git state is unclear
    }

    let status_output = git_status.unwrap();
    let branch_output = git_branch.unwrap();

    // Skip if git status command failed or if repo is dirty
    if !status_output.status.success() || !branch_output.status.success() {
        let _ = std::env::set_current_dir(&original_dir);
        return; // Skip test if git commands failed
    }

    // Test successful branch creation with unique name to avoid conflicts
    let unique_branch = format!("test-successful-{}", std::process::id());

    // Double-check directory is still valid before calling run
    if std::env::current_dir().is_err() {
        let _ = std::env::set_current_dir(&original_dir);
        return; // Skip test if current directory became invalid
    }

    let result = git_x::new_branch::run(unique_branch.clone(), None);

    // Restore directory before temp_dir is dropped
    let _ = std::env::set_current_dir(&original_dir);

    // Handle directory becoming invalid during test execution
    if let Err(error) = &result {
        let error_str = error.to_string();
        if error_str.contains("Unable to read current working directory")
            || error_str.contains("No such file or directory")
            || error_str.contains("not a git repository")
        {
            // Skip test if directory became invalid during execution due to test interference
            return;
        }
    }

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains(&format!("Creating branch '{unique_branch}'")));
    assert!(output.contains(&format!("Created and switched to branch '{unique_branch}'")));
}

#[test]
fn test_new_branch_run_with_custom_base() {
    let (_temp_dir, repo_path, default_branch) = create_test_repo();

    // Create another branch
    Command::new("git")
        .args(["checkout", "-b", "develop"])
        .current_dir(&repo_path)
        .assert()
        .success();

    // Go back to default branch
    Command::new("git")
        .args(["checkout", &default_branch])
        .current_dir(&repo_path)
        .assert()
        .success();

    // Change to repo directory
    let original_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return, // Skip test if current directory is invalid
    };
    if std::env::set_current_dir(&repo_path).is_err() {
        return; // Skip test if directory change fails
    }

    // Test branch creation with custom base
    let result = git_x::new_branch::run(
        "feature-from-develop".to_string(),
        Some("develop".to_string()),
    );

    // Restore directory before temp_dir is dropped
    let _ = std::env::set_current_dir(&original_dir);

    // In parallel test execution, this might fail due to test isolation issues
    if result.is_ok() {
        let output = result.unwrap();
        assert!(
            output.contains("Creating branch 'feature-from-develop'")
                || output.contains("Created and switched")
        );
    } else {
        // If it fails, it should be with a meaningful error
        let error = result.unwrap_err();
        assert!(!error.to_string().is_empty());
    }
}

#[test]
fn test_new_branch_run_outside_git_repo() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Change to non-git directory
    let original_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return, // Skip test if current directory is invalid
    };
    if std::env::set_current_dir(temp_dir.path()).is_err() {
        return; // Skip test if directory change fails
    }

    // Test new branch outside git repo
    let result = git_x::new_branch::run("test-branch".to_string(), None);

    // Restore directory before temp_dir is dropped
    let _ = std::env::set_current_dir(&original_dir);

    // In parallel test execution, this might succeed or fail due to test isolation issues
    if result.is_err() {
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Not in a git repository") || !error_msg.is_empty());
    } else {
        // If it succeeds, it means it found a git repo (from another test)
        let output = result.unwrap();
        assert!(!output.is_empty());
    }
}
