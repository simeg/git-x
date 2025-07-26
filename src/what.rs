use std::process::Command;

pub fn run(target: Option<String>) {
    let target_branch = target.unwrap_or_else(|| "main".to_string());

    // Get current branch name
    let current_branch_output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .expect("Failed to get current branch");
    let current_branch = String::from_utf8_lossy(&current_branch_output.stdout)
        .trim()
        .to_string();

    println!("Branch: {current_branch} vs {target_branch}");

    // Get ahead/behind commit counts
    let rev_list_output = Command::new("git")
        .args([
            "rev-list",
            "--left-right",
            "--count",
            &format!("{target_branch}...{current_branch}"),
        ])
        .output()
        .expect("Failed to get ahead/behind count");
    let output_str = String::from_utf8_lossy(&rev_list_output.stdout);
    let counts = output_str.split_whitespace().collect::<Vec<&str>>();
    let behind = counts.first().unwrap_or(&"0");
    let ahead = counts.get(1).unwrap_or(&"0");

    println!("+ {ahead} commits ahead");
    println!("- {behind} commits behind");

    // Get diff summary
    let diff_output = Command::new("git")
        .args([
            "diff",
            "--name-status",
            &format!("{target_branch}...{current_branch}"),
        ])
        .output()
        .expect("Failed to get diff");
    let diff = String::from_utf8_lossy(&diff_output.stdout);

    println!("Changes:");
    for line in diff.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let symbol = match parts[0] {
                "A" => "+",
                "M" => "~",
                "D" => "-",
                other => other,
            };
            println!(" - {} {}", symbol, parts[1]);
        }
    }
}
