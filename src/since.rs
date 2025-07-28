use crate::common::{Format, GitCommand};

pub fn run(reference: String) {
    match run_since(&reference) {
        Ok(output) => println!("{output}"),
        Err(e) => eprintln!(
            "{}",
            Format::error(&format!(
                "Failed to retrieve commits since '{reference}': {e}"
            ))
        ),
    }
}

fn run_since(reference: &str) -> crate::Result<String> {
    let log_range = format!("{reference}..HEAD");
    let log = GitCommand::run(&["log", &log_range, "--pretty=format:- %h %s"])?;

    if log.trim().is_empty() {
        Ok(format!("{} No new commits since {reference}", "✅"))
    } else {
        Ok(format!("🔍 Commits since {reference}:\n{log}"))
    }
}
