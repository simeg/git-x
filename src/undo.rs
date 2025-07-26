use std::process::Command;

pub fn run() {
    let status = Command::new("git")
        .args(["reset", "--soft", "HEAD~1"])
        .status()
        .expect("Failed to execute git reset");

    if status.success() {
        println!("Last commit undone (soft reset). Changes kept in working directory.");
    } else {
        eprintln!("❌ Failed to undo last commit.");
    }
}
