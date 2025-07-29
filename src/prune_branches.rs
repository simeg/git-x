use crate::GitXError;
use crate::command::Command;
use crate::core::git::{BranchOperations, GitOperations};
use crate::core::output::BufferedOutput;
use crate::core::safety::Safety;

pub fn run(except: Option<String>, dry_run: bool) -> Result<(), GitXError> {
    let cmd = PruneBranchesCommand;
    cmd.execute((except, dry_run))
}

/// Command implementation for git prune-branches
pub struct PruneBranchesCommand;

impl Command for PruneBranchesCommand {
    type Input = (Option<String>, bool);
    type Output = ();

    fn execute(&self, (except, dry_run): (Option<String>, bool)) -> Result<(), GitXError> {
        run_prune_branches(except, dry_run)
    }

    fn name(&self) -> &'static str {
        "prune-branches"
    }

    fn description(&self) -> &'static str {
        "Prune merged branches with optional exceptions"
    }

    fn is_destructive(&self) -> bool {
        true
    }
}

fn run_prune_branches(except: Option<String>, dry_run: bool) -> Result<(), GitXError> {
    let protected_branches = get_all_protected_branches(except.as_deref());

    // Step 1: Get current branch
    let current_branch = GitOperations::current_branch()
        .map_err(|e| GitXError::GitCommand(format!("Failed to get current branch: {e}")))?;

    // Step 2: Get merged branches
    let merged_branches = GitOperations::merged_branches()
        .map_err(|e| GitXError::GitCommand(format!("Failed to get merged branches: {e}")))?;

    let branches: Vec<String> = merged_branches
        .into_iter()
        .filter(|b| !is_branch_protected(b, &current_branch, &protected_branches))
        .collect();

    if branches.is_empty() {
        println!("✅ No merged branches to prune.");
        return Ok(());
    }

    // Step 3: Safety confirmation for destructive operation (skip if dry run)
    if !dry_run {
        let details = format!(
            "This will delete {} merged branches: {}",
            branches.len(),
            branches.join(", ")
        );

        match Safety::confirm_destructive_operation("Prune merged branches", &details) {
            Ok(confirmed) => {
                if !confirmed {
                    println!("Operation cancelled by user.");
                    return Ok(());
                }
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    // Step 4: Delete branches or show what would be deleted
    let mut output = BufferedOutput::new();
    let mut error_output = BufferedOutput::new();

    if dry_run {
        // Dry run: show what would be deleted
        let count = branches.len();
        output.add_line(format!("🧪 (dry run) {count} branches would be deleted:"));
        for branch in branches {
            output.add_line(branch);
        }
    } else {
        // Actually delete branches
        for branch in branches {
            match BranchOperations::delete(&branch, false) {
                Ok(()) => {
                    output.add_line(branch);
                }
                Err(_) => {
                    error_output.add_line(branch);
                }
            }
        }
    }

    // Flush all outputs at once for better performance
    output.flush();
    error_output.flush_err();

    Ok(())
}

const DEFAULT_PROTECTED_BRANCHES: &[&str] = &["main", "master", "develop"];

pub fn get_all_protected_branches(except: Option<&str>) -> Vec<String> {
    let mut protected: Vec<String> = DEFAULT_PROTECTED_BRANCHES
        .iter()
        .map(|&s| s.to_string())
        .collect();

    if let Some(except_str) = except {
        let vec: Vec<_> = except_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        protected.extend(vec);
    }

    protected
}

pub fn is_branch_protected(
    branch: &str,
    current_branch: &str,
    protected_branches: &[String],
) -> bool {
    branch == current_branch || protected_branches.iter().any(|pb| pb == branch)
}
