use std::io::{BufRead, BufReader};
use std::process::{Command, exit};

pub fn run(except: Option<String>) {
    let protected_branches = get_all_protected_branches(except.as_deref());

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
        .map(|b| clean_git_branch_name(&b))
        .filter(|b| !is_branch_protected(b, &current_branch, &protected_branches))
        .collect();

    if branches.is_empty() {
        println!("{}", format_no_branches_to_prune_message());
        return;
    }

    // Step 3: Delete branches
    for branch in branches {
        let delete_args = get_git_branch_delete_args(&branch);
        let status = Command::new("git")
            .args(&delete_args)
            .status()
            .expect("Failed to delete branch");

        if status.success() {
            println!("{}", format_branch_deleted_message(&branch));
        } else {
            eprintln!("{}", format_branch_delete_failed_message(&branch));
        }
    }
}

// Helper function to get default protected branches
pub fn get_default_protected_branches() -> Vec<&'static str> {
    vec!["main", "master", "develop"]
}

// Helper function to parse except string into vec
pub fn parse_except_branches(except: &str) -> Vec<String> {
    except
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

// Helper function to get all protected branches
pub fn get_all_protected_branches(except: Option<&str>) -> Vec<String> {
    let mut protected: Vec<String> = get_default_protected_branches()
        .into_iter()
        .map(|s| s.to_string())
        .collect();
    
    if let Some(except_str) = except {
        protected.extend(parse_except_branches(except_str));
    }
    
    protected
}

// Helper function to clean branch name from git output
pub fn clean_git_branch_name(branch: &str) -> String {
    branch.trim().trim_start_matches("* ").to_string()
}

// Helper function to check if branch should be protected
pub fn is_branch_protected(branch: &str, current_branch: &str, protected_branches: &[String]) -> bool {
    branch == current_branch || protected_branches.iter().any(|pb| pb == branch)
}

// Helper function to get git branch delete args
pub fn get_git_branch_delete_args(branch: &str) -> [String; 3] {
    ["branch".to_string(), "-d".to_string(), branch.to_string()]
}

// Helper function to format success message
pub fn format_branch_deleted_message(branch: &str) -> String {
    format!("🧹 Deleted merged branch '{branch}'")
}

// Helper function to format failure message
pub fn format_branch_delete_failed_message(branch: &str) -> String {
    format!("⚠️ Failed to delete branch '{branch}'")
}

// Helper function to format no branches message
pub fn format_no_branches_to_prune_message() -> &'static str {
    "✅ No merged branches to prune."
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_default_protected_branches() {
        assert_eq!(get_default_protected_branches(), vec!["main", "master", "develop"]);
    }

    #[test]
    fn test_parse_except_branches() {
        assert_eq!(
            parse_except_branches("feature,hotfix"),
            vec!["feature".to_string(), "hotfix".to_string()]
        );
        assert_eq!(
            parse_except_branches("  branch1  ,  branch2  "),
            vec!["branch1".to_string(), "branch2".to_string()]
        );
        assert_eq!(parse_except_branches("single"), vec!["single".to_string()]);
        assert_eq!(parse_except_branches(""), Vec::<String>::new());
    }

    #[test]
    fn test_get_all_protected_branches() {
        let default_only = get_all_protected_branches(None);
        assert_eq!(default_only, vec!["main", "master", "develop"]);
        
        let with_except = get_all_protected_branches(Some("feature,hotfix"));
        assert_eq!(with_except, vec!["main", "master", "develop", "feature", "hotfix"]);
    }

    #[test]
    fn test_clean_git_branch_name() {
        assert_eq!(clean_git_branch_name("  feature/test  "), "feature/test");
        assert_eq!(clean_git_branch_name("* main"), "main");
        assert_eq!(clean_git_branch_name("develop"), "develop");
    }

    #[test]
    fn test_is_branch_protected() {
        let protected = vec!["main".to_string(), "develop".to_string()];
        
        assert!(is_branch_protected("main", "current", &protected));
        assert!(is_branch_protected("develop", "current", &protected));
        assert!(is_branch_protected("current", "current", &protected));
        assert!(!is_branch_protected("feature", "current", &protected));
    }

    #[test]
    fn test_get_git_branch_delete_args() {
        assert_eq!(
            get_git_branch_delete_args("feature"),
            ["branch".to_string(), "-d".to_string(), "feature".to_string()]
        );
    }

    #[test]
    fn test_format_branch_deleted_message() {
        assert_eq!(
            format_branch_deleted_message("feature/test"),
            "🧹 Deleted merged branch 'feature/test'"
        );
    }

    #[test]
    fn test_format_branch_delete_failed_message() {
        assert_eq!(
            format_branch_delete_failed_message("feature/test"),
            "⚠️ Failed to delete branch 'feature/test'"
        );
    }

    #[test]
    fn test_format_no_branches_to_prune_message() {
        assert_eq!(format_no_branches_to_prune_message(), "✅ No merged branches to prune.");
    }
}
