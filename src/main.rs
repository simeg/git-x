mod cli;

use clap::Parser;
use git_x::cli::{Cli, Commands};
use git_x::{
    bisect, clean_branches, contributors, fixup, health, large_files, new_branch, prune_branches,
    rename_branch, stash_branch, summary, switch_recent, sync, technical_debt, upstream, what,
};

// Import Command trait and specific command implementations for demonstration
use git_x::color_graph::ColorGraphCommand;
use git_x::command::Command;
use git_x::graph::GraphCommand;
use git_x::info::InfoCommand;
use git_x::since::SinceCommand;
use git_x::undo::UndoCommand;

// This demonstrates how the Command trait could be used for unified execution
fn execute_command_with_trait<C: Command>(cmd: C, input: C::Input) {
    println!("📋 Executing '{}': {}", cmd.name(), cmd.description());

    if cmd.is_destructive() {
        println!("⚠️  Warning: This is a destructive operation!");
    }

    match cmd.execute(input) {
        Ok(_) => {}
        Err(e) => eprintln!("❌ {e}"),
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::RenameBranch { new_name } => match rename_branch::run(&new_name) {
            Ok(()) => {}
            Err(e) => eprintln!("❌ {e}"),
        },
        Commands::PruneBranches { except, dry_run } => match prune_branches::run(except, dry_run) {
            Ok(()) => {}
            Err(e) => eprintln!("❌ {e}"),
        },
        // Demonstration of trait-based dispatch
        Commands::Info => {
            execute_command_with_trait(InfoCommand, ());
        }
        Commands::Graph => {
            execute_command_with_trait(GraphCommand, ());
        }
        Commands::ColorGraph => {
            execute_command_with_trait(ColorGraphCommand, ());
        }
        Commands::Health => match health::run() {
            Ok(output) => println!("{output}"),
            Err(e) => eprintln!("Error: {e}"),
        },
        Commands::Since { reference } => {
            execute_command_with_trait(SinceCommand, reference);
        }
        Commands::Undo => {
            execute_command_with_trait(UndoCommand, ());
        }
        Commands::CleanBranches { dry_run } => match clean_branches::run(dry_run) {
            Ok(()) => {}
            Err(e) => eprintln!("❌ {e}"),
        },
        Commands::What { target } => match what::run(target) {
            Ok(output) => println!("{output}"),
            Err(e) => eprintln!("Error: {e}"),
        },
        Commands::Summary { since } => match summary::run(since) {
            Ok(()) => {}
            Err(e) => eprintln!("❌ {e}"),
        },
        Commands::Sync { merge } => match sync::run(merge) {
            Ok(()) => {}
            Err(e) => eprintln!("❌ {e}"),
        },
        Commands::New { branch_name, from } => match new_branch::run(branch_name, from) {
            Ok(output) => println!("{output}"),
            Err(e) => eprintln!("Error: {e}"),
        },
        Commands::LargeFiles { limit, threshold } => match large_files::run(limit, threshold) {
            Ok(()) => {}
            Err(e) => eprintln!("❌ {e}"),
        },
        Commands::Fixup {
            commit_hash,
            rebase,
        } => match fixup::run(commit_hash, rebase) {
            Ok(()) => {}
            Err(e) => eprintln!("❌ {e}"),
        },
        Commands::StashBranch { action } => match stash_branch::run(action) {
            Ok(()) => {}
            Err(e) => eprintln!("❌ {e}"),
        },
        Commands::Upstream { action } => match upstream::run(action) {
            Ok(()) => {}
            Err(e) => eprintln!("❌ {e}"),
        },
        Commands::SwitchRecent => match switch_recent::run() {
            Ok(message) => println!("{message}"),
            Err(e) => eprintln!("❌ {e}"),
        },
        Commands::Contributors => match contributors::run() {
            Ok(output) => println!("{output}"),
            Err(e) => eprintln!("❌ {e}"),
        },
        Commands::TechnicalDebt => match technical_debt::run() {
            Ok(output) => println!("{output}"),
            Err(e) => eprintln!("❌ {e}"),
        },
        Commands::Bisect { action } => match bisect::run(action) {
            Ok(output) => println!("{output}"),
            Err(e) => eprintln!("❌ {e}"),
        },
    }
}
