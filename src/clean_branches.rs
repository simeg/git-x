use crate::Result;
use crate::command::Command;
use crate::core::git::{BranchOperations, GitOperations};
use crate::core::safety::Safety;

pub fn run(dry_run: bool) -> Result<()> {
    let cmd = CleanBranchesCommand;
    cmd.execute(dry_run)
}

/// Command implementation for git clean-branches
pub struct CleanBranchesCommand;

impl Command for CleanBranchesCommand {
    type Input = bool;
    type Output = ();

    fn execute(&self, dry_run: bool) -> Result<()> {
        run_clean_branches(dry_run)
    }

    fn name(&self) -> &'static str {
        "clean-branches"
    }

    fn description(&self) -> &'static str {
        "Clean up merged branches"
    }

    fn is_destructive(&self) -> bool {
        true
    }
}

fn run_clean_branches(dry_run: bool) -> Result<()> {
    // Safety check: ensure working directory is clean
    if !dry_run {
        Safety::ensure_clean_working_directory()?;
    }

    let merged_branches = GitOperations::merged_branches()?;
    let branches: Vec<String> = merged_branches
        .into_iter()
        .filter(|branch| !is_protected_branch(branch))
        .collect();

    // Safety confirmation for destructive operation
    if !dry_run && !branches.is_empty() {
        let details = format!(
            "This will delete {} merged branches: {}",
            branches.len(),
            branches.join(", ")
        );

        if !Safety::confirm_destructive_operation("Clean merged branches", &details)? {
            println!("Operation cancelled by user.");
            return Ok(());
        }
    }

    let mut deleted = Vec::new();

    for branch in branches {
        if dry_run {
            deleted.push(branch);
        } else {
            match BranchOperations::delete(&branch, false) {
                Ok(()) => {
                    deleted.push(branch);
                }
                Err(_) => {
                    // Branch deletion failed, skip it
                }
            }
        }
    }

    if deleted.is_empty() {
        println!("No merged branches to delete.");
    } else {
        println!("{}", format_deletion_summary(deleted.len(), dry_run));
        for branch in deleted {
            if dry_run {
                let branch1 = &branch;
                println!("{branch1}");
            } else {
                println!("  {branch}");
            }
        }
    }

    Ok(())
}

fn is_protected_branch(branch: &str) -> bool {
    ["main", "master", "develop"].contains(&branch)
}

fn format_deletion_summary(count: usize, dry_run: bool) -> String {
    if dry_run {
        format!("🧪 (dry run) {count} branches would be deleted:")
    } else {
        format!("🧹 Deleted {count} merged branches:")
    }
}
