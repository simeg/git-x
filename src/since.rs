use std::process::Command;

pub fn run(reference: String) {
    let output = Command::new("git")
        .args([
            "log",
            &format!("{reference}..HEAD"),
            "--pretty=format:- %h %s",
        ])
        .output()
        .expect("Failed to run git log");

    if !output.status.success() {
        eprintln!("❌ Failed to retrieve commits since '{reference}'");
        return;
    }

    let log = String::from_utf8_lossy(&output.stdout);
    if log.trim().is_empty() {
        println!("✅ No new commits since {reference}");
    } else {
        println!("🔍 Commits since {reference}:");
        println!("{log}");
    }
}
