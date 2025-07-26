use assert_cmd::Command;
use std::fs;
use std::path::Path;

#[test]
fn test_git_xsummary_shows_grouped_commits() {
    let dir = tempfile::tempdir().unwrap(); // keep dir alive
    let repo = init_test_repo_with_commits(dir.path());

    Command::cargo_bin("git-x")
        .unwrap()
        .current_dir(&repo)
        .args(["summary", "--since", "3 days ago"])
        .assert()
        .success()
        .stdout(predicates::str::contains("🗞️ Commit summary since"))
        .stdout(predicates::str::contains("📅"))
        .stdout(predicates::str::contains("✨ feat: initial commit"))
        .stdout(predicates::str::contains("🐛 fix: bug fix"));
}

fn init_test_repo_with_commits(path: &Path) -> std::path::PathBuf {
    let repo_path = path.to_path_buf();
    std::env::set_current_dir(&repo_path).unwrap();

    // Initialize repo and create commits with different messages
    Command::new("git")
        .args(["init"])
        .current_dir(&repo_path)
        .assert()
        .success();

    fs::write(repo_path.join("file1.txt"), "Initial").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(&repo_path)
        .assert()
        .success();
    Command::new("git")
        .args([
            "commit",
            "-m",
            "feat: initial commit",
            "--author=Alice <alice@example.com>",
        ])
        .current_dir(&repo_path)
        .assert()
        .success();

    fs::write(repo_path.join("file2.txt"), "Fix").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(&repo_path)
        .assert()
        .success();
    Command::new("git")
        .args([
            "commit",
            "-m",
            "fix: bug fix",
            "--author=Bob <bob@example.com>",
        ])
        .current_dir(&repo_path)
        .assert()
        .success();

    repo_path
}
