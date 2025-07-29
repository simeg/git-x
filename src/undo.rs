use crate::command::Command;
use crate::core::git::GitOperations;
use crate::core::output::Format;

pub fn run() -> crate::Result<()> {
    let cmd = UndoCommand;
    cmd.execute(())
}

/// Command implementation for git undo
pub struct UndoCommand;

impl Command for UndoCommand {
    type Input = ();
    type Output = ();

    fn execute(&self, _input: ()) -> crate::Result<()> {
        GitOperations::run_status(&["reset", "--soft", "HEAD~1"])?;
        println!(
            "{}",
            Format::success("Last commit undone (soft reset). Changes kept in working directory.")
        );
        Ok(())
    }

    fn name(&self) -> &'static str {
        "undo"
    }

    fn description(&self) -> &'static str {
        "Undo the last commit (without losing changes)"
    }

    fn is_destructive(&self) -> bool {
        true
    }
}
