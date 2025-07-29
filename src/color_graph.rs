use crate::command::Command;
use crate::{GitXError, Result};
use std::process::Command as StdCommand;

pub fn run() -> Result<()> {
    let cmd = ColorGraphCommand;
    cmd.execute(())
}

/// Command implementation for git color-graph
pub struct ColorGraphCommand;

impl Command for ColorGraphCommand {
    type Input = ();
    type Output = ();

    fn execute(&self, _input: ()) -> Result<()> {
        let output = run_color_graph()?;
        print!("{output}");
        Ok(())
    }

    fn name(&self) -> &'static str {
        "color-graph"
    }

    fn description(&self) -> &'static str {
        "Show a colorized git log graph"
    }
}

fn run_color_graph() -> Result<String> {
    let output = StdCommand::new("git")
        .args(get_color_git_log_args())
        .output()
        .map_err(GitXError::Io)?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(GitXError::GitCommand(format!(
            "git log failed: {}",
            stderr.trim()
        )))
    }
}

fn get_color_git_log_args() -> [&'static str; 7] {
    [
        "log",
        "--oneline",
        "--graph",
        "--decorate",
        "--all",
        "--color=always",
        "--pretty=format:%C(auto)%h%d %s %C(dim)(%an, %ar)%C(reset)",
    ]
}
