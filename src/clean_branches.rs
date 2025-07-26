use std::process::Command;

pub fn run(dry_run: bool) {
    let output = Command::new("git")
        .args(["branch", "--merged"])
        .output()
        .expect("Failed to list merged branches");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let branches: Vec<String> = stdout
        .lines()
        .map(|line| line.trim().trim_start_matches('*').trim().to_string())
        .filter(|branch| branch != "main" && branch != "master" && branch != "develop")
        .collect();

    let mut deleted = Vec::new();

    for branch in branches {
        if dry_run {
            println!("(dry run) Would delete: {branch}");
            deleted.push(branch);
        } else {
            let status = Command::new("git")
                .args(["branch", "-d", &branch])
                .status()
                .expect("Failed to delete branch");

            if status.success() {
                deleted.push(branch);
            }
        }
    }

    if deleted.is_empty() {
        println!("No merged branches to delete.");
    } else {
        if dry_run {
            println!("🧪 (dry run) {} branches would be deleted:", deleted.len());
        } else {
            println!("🧹 Deleted {} merged branches:", deleted.len());
        }
        for branch in deleted {
            println!("  {branch}");
        }
    }
}
