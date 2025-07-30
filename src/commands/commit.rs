use crate::core::git::*;
use crate::core::traits::*;
use crate::{GitXError, Result};

/// Commit-related commands grouped together
pub struct CommitCommands;

impl CommitCommands {
    /// Create a fixup commit
    pub fn fixup(commit_hash: &str, auto_rebase: bool) -> Result<String> {
        FixupCommand::new(commit_hash.to_string(), auto_rebase).execute()
    }

    /// Undo the last commit
    pub fn undo() -> Result<String> {
        UndoCommand::new().execute()
    }

    /// Bisect workflow
    pub fn bisect(action: BisectAction) -> Result<String> {
        BisectCommand::new(action).execute()
    }
}

/// Command to create fixup commits
pub struct FixupCommand {
    commit_hash: String,
    auto_rebase: bool,
}

impl FixupCommand {
    pub fn new(commit_hash: String, auto_rebase: bool) -> Self {
        Self {
            commit_hash,
            auto_rebase,
        }
    }

    fn has_staged_changes() -> Result<bool> {
        let staged = GitOperations::staged_files()?;
        Ok(!staged.is_empty())
    }
}

impl Command for FixupCommand {
    fn execute(&self) -> Result<String> {
        // Allow any Git reference (commit hash, branch, tag, etc.)
        // Try to resolve reference to verify it exists
        if GitOperations::run(&["rev-parse", "--verify", &self.commit_hash]).is_err() {
            // Check if we're in a git repo
            if GitOperations::repo_root().is_err() {
                return Err(GitXError::GitCommand(
                    "Commit hash does not exist".to_string(),
                ));
            } else {
                return Err(GitXError::Parse(format!(
                    "Invalid commit hash format: '{}'",
                    self.commit_hash
                )));
            }
        }

        // Check for staged changes
        if !Self::has_staged_changes()? {
            return Err(GitXError::GitCommand(
                "No staged changes found. Please stage your changes first with 'git add'"
                    .to_string(),
            ));
        }

        // Create fixup commit
        CommitOperations::fixup(&self.commit_hash)?;

        let mut result = format!("✅ Fixup commit created for {}", self.commit_hash);

        if self.auto_rebase {
            result.push_str("\n🔄 Starting interactive rebase with autosquash");
            // Perform interactive rebase with autosquash
            match GitOperations::run_status(&[
                "rebase",
                "-i",
                "--autosquash",
                &format!("{}^", self.commit_hash),
            ]) {
                Ok(_) => {
                    result.push_str("\n✅ Interactive rebase completed successfully");
                }
                Err(_) => {
                    result.push_str(&format!(
                        "\n💡 To squash the fixup commit, run: git rebase -i --autosquash {}^",
                        self.commit_hash
                    ));
                }
            }
        } else {
            result.push_str(&format!(
                "\n💡 To squash the fixup commit, run: git rebase -i --autosquash {}^",
                self.commit_hash
            ));
        }

        Ok(result)
    }

    fn name(&self) -> &'static str {
        "fixup"
    }

    fn description(&self) -> &'static str {
        "Create fixup commits for easier interactive rebasing"
    }
}

impl GitCommand for FixupCommand {}

/// Command to undo the last commit
pub struct UndoCommand;

impl Default for UndoCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl UndoCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for UndoCommand {
    fn execute(&self) -> Result<String> {
        GitOperations::run_status(&["reset", "--soft", "HEAD~1"])?;
        Ok("✅ Last commit undone (soft reset). Changes kept in working directory.".to_string())
    }

    fn name(&self) -> &'static str {
        "undo"
    }

    fn description(&self) -> &'static str {
        "Undo the last commit (without losing changes)"
    }
}

impl GitCommand for UndoCommand {}
impl Destructive for UndoCommand {
    fn destruction_description(&self) -> String {
        "This will undo your last commit (but keep the changes staged)".to_string()
    }
}

/// Bisect workflow actions
#[derive(Debug, Clone)]
pub enum BisectAction {
    Start { bad: String, good: String },
    Good,
    Bad,
    Skip,
    Reset,
    Status,
}

/// Command for Git bisect workflow
pub struct BisectCommand {
    action: BisectAction,
}

impl BisectCommand {
    pub fn new(action: BisectAction) -> Self {
        Self { action }
    }

    fn is_bisecting() -> Result<bool> {
        // Check if .git/BISECT_HEAD exists
        match GitOperations::repo_root() {
            Ok(root) => {
                let bisect_head = std::path::Path::new(&root).join(".git").join("BISECT_HEAD");
                Ok(bisect_head.exists())
            }
            Err(_) => Ok(false),
        }
    }

    fn execute_bisect_action(&self) -> Result<String> {
        match &self.action {
            BisectAction::Start { bad, good } => {
                // Allow any Git reference (commit hash, branch, tag, etc.)
                // Don't validate as strict hex - Git will handle this

                // Try to resolve references to verify they exist
                if GitOperations::run(&["rev-parse", "--verify", bad]).is_err() {
                    return Err(GitXError::GitCommand(format!(
                        "Reference '{bad}' does not exist"
                    )));
                }
                if GitOperations::run(&["rev-parse", "--verify", good]).is_err() {
                    return Err(GitXError::GitCommand(format!(
                        "Reference '{good}' does not exist"
                    )));
                }

                // Start bisect and capture git output for proper feedback
                let output = GitOperations::run(&["bisect", "start", bad, good])?;

                let mut result =
                    format!("🔍 Starting bisect between {bad} (bad) and {good} (good)");
                if !output.trim().is_empty() {
                    result = format!("{}\n{}", output.trim(), result);
                }
                result.push_str("\n✅ Checked out commit");

                Ok(result)
            }
            BisectAction::Good => {
                if !Self::is_bisecting()? {
                    return Err(GitXError::GitCommand(
                        "Not currently in bisect mode".to_string(),
                    ));
                }
                GitOperations::run_status(&["bisect", "good"])?;
                Ok("✅ Marked current commit as good".to_string())
            }
            BisectAction::Bad => {
                if !Self::is_bisecting()? {
                    return Err(GitXError::GitCommand(
                        "Not currently in bisect mode".to_string(),
                    ));
                }
                GitOperations::run_status(&["bisect", "bad"])?;
                Ok("❌ Marked current commit as bad".to_string())
            }
            BisectAction::Skip => {
                if !Self::is_bisecting()? {
                    return Err(GitXError::GitCommand(
                        "Not currently in bisect mode".to_string(),
                    ));
                }
                GitOperations::run_status(&["bisect", "skip"])?;
                Ok("⏭️ Skipped current commit".to_string())
            }
            BisectAction::Reset => {
                if !Self::is_bisecting()? {
                    return Ok("Not currently in bisect mode".to_string());
                }
                GitOperations::run_status(&["bisect", "reset"])?;
                Ok("🔄 Reset bisect and returned to original branch".to_string())
            }
            BisectAction::Status => {
                if !Self::is_bisecting()? {
                    return Ok("Not currently in bisect mode".to_string());
                }

                let log = GitOperations::run(&["bisect", "log"])
                    .unwrap_or_else(|_| "No bisect log available".to_string());
                Ok(format!("🔍 Bisect status:\n{log}"))
            }
        }
    }
}

impl Command for BisectCommand {
    fn execute(&self) -> Result<String> {
        self.execute_bisect_action()
    }

    fn name(&self) -> &'static str {
        "bisect"
    }

    fn description(&self) -> &'static str {
        "Simplified Git bisect workflow for finding bugs"
    }
}

impl GitCommand for BisectCommand {}

impl Destructive for BisectCommand {
    fn destruction_description(&self) -> String {
        match &self.action {
            BisectAction::Start { .. } => {
                "This will start a bisect session and change your working directory".to_string()
            }
            BisectAction::Reset => {
                "This will reset the bisect session and return to your original branch".to_string()
            }
            _ => "This will change your working directory to a different commit".to_string(),
        }
    }
}
