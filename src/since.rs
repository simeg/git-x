use crate::command::Command;
use crate::core::git::GitOperations;

pub fn run(reference: String) -> crate::Result<()> {
    let cmd = SinceCommand;
    cmd.execute(reference)
}

/// Command implementation for git since
pub struct SinceCommand;

impl Command for SinceCommand {
    type Input = String;
    type Output = ();

    fn execute(&self, reference: String) -> crate::Result<()> {
        let output = run_since(&reference)?;
        println!("{output}");
        Ok(())
    }

    fn name(&self) -> &'static str {
        "since"
    }

    fn description(&self) -> &'static str {
        "Show commits since a reference (e.g., cb676ec, origin/main)"
    }
}

fn run_since(reference: &str) -> crate::Result<String> {
    let log_range = format!("{reference}..HEAD");
    let log = GitOperations::run(&["log", &log_range, "--pretty=format:- %h %s"])?;

    if log.trim().is_empty() {
        Ok(format!("{} No new commits since {reference}", "✅"))
    } else {
        Ok(format!("🔍 Commits since {reference}:\n{log}"))
    }
}
