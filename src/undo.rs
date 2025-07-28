use crate::common::{Format, GitCommand};

pub fn run() {
    match GitCommand::run_status(&["reset", "--soft", "HEAD~1"]) {
        Ok(()) => println!(
            "{}",
            Format::success("Last commit undone (soft reset). Changes kept in working directory.")
        ),
        Err(e) => eprintln!(
            "{}",
            Format::error(&format!("Failed to undo last commit: {e}"))
        ),
    }
}
