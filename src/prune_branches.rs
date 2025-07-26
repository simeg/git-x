use std::io::{BufRead, BufReader};
use std::process::{Command, exit};

pub fn run(except: Option<String>) {
    let mut protected_branches = vec!["main", "master", "develop"];

    // Extend protected list with user-supplied excluded branches
    if let Some(ref input) = except {
        let extras: Vec<&str> = input
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        protected_branches.extend(extras);
    }

    // Step 1: Get current branch
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .expect("Failed to get current branch");

    if !output.status.success() {
        eprintln!("Error: Could not determine current branch.");
        exit(1);
    }

    let current_branch = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Step 2: Get merged branches
    let output = Command::new("git")
        .args(["branch", "--merged"])
        .output()
        .expect("Failed to get merged branches");

    if !output.status.success() {
        eprintln!("Error: Failed to list merged branches.");
        exit(1);
    }

    let reader = BufReader::new(output.stdout.as_slice());
    let branches: Vec<String> = reader
        .lines()
        .map_while(Result::ok)
        .map(|b| b.trim().trim_start_matches("* ").to_string())
        .filter(|b| b != &current_branch && !protected_branches.iter().any(|pb| pb == b))
        .collect();

    if branches.is_empty() {
        println!("✅ No merged branches to prune.");
        return;
    }

    // Step 3: Delete branches
    for branch in branches {
        let status = Command::new("git")
            .args(["branch", "-d", &branch])
            .status()
            .expect("Failed to delete branch");

        if status.success() {
            println!("🧹 Deleted merged branch '{branch}'");
        } else {
            eprintln!("⚠️ Failed to delete branch '{branch}'");
        }
    }
}
