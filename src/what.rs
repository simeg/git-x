use std::process::Command;

pub fn run(target: Option<String>) {
    let target_branch = target.unwrap_or_else(get_default_target);

    // Get current branch name
    let current_branch_output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .expect("Failed to get current branch");
    let current_branch = String::from_utf8_lossy(&current_branch_output.stdout)
        .trim()
        .to_string();

    println!(
        "{}",
        format_branch_comparison(&current_branch, &target_branch)
    );

    // Get ahead/behind commit counts
    let rev_list_output = Command::new("git")
        .args([
            "rev-list",
            "--left-right",
            "--count",
            &format_rev_list_range(&target_branch, &current_branch),
        ])
        .output()
        .expect("Failed to get ahead/behind count");
    let output_str = String::from_utf8_lossy(&rev_list_output.stdout);
    let (ahead, behind) = parse_commit_counts(&output_str);

    let (ahead_msg, behind_msg) = format_commit_counts(&ahead, &behind);
    println!("{ahead_msg}");
    println!("{behind_msg}");

    // Get diff summary
    let diff_output = Command::new("git")
        .args([
            "diff",
            "--name-status",
            &format_rev_list_range(&target_branch, &current_branch),
        ])
        .output()
        .expect("Failed to get diff");
    let diff = String::from_utf8_lossy(&diff_output.stdout);

    println!("Changes:");
    for line in diff.lines() {
        if let Some(formatted_line) = format_diff_line(line) {
            println!("{formatted_line}");
        }
    }
}

// Helper function to get default target branch
pub fn get_default_target() -> String {
    "main".to_string()
}

// Helper function to format branch comparison header
pub fn format_branch_comparison(current: &str, target: &str) -> String {
    format!("Branch: {current} vs {target}")
}

// Helper function to format commit counts
pub fn format_commit_counts(ahead: &str, behind: &str) -> (String, String) {
    (
        format!("+ {ahead} commits ahead"),
        format!("- {behind} commits behind"),
    )
}

// Helper function to format rev-list range
pub fn format_rev_list_range(target: &str, current: &str) -> String {
    format!("{target}...{current}")
}

// Helper function to parse ahead/behind counts
pub fn parse_commit_counts(output: &str) -> (String, String) {
    let counts = output.split_whitespace().collect::<Vec<&str>>();
    let behind = counts.first().unwrap_or(&"0").to_string();
    let ahead = counts.get(1).unwrap_or(&"0").to_string();
    (ahead, behind)
}

// Helper function to convert git status to symbol
pub fn git_status_to_symbol(status: &str) -> &str {
    match status {
        "A" => "+",
        "M" => "~",
        "D" => "-",
        other => other,
    }
}

// Helper function to format diff line
pub fn format_diff_line(line: &str) -> Option<String> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 2 {
        let symbol = git_status_to_symbol(parts[0]);
        Some(format!(" - {} {}", symbol, parts[1]))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_default_target() {
        assert_eq!(get_default_target(), "main");
    }

    #[test]
    fn test_format_branch_comparison() {
        assert_eq!(
            format_branch_comparison("feature", "main"),
            "Branch: feature vs main"
        );
        assert_eq!(
            format_branch_comparison("develop", "master"),
            "Branch: develop vs master"
        );
    }

    #[test]
    fn test_format_commit_counts() {
        let (ahead, behind) = format_commit_counts("3", "1");
        assert_eq!(ahead, "+ 3 commits ahead");
        assert_eq!(behind, "- 1 commits behind");

        let (ahead, behind) = format_commit_counts("0", "5");
        assert_eq!(ahead, "+ 0 commits ahead");
        assert_eq!(behind, "- 5 commits behind");
    }

    #[test]
    fn test_format_rev_list_range() {
        assert_eq!(format_rev_list_range("main", "feature"), "main...feature");
        assert_eq!(
            format_rev_list_range("develop", "hotfix"),
            "develop...hotfix"
        );
    }

    #[test]
    fn test_parse_commit_counts() {
        assert_eq!(
            parse_commit_counts("2 3"),
            ("3".to_string(), "2".to_string())
        );
        assert_eq!(
            parse_commit_counts("0 1"),
            ("1".to_string(), "0".to_string())
        );
        assert_eq!(parse_commit_counts("5"), ("0".to_string(), "5".to_string()));
        assert_eq!(parse_commit_counts(""), ("0".to_string(), "0".to_string()));
    }

    #[test]
    fn test_git_status_to_symbol() {
        assert_eq!(git_status_to_symbol("A"), "+");
        assert_eq!(git_status_to_symbol("M"), "~");
        assert_eq!(git_status_to_symbol("D"), "-");
        assert_eq!(git_status_to_symbol("R"), "R");
        assert_eq!(git_status_to_symbol("C"), "C");
    }

    #[test]
    fn test_format_diff_line() {
        assert_eq!(
            format_diff_line("A\tfile.txt"),
            Some(" - + file.txt".to_string())
        );
        assert_eq!(
            format_diff_line("M\tsrc/main.rs"),
            Some(" - ~ src/main.rs".to_string())
        );
        assert_eq!(
            format_diff_line("D\told.txt"),
            Some(" - - old.txt".to_string())
        );
        assert_eq!(format_diff_line("A"), None);
        assert_eq!(format_diff_line(""), None);
    }
}
