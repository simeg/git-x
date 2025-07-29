use crate::command::Command;
use crate::core::git::GitOperations;

pub fn run() -> crate::Result<()> {
    let cmd = GraphCommand;
    cmd.execute(())
}

/// Command implementation for git graph
pub struct GraphCommand;

impl Command for GraphCommand {
    type Input = ();
    type Output = ();

    fn execute(&self, _input: ()) -> crate::Result<()> {
        let output = GitOperations::run(&["log", "--oneline", "--graph", "--decorate", "--all"])?;
        println!("{output}");
        Ok(())
    }

    fn name(&self) -> &'static str {
        "graph"
    }

    fn description(&self) -> &'static str {
        "Show a simple git log graph"
    }
}
