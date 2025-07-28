use crate::common::{Format, GitCommand, RepoInfo};

pub fn run() {
    match run_info() {
        Ok(output) => println!("{output}"),
        Err(e) => eprintln!("{}", Format::error(&e.to_string())),
    }
}

fn run_info() -> crate::Result<String> {
    let repo_name = RepoInfo::name()?;
    let branch_status = RepoInfo::branch_status()?;
    let last_commit = GitCommand::run(&["log", "-1", "--pretty=format:%s (%cr)"])?;

    let mut lines = Vec::new();
    lines.push(format!("Repo: {}", Format::bold(&repo_name)));
    lines.push(format!("Branch: {}", Format::bold(&branch_status.current)));
    lines.push(format!(
        "Tracking: {}",
        Format::bold(&branch_status.format_tracking())
    ));
    lines.push(format!(
        "Ahead: {} Behind: {}",
        Format::bold(&branch_status.ahead.to_string()),
        Format::bold(&branch_status.behind.to_string())
    ));
    lines.push(format!("Last Commit: \"{}\"", Format::bold(&last_commit)));

    Ok(lines.join("\n"))
}
