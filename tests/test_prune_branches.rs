use assert_cmd::Command;
use predicates::boolean::PredicateBooleanExt;
use predicates::str::contains;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_prune_branches_deletes_merged_branch() {
    let repo = setup_repo_with_merged_branch();
    let repo_path = repo.path();

    Command::cargo_bin("git-x")
        .unwrap()
        .args(["prune-branches"])
        .current_dir(repo_path)
        .assert()
        .success()
        .stdout(contains("🧹 Deleted merged branch 'feature/delete-me'"));
}

#[test]
fn test_prune_branches_respects_exclude() {
    let repo = setup_repo_with_merged_branch();
    let repo_path = repo.path();

    // Create another merged branch
    Command::new("git")
        .args(["checkout", "-b", "feature/keep-me"])
        .current_dir(repo_path)
        .assert()
        .success();
    Command::new("git")
        .args(["checkout", "main"])
        .current_dir(repo_path)
        .assert()
        .success();
    Command::new("git")
        .args(["merge", "feature/keep-me"])
        .current_dir(repo_path)
        .assert()
        .success();

    Command::cargo_bin("git-x")
        .unwrap()
        .args(["prune-branches", "--except", "feature/keep-me"])
        .current_dir(repo_path)
        .assert()
        .success()
        .stdout(contains("🧹 Deleted merged branch 'feature/delete-me'"))
        .stdout(contains("✅ No merged branches to prune").not());
}

fn setup_repo_with_merged_branch() -> TempDir {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path();

    Command::new("git")
        .arg("init")
        .current_dir(path)
        .assert()
        .success();

    fs::write(path.join("init.txt"), "arbitrary").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(path)
        .assert()
        .success();
    Command::new("git")
        .args(["commit", "-m", "init"])
        .current_dir(path)
        .assert()
        .success();

    // Create and switch to delete-me branch
    Command::new("git")
        .args(["checkout", "-b", "feature/delete-me"])
        .current_dir(path)
        .assert()
        .success();
    fs::write(path.join("temp.txt"), "arbitrary").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(path)
        .assert()
        .success();
    Command::new("git")
        .args(["commit", "-m", "add temp"])
        .current_dir(path)
        .assert()
        .success();

    // Merge into main
    Command::new("git")
        .args(["checkout", "-b", "main"])
        .current_dir(path)
        .assert()
        .success();
    Command::new("git")
        .args(["merge", "feature/delete-me"])
        .current_dir(path)
        .assert()
        .success();

    temp
}
