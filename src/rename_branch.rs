use std::process::{Command, exit};

pub fn run(new_name: &str) {
    // Step 1: Get current branch name
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .expect("Failed to execute git");

    if !output.status.success() {
        eprintln!("Error: Failed to get current branch name.");
        exit(1);
    }

    let current_branch = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if current_branch == new_name {
        println!("Current branch is already named '{new_name}'. Nothing to do.");
        return;
    }

    println!("Renaming branch '{current_branch}' to '{new_name}'");

    // Step 2: Rename branch locally
    let status = Command::new("git")
        .args(["branch", "-m", new_name])
        .status()
        .expect("Failed to rename branch");

    if !status.success() {
        eprintln!("Error: Failed to rename local branch.");
        exit(1);
    }

    // Step 3: Push the new branch to origin
    let status = Command::new("git")
        .args(["push", "-u", "origin", new_name])
        .status()
        .expect("Failed to push new branch");

    if !status.success() {
        eprintln!("Error: Failed to push new branch to origin.");
        exit(1);
    }

    // Step 4: Delete the old branch from origin
    let status = Command::new("git")
        .args(["push", "origin", "--delete", &current_branch])
        .status()
        .expect("Failed to delete old branch");

    if !status.success() {
        eprintln!("Warning: Failed to delete old branch '{current_branch}' from origin.");
    } else {
        println!("Deleted old branch '{current_branch}' from origin.");
    }

    println!("Branch renamed successfully.");
}
