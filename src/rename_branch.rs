use std::process::{Command, exit};

pub fn run(new_name: &str) {
    // Step 1: Get current branch name
    let output = Command::new("git")
        .args(get_current_branch_args())
        .output()
        .expect("Failed to execute git");

    if !output.status.success() {
        eprintln!("Error: Failed to get current branch name.");
        exit(1);
    }

    let current_branch = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if is_branch_already_named(&current_branch, new_name) {
        println!("{}", format_already_named_message(new_name));
        return;
    }

    println!("{}", format_rename_start_message(&current_branch, new_name));

    // Step 2: Rename branch locally
    let status = Command::new("git")
        .args(get_local_rename_args(new_name))
        .status()
        .expect("Failed to rename branch");

    if !status.success() {
        eprintln!("Error: Failed to rename local branch.");
        exit(1);
    }

    // Step 3: Push the new branch to origin
    let status = Command::new("git")
        .args(get_push_new_branch_args(new_name))
        .status()
        .expect("Failed to push new branch");

    if !status.success() {
        eprintln!("Error: Failed to push new branch to origin.");
        exit(1);
    }

    // Step 4: Delete the old branch from origin
    let status = Command::new("git")
        .args(get_delete_old_branch_args(&current_branch))
        .status()
        .expect("Failed to delete old branch");

    if !status.success() {
        eprintln!("{}", format_delete_failed_message(&current_branch));
    } else {
        println!("{}", format_delete_success_message(&current_branch));
    }

    println!("{}", format_rename_success_message());
}

// Helper function to get current branch args
pub fn get_current_branch_args() -> [&'static str; 3] {
    ["rev-parse", "--abbrev-ref", "HEAD"]
}

// Helper function to check if branch is already named
pub fn is_branch_already_named(current_branch: &str, new_name: &str) -> bool {
    current_branch == new_name
}

// Helper function to get local rename args
pub fn get_local_rename_args(new_name: &str) -> Vec<String> {
    vec!["branch".to_string(), "-m".to_string(), new_name.to_string()]
}

// Helper function to get push new branch args
pub fn get_push_new_branch_args(new_name: &str) -> Vec<String> {
    vec![
        "push".to_string(),
        "-u".to_string(),
        "origin".to_string(),
        new_name.to_string(),
    ]
}

// Helper function to get delete old branch args
pub fn get_delete_old_branch_args(old_branch: &str) -> Vec<String> {
    vec![
        "push".to_string(),
        "origin".to_string(),
        "--delete".to_string(),
        old_branch.to_string(),
    ]
}

// Helper function to format already named message
pub fn format_already_named_message(new_name: &str) -> String {
    format!("Current branch is already named '{new_name}'. Nothing to do.")
}

// Helper function to format rename start message
pub fn format_rename_start_message(current_branch: &str, new_name: &str) -> String {
    format!("Renaming branch '{current_branch}' to '{new_name}'")
}

// Helper function to format delete failed message
pub fn format_delete_failed_message(old_branch: &str) -> String {
    format!("Warning: Failed to delete old branch '{old_branch}' from origin.")
}

// Helper function to format delete success message
pub fn format_delete_success_message(old_branch: &str) -> String {
    format!("Deleted old branch '{old_branch}' from origin.")
}

// Helper function to format rename success message
pub fn format_rename_success_message() -> &'static str {
    "Branch renamed successfully."
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_current_branch_args() {
        assert_eq!(get_current_branch_args(), ["rev-parse", "--abbrev-ref", "HEAD"]);
    }

    #[test]
    fn test_is_branch_already_named() {
        assert!(is_branch_already_named("main", "main"));
        assert!(is_branch_already_named("feature", "feature"));
        assert!(!is_branch_already_named("main", "develop"));
        assert!(!is_branch_already_named("feature/test", "hotfix/test"));
    }

    #[test]
    fn test_get_local_rename_args() {
        assert_eq!(
            get_local_rename_args("new-branch"),
            vec!["branch".to_string(), "-m".to_string(), "new-branch".to_string()]
        );
        assert_eq!(
            get_local_rename_args("feature/awesome"),
            vec!["branch".to_string(), "-m".to_string(), "feature/awesome".to_string()]
        );
    }

    #[test]
    fn test_get_push_new_branch_args() {
        assert_eq!(
            get_push_new_branch_args("new-branch"),
            vec![
                "push".to_string(),
                "-u".to_string(),
                "origin".to_string(),
                "new-branch".to_string()
            ]
        );
    }

    #[test]
    fn test_get_delete_old_branch_args() {
        assert_eq!(
            get_delete_old_branch_args("old-branch"),
            vec![
                "push".to_string(),
                "origin".to_string(),
                "--delete".to_string(),
                "old-branch".to_string()
            ]
        );
    }

    #[test]
    fn test_format_already_named_message() {
        assert_eq!(
            format_already_named_message("main"),
            "Current branch is already named 'main'. Nothing to do."
        );
        assert_eq!(
            format_already_named_message("feature/test"),
            "Current branch is already named 'feature/test'. Nothing to do."
        );
    }

    #[test]
    fn test_format_rename_start_message() {
        assert_eq!(
            format_rename_start_message("old-branch", "new-branch"),
            "Renaming branch 'old-branch' to 'new-branch'"
        );
        assert_eq!(
            format_rename_start_message("feature/old", "feature/new"),
            "Renaming branch 'feature/old' to 'feature/new'"
        );
    }

    #[test]
    fn test_format_delete_failed_message() {
        assert_eq!(
            format_delete_failed_message("old-branch"),
            "Warning: Failed to delete old branch 'old-branch' from origin."
        );
    }

    #[test]
    fn test_format_delete_success_message() {
        assert_eq!(
            format_delete_success_message("old-branch"),
            "Deleted old branch 'old-branch' from origin."
        );
    }

    #[test]
    fn test_format_rename_success_message() {
        assert_eq!(format_rename_success_message(), "Branch renamed successfully.");
    }
}
