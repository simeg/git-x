use std::process::Command;

pub fn run(dry_run: bool) {
    let output = Command::new("git")
        .args(get_git_branch_args())
        .output()
        .expect("Failed to list merged branches");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let branches: Vec<String> = stdout
        .lines()
        .map(|line| clean_branch_name(line))
        .filter(|branch| !is_protected_branch(branch))
        .collect();

    let mut deleted = Vec::new();

    for branch in branches {
        if dry_run {
            println!("{}", format_dry_run_message(&branch));
            deleted.push(branch);
        } else {
            let delete_args = get_git_delete_args(&branch);
            let status = Command::new("git")
                .args(&delete_args)
                .status()
                .expect("Failed to delete branch");

            if status.success() {
                deleted.push(branch);
            }
        }
    }

    if deleted.is_empty() {
        println!("{}", format_no_branches_message());
    } else {
        println!("{}", format_deletion_summary(deleted.len(), dry_run));
        for branch in deleted {
            println!("  {branch}");
        }
    }
}

// Helper function to get git branch args
pub fn get_git_branch_args() -> [&'static str; 2] {
    ["branch", "--merged"]
}

// Helper function to get protected branches
pub fn get_protected_branches() -> Vec<&'static str> {
    vec!["main", "master", "develop"]
}

// Helper function to clean branch name
pub fn clean_branch_name(line: &str) -> String {
    line.trim().trim_start_matches('*').trim().to_string()
}

// Helper function to is_protected_branch
pub fn is_protected_branch(branch: &str) -> bool {
    get_protected_branches().contains(&branch)
}

// Helper function to get git delete args
pub fn get_git_delete_args(branch: &str) -> Vec<String> {
    vec!["branch".to_string(), "-d".to_string(), branch.to_string()]
}

// Helper function to format dry run message
pub fn format_dry_run_message(branch: &str) -> String {
    format!("(dry run) Would delete: {branch}")
}

// Helper function to format no branches message
pub fn format_no_branches_message() -> &'static str {
    "No merged branches to delete."
}

// Helper function to format deletion summary
pub fn format_deletion_summary(count: usize, dry_run: bool) -> String {
    if dry_run {
        format!("🧪 (dry run) {} branches would be deleted:", count)
    } else {
        format!("🧹 Deleted {} merged branches:", count)
    }
}

