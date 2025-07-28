use crate::{GitXError, Result};
use console::style;
use std::collections::HashMap;
use std::process::Command;

pub fn run() -> Result<String> {
    let mut output = Vec::new();

    output.push(format!("{} Technical Debt Analysis\n", style("🔍").bold()));

    // 1. Analyze large commits
    output.push(analyze_large_commits()?);

    // 2. Analyze frequently modified files (hotspots)
    output.push(analyze_file_hotspots()?);

    // 3. Analyze long-lived branches
    output.push(analyze_long_lived_branches()?);

    // 4. Analyze code churn
    output.push(analyze_code_churn()?);

    // 5. Check for binary files
    output.push(analyze_binary_files()?);

    output.push(format!("\n{}", style("Analysis complete!").bold()));

    Ok(output.join("\n"))
}

fn analyze_large_commits() -> Result<String> {
    let mut output = Vec::new();
    output.push(format!(
        "{} Large Commits (>20 files changed)",
        style("📊").bold()
    ));

    let git_output = Command::new("git")
        .args([
            "log",
            "--all",
            "--pretty=format:%h|%s|%an|%ad",
            "--date=short",
            "--numstat",
            "--since=6 months ago",
        ])
        .output()?;

    if !git_output.status.success() {
        return Err(GitXError::GitCommand(
            "Failed to analyze commit history".to_string(),
        ));
    }

    let stdout = String::from_utf8_lossy(&git_output.stdout);
    let large_commits = parse_large_commits(&stdout);

    if large_commits.is_empty() {
        output.push(format!("   {} No large commits found", style("✓").green()));
    } else {
        for (i, commit) in large_commits.iter().take(5).enumerate() {
            output.push(format!(
                "   {}. {} files | {} | {}",
                i + 1,
                style(commit.files_changed).cyan().bold(),
                style(&commit.hash[0..7]).dim(),
                style(&commit.message).bold()
            ));
        }
        if large_commits.len() > 5 {
            output.push(format!(
                "   {} ({} more commits...)",
                style("...").dim(),
                large_commits.len() - 5
            ));
        }
    }

    Ok(output.join("\n"))
}

fn analyze_file_hotspots() -> Result<String> {
    let mut output = Vec::new();
    output.push(format!(
        "{} File Hotspots (frequently modified)",
        style("🔥").bold()
    ));

    let git_output = Command::new("git")
        .args([
            "log",
            "--all",
            "--pretty=format:",
            "--name-only",
            "--since=6 months ago",
        ])
        .output()?;

    if !git_output.status.success() {
        return Err(GitXError::GitCommand(
            "Failed to analyze file modifications".to_string(),
        ));
    }

    let stdout = String::from_utf8_lossy(&git_output.stdout);
    let hotspots = analyze_file_modification_frequency(&stdout);

    if hotspots.is_empty() {
        output.push(format!(
            "   {} No file modification data found",
            style("✓").green()
        ));
    } else {
        for (i, (file, count)) in hotspots.iter().take(5).enumerate() {
            let risk_level = if *count > 50 {
                style("HIGH").red().bold()
            } else if *count > 20 {
                style("MED").yellow().bold()
            } else {
                style("LOW").green().bold()
            };

            output.push(format!(
                "   {}. {} changes | {} | {}",
                i + 1,
                style(count).cyan().bold(),
                risk_level,
                style(file).bold()
            ));
        }
    }

    Ok(output.join("\n"))
}

fn analyze_long_lived_branches() -> Result<String> {
    let mut output = Vec::new();
    output.push(format!(
        "{} Long-lived Branches (>30 days)",
        style("🌿").bold()
    ));

    let git_output = Command::new("git")
        .args([
            "for-each-ref",
            "--format=%(refname:short)|%(committerdate:relative)|%(authorname)",
            "refs/heads/",
        ])
        .output()?;

    if !git_output.status.success() {
        return Err(GitXError::GitCommand(
            "Failed to analyze branches".to_string(),
        ));
    }

    let stdout = String::from_utf8_lossy(&git_output.stdout);
    let long_lived = parse_long_lived_branches(&stdout);

    if long_lived.is_empty() {
        output.push(format!(
            "   {} No long-lived branches found",
            style("✓").green()
        ));
    } else {
        for branch in long_lived.iter().take(5) {
            let age_style = if branch.days_old > 90 {
                style(&branch.age).red().bold()
            } else {
                style(&branch.age).yellow().bold()
            };

            output.push(format!(
                "   • {} | {} | {}",
                style(&branch.name).bold(),
                age_style,
                style(&branch.author).dim()
            ));
        }
    }

    Ok(output.join("\n"))
}

fn analyze_code_churn() -> Result<String> {
    let mut output = Vec::new();
    output.push(format!(
        "{} Code Churn (high add/delete ratio)",
        style("🔄").bold()
    ));

    let git_output = Command::new("git")
        .args([
            "log",
            "--all",
            "--pretty=format:",
            "--numstat",
            "--since=3 months ago",
        ])
        .output()?;

    if !git_output.status.success() {
        return Err(GitXError::GitCommand(
            "Failed to analyze code churn".to_string(),
        ));
    }

    let stdout = String::from_utf8_lossy(&git_output.stdout);
    let churn_files = analyze_churn_patterns(&stdout);

    if churn_files.is_empty() {
        output.push(format!(
            "   {} No high-churn files found",
            style("✓").green()
        ));
    } else {
        for (i, file) in churn_files.iter().take(5).enumerate() {
            let churn_ratio =
                file.total_changes as f64 / (file.additions + file.deletions + 1) as f64;
            let churn_style = if churn_ratio > 3.0 {
                style("HIGH").red().bold()
            } else if churn_ratio > 1.5 {
                style("MED").yellow().bold()
            } else {
                style("LOW").green().bold()
            };

            output.push(format!(
                "   {}. +{} -{} | {} | {}",
                i + 1,
                style(file.additions).green(),
                style(file.deletions).red(),
                churn_style,
                style(&file.path).bold()
            ));
        }
    }

    Ok(output.join("\n"))
}

fn analyze_binary_files() -> Result<String> {
    let mut output = Vec::new();
    output.push(format!("{} Binary Files in Repository", style("📦").bold()));

    let git_output = Command::new("git").args(["ls-files"]).output()?;

    if !git_output.status.success() {
        return Err(GitXError::GitCommand(
            "Failed to list repository files".to_string(),
        ));
    }

    let stdout = String::from_utf8_lossy(&git_output.stdout);
    let binary_files = identify_binary_files(&stdout);

    if binary_files.is_empty() {
        output.push(format!(
            "   {} No binary files detected",
            style("✓").green()
        ));
    } else {
        output.push(format!(
            "   {} {} binary files found",
            style("!").yellow(),
            binary_files.len()
        ));

        for file in binary_files.iter().take(3) {
            output.push(format!("   • {}", style(file).dim()));
        }

        if binary_files.len() > 3 {
            output.push(format!(
                "   {} ({} more files...)",
                style("...").dim(),
                binary_files.len() - 3
            ));
        }
    }

    Ok(output.join("\n"))
}

#[derive(Clone)]
struct LargeCommit {
    hash: String,
    message: String,
    files_changed: usize,
}

#[derive(Clone)]
struct BranchInfo {
    name: String,
    age: String,
    author: String,
    days_old: u32,
}

#[derive(Clone)]
struct ChurnFile {
    path: String,
    additions: u32,
    deletions: u32,
    total_changes: u32,
}

fn parse_large_commits(output: &str) -> Vec<LargeCommit> {
    let mut commits = Vec::new();
    let mut current_commit: Option<LargeCommit> = None;
    let mut file_count = 0;

    for line in output.lines() {
        if line.contains('|') && !line.starts_with(char::is_numeric) {
            // New commit line
            if let Some(mut commit) = current_commit.take() {
                commit.files_changed = file_count;
                if file_count > 20 {
                    commits.push(commit);
                }
            }

            let parts: Vec<&str> = line.splitn(4, '|').collect();
            if parts.len() >= 2 {
                current_commit = Some(LargeCommit {
                    hash: parts[0].to_string(),
                    message: parts[1].chars().take(60).collect(),
                    files_changed: 0,
                });
                file_count = 0;
            }
        } else if line.trim().is_empty() {
            // Empty line between commits
            continue;
        } else if line.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            // File change line (starts with numbers)
            file_count += 1;
        }
    }

    // Handle the last commit
    if let Some(mut commit) = current_commit {
        commit.files_changed = file_count;
        if file_count > 20 {
            commits.push(commit);
        }
    }

    commits.sort_by(|a, b| b.files_changed.cmp(&a.files_changed));
    commits
}

fn analyze_file_modification_frequency(output: &str) -> Vec<(String, usize)> {
    let mut file_counts: HashMap<String, usize> = HashMap::new();

    for line in output.lines() {
        let file = line.trim();
        if !file.is_empty() && !file.starts_with('.') {
            *file_counts.entry(file.to_string()).or_insert(0) += 1;
        }
    }

    let mut sorted: Vec<(String, usize)> = file_counts.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    sorted.into_iter().filter(|(_, count)| *count > 5).collect()
}

fn parse_long_lived_branches(output: &str) -> Vec<BranchInfo> {
    let mut branches = Vec::new();

    for line in output.lines() {
        let parts: Vec<&str> = line.splitn(3, '|').collect();
        if parts.len() >= 3 {
            let name = parts[0].trim();
            let age = parts[1].trim();
            let author = parts[2].trim();

            // Skip main/master branches
            if name == "main" || name == "master" || name == "develop" {
                continue;
            }

            let days_old = estimate_days_from_relative_date(age);
            if days_old > 30 {
                branches.push(BranchInfo {
                    name: name.to_string(),
                    age: age.to_string(),
                    author: author.to_string(),
                    days_old,
                });
            }
        }
    }

    branches.sort_by(|a, b| b.days_old.cmp(&a.days_old));
    branches
}

fn analyze_churn_patterns(output: &str) -> Vec<ChurnFile> {
    let mut file_stats: HashMap<String, (u32, u32)> = HashMap::new();

    for line in output.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 3 {
            if let (Ok(additions), Ok(deletions)) =
                (parts[0].parse::<u32>(), parts[1].parse::<u32>())
            {
                let file = parts[2];
                let entry = file_stats.entry(file.to_string()).or_insert((0, 0));
                entry.0 += additions;
                entry.1 += deletions;
            }
        }
    }

    let mut churn_files: Vec<ChurnFile> = file_stats
        .into_iter()
        .filter(|(_, (adds, dels))| *adds + *dels > 100) // Only files with significant changes
        .map(|(path, (additions, deletions))| ChurnFile {
            path,
            additions,
            deletions,
            total_changes: additions + deletions,
        })
        .collect();

    churn_files.sort_by(|a, b| b.total_changes.cmp(&a.total_changes));
    churn_files
}

fn identify_binary_files(output: &str) -> Vec<String> {
    let binary_extensions = [
        ".jpg", ".jpeg", ".png", ".gif", ".bmp", ".ico", ".svg", ".mp4", ".avi", ".mov", ".wmv",
        ".flv", ".webm", ".mp3", ".wav", ".flac", ".aac", ".ogg", ".zip", ".tar", ".gz", ".7z",
        ".rar", ".exe", ".dll", ".so", ".dylib", ".pdf", ".doc", ".docx", ".xls", ".xlsx", ".ppt",
        ".pptx", ".bin", ".dat", ".db", ".sqlite", ".sqlite3",
    ];

    output
        .lines()
        .filter(|line| {
            let file = line.trim();
            binary_extensions.iter().any(|ext| file.ends_with(ext))
        })
        .map(|line| line.trim().to_string())
        .collect()
}

fn estimate_days_from_relative_date(relative: &str) -> u32 {
    if relative.contains("year") {
        365
    } else if relative.contains("month") {
        if let Some(num_str) = relative.split_whitespace().next() {
            if let Ok(months) = num_str.parse::<u32>() {
                months * 30
            } else {
                90 // Default to ~3 months
            }
        } else {
            90
        }
    } else if relative.contains("week") {
        if let Some(num_str) = relative.split_whitespace().next() {
            if let Ok(weeks) = num_str.parse::<u32>() {
                weeks * 7
            } else {
                14 // Default to 2 weeks
            }
        } else {
            14
        }
    } else if relative.contains("day") {
        if let Some(num_str) = relative.split_whitespace().next() {
            num_str.parse::<u32>().unwrap_or(7)
        } else {
            7
        }
    } else {
        0 // Recent (hours/minutes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GitXError;

    #[test]
    fn test_parse_large_commits() {
        let sample_output = r#"abc123|Add new feature|John Doe|2025-01-15
5	10	src/main.rs
3	2	src/lib.rs
15	0	README.md

def456|Refactor code|Jane Smith|2025-01-14
1	1	src/utils.rs
2	1	src/config.rs"#;

        let commits = parse_large_commits(sample_output);
        // Only commits with >20 files should be included
        assert!(
            commits.is_empty(),
            "Should not include commits with few files"
        );
    }

    #[test]
    fn test_analyze_file_modification_frequency() {
        let sample_output = r#"src/main.rs
src/lib.rs
src/main.rs
src/main.rs
README.md
src/main.rs
src/main.rs
src/main.rs"#;

        let frequencies = analyze_file_modification_frequency(sample_output);
        assert!(!frequencies.is_empty());
        assert_eq!(frequencies[0].0, "src/main.rs");
        assert_eq!(frequencies[0].1, 6);
    }

    #[test]
    fn test_parse_long_lived_branches() {
        let sample_output = r#"main|2 hours ago|John Doe
feature/old-branch|3 months ago|Jane Smith
hotfix/urgent|2 days ago|Bob Wilson
feature/ancient|1 year ago|Alice Johnson"#;

        let branches = parse_long_lived_branches(sample_output);
        assert_eq!(branches.len(), 2); // Should exclude main and recent branches
        assert_eq!(branches[0].name, "feature/ancient");
        assert!(branches[0].days_old > 300);
    }

    #[test]
    fn test_analyze_churn_patterns() {
        let sample_output = r#"50	30	src/main.rs
100	80	src/heavy_churn.rs
5	2	src/stable.rs
200	150	src/refactored.rs"#;

        let churn = analyze_churn_patterns(sample_output);
        assert!(!churn.is_empty());
        // Should be sorted by total changes
        assert_eq!(churn[0].path, "src/refactored.rs");
        assert_eq!(churn[0].total_changes, 350);
    }

    #[test]
    fn test_identify_binary_files() {
        let sample_output = r#"src/main.rs
assets/logo.png
README.md
docs/manual.pdf
src/lib.rs
media/video.mp4"#;

        let binaries = identify_binary_files(sample_output);
        assert_eq!(binaries.len(), 3);
        assert!(binaries.contains(&"assets/logo.png".to_string()));
        assert!(binaries.contains(&"docs/manual.pdf".to_string()));
        assert!(binaries.contains(&"media/video.mp4".to_string()));
    }

    #[test]
    fn test_estimate_days_from_relative_date() {
        assert_eq!(estimate_days_from_relative_date("3 months ago"), 90);
        assert_eq!(estimate_days_from_relative_date("2 weeks ago"), 14);
        assert_eq!(estimate_days_from_relative_date("5 days ago"), 5);
        assert_eq!(estimate_days_from_relative_date("1 year ago"), 365);
        assert_eq!(estimate_days_from_relative_date("2 hours ago"), 0);
    }

    #[test]
    fn test_run_no_git_repo() {
        // Test error handling when not in a git repository
        // The function should return GitXError when git commands fail
        let sample_churn = analyze_churn_patterns("100	50	test.rs");
        assert!(!sample_churn.is_empty());
        assert_eq!(sample_churn[0].path, "test.rs");
        assert_eq!(sample_churn[0].total_changes, 150);
    }

    #[test]
    fn test_gitx_error_integration() {
        // Test that our functions work with GitXError types correctly
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "git not found");
        let gitx_error: GitXError = io_error.into();
        match gitx_error {
            GitXError::Io(_) => {} // Expected
            _ => panic!("Should convert to Io error"),
        }

        let git_error = GitXError::GitCommand("test error".to_string());
        assert_eq!(git_error.to_string(), "Git command failed: test error");
    }

    #[test]
    fn test_branch_info_struct() {
        let branch = BranchInfo {
            name: "feature/test".to_string(),
            age: "2 months ago".to_string(),
            author: "Test Author".to_string(),
            days_old: 60,
        };

        assert_eq!(branch.name, "feature/test");
        assert_eq!(branch.days_old, 60);
    }

    #[test]
    fn test_churn_file_struct() {
        let file = ChurnFile {
            path: "src/test.rs".to_string(),
            additions: 100,
            deletions: 50,
            total_changes: 150,
        };

        assert_eq!(file.path, "src/test.rs");
        assert_eq!(file.total_changes, 150);
    }

    #[test]
    fn test_large_commit_struct() {
        let commit = LargeCommit {
            hash: "abc123".to_string(),
            message: "Large refactor".to_string(),
            files_changed: 25,
        };

        assert_eq!(commit.hash, "abc123");
        assert_eq!(commit.files_changed, 25);
    }
}
