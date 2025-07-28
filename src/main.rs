mod cli;

use clap::Parser;
use git_x::cli::{Cli, Commands};
use git_x::{
    clean_branches, color_graph, contributors, fixup, graph, health, info, large_files, new_branch,
    prune_branches, rename_branch, since, stash_branch, summary, switch_recent, sync, undo,
    upstream, what,
};

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::RenameBranch { new_name } => rename_branch::run(&new_name),
        Commands::PruneBranches { except } => prune_branches::run(except),
        Commands::Info => info::run(),
        Commands::Graph => graph::run(),
        Commands::ColorGraph => color_graph::run(),
        Commands::Health => match health::run() {
            Ok(output) => println!("{output}"),
            Err(e) => eprintln!("Error: {e}"),
        },
        Commands::Since { reference } => since::run(reference),
        Commands::Undo => undo::run(),
        Commands::CleanBranches { dry_run } => clean_branches::run(dry_run),
        Commands::What { target } => match what::run(target) {
            Ok(output) => println!("{output}"),
            Err(e) => eprintln!("Error: {e}"),
        },
        Commands::Summary { since } => summary::run(since),
        Commands::Sync { merge } => sync::run(merge),
        Commands::New { branch_name, from } => match new_branch::run(branch_name, from) {
            Ok(output) => println!("{output}"),
            Err(e) => eprintln!("Error: {e}"),
        },
        Commands::LargeFiles { limit, threshold } => large_files::run(limit, threshold),
        Commands::Fixup {
            commit_hash,
            rebase,
        } => fixup::run(commit_hash, rebase),
        Commands::StashBranch { action } => stash_branch::run(action),
        Commands::Upstream { action } => upstream::run(action),
        Commands::SwitchRecent => match switch_recent::run() {
            Ok(message) => println!("{message}"),
            Err(e) => eprintln!("❌ {e}"),
        },
        Commands::Contributors => match contributors::run() {
            Ok(output) => println!("{output}"),
            Err(e) => eprintln!("❌ {e}"),
        },
    }
}
