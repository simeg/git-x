use crate::GitXError;
use crate::command::Command;
use crate::core::git::{BranchOperations, GitOperations, RemoteOperations};

pub fn run(new_name: &str) -> Result<(), GitXError> {
    let cmd = RenameBranchCommand;
    cmd.execute(new_name.to_string())
}

/// Command implementation for git rename-branch
pub struct RenameBranchCommand;

impl Command for RenameBranchCommand {
    type Input = String;
    type Output = ();

    fn execute(&self, new_name: String) -> Result<(), GitXError> {
        run_rename_branch(&new_name)
    }

    fn name(&self) -> &'static str {
        "rename-branch"
    }

    fn description(&self) -> &'static str {
        "Rename the current branch both locally and remotely"
    }

    fn is_destructive(&self) -> bool {
        true
    }
}

fn run_rename_branch(new_name: &str) -> Result<(), GitXError> {
    // Step 1: Get current branch name
    let current_branch = GitOperations::current_branch()
        .map_err(|e| GitXError::GitCommand(format!("Failed to get current branch: {e}")))?;

    if is_branch_already_named(&current_branch, new_name) {
        println!("{new_name}");
        return Ok(());
    }

    let current_branch1 = &current_branch;
    println!("{current_branch1}");

    // Step 2: Rename branch locally
    BranchOperations::rename(new_name)
        .map_err(|e| GitXError::GitCommand(format!("Failed to rename local branch: {e}")))?;

    // Step 3: Push the new branch to origin
    RemoteOperations::push(Some("origin"), Some(new_name))
        .map_err(|e| GitXError::GitCommand(format!("Failed to push new branch: {e}")))?;

    // Step 4: Delete the old branch from origin
    match GitOperations::run_status(&["push", "origin", "--delete", &current_branch]) {
        Ok(()) => {
            let old_branch = &current_branch;
            println!("Deleted old branch '{old_branch}' from origin.");
        }
        Err(_) => {
            let old_branch = &current_branch;
            eprintln!("Warning: Failed to delete old branch '{old_branch}' from origin.");
        }
    }

    println!("Branch renamed successfully.");
    Ok(())
}

fn is_branch_already_named(current_branch: &str, new_name: &str) -> bool {
    current_branch == new_name
}
