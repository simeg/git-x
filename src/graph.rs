use crate::common::{Format, GitCommand};

pub fn run() {
    match GitCommand::run(&["log", "--oneline", "--graph", "--decorate", "--all"]) {
        Ok(output) => println!("{output}"),
        Err(e) => eprintln!("{}", Format::error(&format!("git log failed: {e}"))),
    }
}
