use crate::command::Command;
use crate::core::git::GitOperations;
use crate::core::output::Format;

pub fn run() -> crate::Result<()> {
    let cmd = InfoCommand;
    cmd.execute(())
}

/// Command implementation for git info
pub struct InfoCommand;

impl Command for InfoCommand {
    type Input = ();
    type Output = ();

    fn execute(&self, _input: ()) -> crate::Result<()> {
        let output = run_info()?;
        println!("{output}");
        Ok(())
    }

    fn name(&self) -> &'static str {
        "info"
    }

    fn description(&self) -> &'static str {
        "Show a high-level overview of the current repo"
    }
}

fn run_info() -> crate::Result<String> {
    let repo_name = GitOperations::repo_root()?;
    let (current_branch, upstream, ahead, behind) = GitOperations::branch_info_optimized()?;
    let last_commit = GitOperations::run(&["log", "-1", "--pretty=format:%s (%cr)"])?;

    // Format upstream tracking info
    let tracking = upstream
        .as_ref()
        .map(|u| u.to_string())
        .unwrap_or_else(|| "(no upstream)".to_string());

    let mut lines = Vec::new();
    lines.push(format!("Repo: {}", Format::bold(&repo_name)));
    lines.push(format!("Branch: {}", Format::bold(&current_branch)));
    lines.push(format!("Tracking: {}", Format::bold(&tracking)));
    lines.push(format!(
        "Ahead: {} Behind: {}",
        Format::bold(&ahead.to_string()),
        Format::bold(&behind.to_string())
    ));
    lines.push(format!("Last Commit: \"{}\"", Format::bold(&last_commit)));

    Ok(lines.join("\n"))
}
