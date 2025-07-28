mod cli;

use clap::Parser;
use git_x::cli::{Cli, Commands};
use git_x::{
    clean_branches, color_graph, fixup, graph, health, info, large_files, new_branch,
    prune_branches, rename_branch, since, stash_branch, summary, sync, undo, upstream, what,
};
use std::process;

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Info => handle_result(info::run()),
        Commands::Graph => handle_result(graph::run()),
        Commands::Health => handle_result(health::run()),
        Commands::ColorGraph => handle_result(color_graph::run()),
        Commands::Since { reference } => handle_result(since::run(reference)),
        Commands::Undo => handle_result(undo::run()),
        Commands::What { target } => handle_result(what::run(target)),
        Commands::RenameBranch { new_name } => handle_result(rename_branch::run(&new_name)),
        Commands::PruneBranches { except } => handle_result(prune_branches::run(except)),
        Commands::CleanBranches { dry_run } => handle_result(clean_branches::run(dry_run)),
        Commands::Summary { since } => handle_result(summary::run(since)),
        Commands::Sync { merge } => handle_result(sync::run(merge)),
        Commands::New { branch_name, from } => handle_result(new_branch::run(branch_name, from)),
        Commands::LargeFiles { limit, threshold } => {
            handle_result(large_files::run(limit, threshold))
        }
        Commands::Fixup {
            commit_hash,
            rebase,
        } => handle_result(fixup::run(commit_hash, rebase)),
        Commands::StashBranch { action } => handle_result(stash_branch::run(action)),
        Commands::Upstream { action } => handle_result(upstream::run(action)),
    }
}

fn handle_result(result: Result<String, git_x::GitXError>) {
    match result {
        Ok(output) => {
            if !output.is_empty() {
                println!("{output}");
            }
        }
        Err(err) => {
            eprintln!("{err}");
            process::exit(1);
        }
    }
}
