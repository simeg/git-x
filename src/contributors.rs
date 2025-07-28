use crate::{GitXError, Result};
use console::style;
use std::collections::HashMap;
use std::process::Command;

pub fn run() -> Result<String> {
    let output = Command::new("git")
        .args(["log", "--all", "--format=%ae|%an|%ad", "--date=short"])
        .output()?;

    if !output.status.success() {
        return Err(GitXError::GitCommand(
            "Failed to retrieve commit history".to_string(),
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.trim().is_empty() {
        return Ok("📊 No contributors found in this repository".to_string());
    }

    let contributors = parse_contributors(&stdout)?;
    Ok(format_contributors_output(&contributors))
}

#[derive(Clone)]
struct ContributorStats {
    name: String,
    email: String,
    commit_count: usize,
    first_commit: String,
    last_commit: String,
}

fn parse_contributors(output: &str) -> Result<Vec<ContributorStats>> {
    let mut contributors: HashMap<String, ContributorStats> = HashMap::new();

    for line in output.lines() {
        let parts: Vec<&str> = line.splitn(3, '|').collect();
        if parts.len() != 3 {
            continue;
        }

        let email = parts[0].trim().to_string();
        let name = parts[1].trim().to_string();
        let date = parts[2].trim().to_string();

        contributors
            .entry(email.clone())
            .and_modify(|stats| {
                stats.commit_count += 1;
                if date < stats.first_commit {
                    stats.first_commit = date.clone();
                }
                if date > stats.last_commit {
                    stats.last_commit = date.clone();
                }
            })
            .or_insert(ContributorStats {
                name: name.clone(),
                email: email.clone(),
                commit_count: 1,
                first_commit: date.clone(),
                last_commit: date,
            });
    }

    let mut sorted_contributors: Vec<ContributorStats> = contributors.into_values().collect();
    sorted_contributors.sort_by(|a, b| b.commit_count.cmp(&a.commit_count));

    Ok(sorted_contributors)
}

fn format_contributors_output(contributors: &[ContributorStats]) -> String {
    let mut output = Vec::new();
    let total_commits: usize = contributors.iter().map(|c| c.commit_count).sum();

    output.push(format!(
        "{} Repository Contributors ({} total commits):\n",
        style("📊").bold(),
        style(total_commits).bold()
    ));

    for (index, contributor) in contributors.iter().enumerate() {
        let rank_icon = match index {
            0 => "🥇",
            1 => "🥈",
            2 => "🥉",
            _ => "👤",
        };

        let percentage = (contributor.commit_count as f64 / total_commits as f64) * 100.0;

        output.push(format!(
            "{} {} {} commits ({:.1}%)",
            rank_icon,
            style(&contributor.name).bold(),
            style(contributor.commit_count).cyan(),
            percentage
        ));

        output.push(format!(
            "   📧 {} | 📅 {} to {}",
            style(&contributor.email).dim(),
            style(&contributor.first_commit).dim(),
            style(&contributor.last_commit).dim()
        ));

        if index < contributors.len() - 1 {
            output.push(String::new());
        }
    }

    output.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GitXError;

    #[test]
    fn test_parse_contributors_success() {
        let sample_output = r#"alice@example.com|Alice Smith|2025-01-15
bob@example.com|Bob Jones|2025-01-10
alice@example.com|Alice Smith|2025-01-20
charlie@example.com|Charlie Brown|2025-01-12"#;

        let result = parse_contributors(sample_output);
        assert!(result.is_ok());

        let contributors = result.unwrap();
        assert_eq!(contributors.len(), 3);

        // Alice should be first with 2 commits
        assert_eq!(contributors[0].name, "Alice Smith");
        assert_eq!(contributors[0].commit_count, 2);
        assert_eq!(contributors[0].first_commit, "2025-01-15");
        assert_eq!(contributors[0].last_commit, "2025-01-20");

        // Bob and Charlie should have 1 commit each
        assert!(
            contributors
                .iter()
                .any(|c| c.name == "Bob Jones" && c.commit_count == 1)
        );
        assert!(
            contributors
                .iter()
                .any(|c| c.name == "Charlie Brown" && c.commit_count == 1)
        );
    }

    #[test]
    fn test_parse_contributors_empty_input() {
        let result = parse_contributors("");
        assert!(result.is_ok());
        let contributors = result.unwrap();
        assert!(contributors.is_empty());
    }

    #[test]
    fn test_parse_contributors_malformed_input() {
        let malformed_output = "invalid|line\nincomplete";
        let result = parse_contributors(malformed_output);
        assert!(result.is_ok());
        let contributors = result.unwrap();
        assert!(contributors.is_empty());
    }

    #[test]
    fn test_format_contributors_output() {
        let contributors = vec![
            ContributorStats {
                name: "Alice Smith".to_string(),
                email: "alice@example.com".to_string(),
                commit_count: 10,
                first_commit: "2025-01-01".to_string(),
                last_commit: "2025-01-15".to_string(),
            },
            ContributorStats {
                name: "Bob Jones".to_string(),
                email: "bob@example.com".to_string(),
                commit_count: 5,
                first_commit: "2025-01-05".to_string(),
                last_commit: "2025-01-10".to_string(),
            },
        ];

        let output = format_contributors_output(&contributors);

        // Check that output contains expected elements (accounting for styling)
        assert!(output.contains("Repository Contributors"));
        assert!(output.contains("15") && output.contains("total commits")); // Account for styling
        assert!(output.contains("🥇"));
        assert!(output.contains("🥈"));
        assert!(output.contains("Alice Smith"));
        assert!(output.contains("Bob Jones"));
        assert!(output.contains("66.7%"));
        assert!(output.contains("33.3%"));
        assert!(output.contains("10") && output.contains("commits")); // Alice's commit count
        assert!(output.contains("5") && output.contains("commits")); // Bob's commit count
        assert!(output.contains("alice@example.com"));
        assert!(output.contains("bob@example.com"));
    }

    #[test]
    fn test_run_no_git_repo() {
        // This test runs in the context where git might fail
        // We test by calling parse_contributors with valid input
        let result = parse_contributors("test@example.com|Test User|2025-01-01");
        match result {
            Ok(contributors) => {
                assert_eq!(contributors.len(), 1);
                assert_eq!(contributors[0].name, "Test User");
            }
            Err(_) => panic!("Should parse valid input successfully"),
        }
    }

    #[test]
    fn test_contributor_stats_fields() {
        let stats = ContributorStats {
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            commit_count: 5,
            first_commit: "2025-01-01".to_string(),
            last_commit: "2025-01-15".to_string(),
        };

        assert_eq!(stats.name, "Test User");
        assert_eq!(stats.email, "test@example.com");
        assert_eq!(stats.commit_count, 5);
        assert_eq!(stats.first_commit, "2025-01-01");
        assert_eq!(stats.last_commit, "2025-01-15");
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
    fn test_sorting_by_commit_count() {
        let sample_output = r#"alice@example.com|Alice Smith|2025-01-15
bob@example.com|Bob Jones|2025-01-10
alice@example.com|Alice Smith|2025-01-20
alice@example.com|Alice Smith|2025-01-25
bob@example.com|Bob Jones|2025-01-12
charlie@example.com|Charlie Brown|2025-01-12"#;

        let result = parse_contributors(sample_output);
        assert!(result.is_ok());

        let contributors = result.unwrap();
        assert_eq!(contributors.len(), 3);

        // Should be sorted by commit count (descending)
        assert_eq!(contributors[0].name, "Alice Smith");
        assert_eq!(contributors[0].commit_count, 3);
        assert_eq!(contributors[1].name, "Bob Jones");
        assert_eq!(contributors[1].commit_count, 2);
        assert_eq!(contributors[2].name, "Charlie Brown");
        assert_eq!(contributors[2].commit_count, 1);
    }

    #[test]
    fn test_date_range_tracking() {
        let sample_output = r#"alice@example.com|Alice Smith|2025-01-20
alice@example.com|Alice Smith|2025-01-10
alice@example.com|Alice Smith|2025-01-15"#;

        let result = parse_contributors(sample_output);
        assert!(result.is_ok());

        let contributors = result.unwrap();
        assert_eq!(contributors.len(), 1);

        let alice = &contributors[0];
        assert_eq!(alice.first_commit, "2025-01-10"); // Earliest date
        assert_eq!(alice.last_commit, "2025-01-20"); // Latest date
        assert_eq!(alice.commit_count, 3);
    }
}
