mod cli;

use clap::Parser;
use git_x::cli::{Cli, Commands};
use git_x::{
    clean_branches, color_graph, graph, health, info, prune_branches, rename_branch, since,
    summary, undo, what,
};

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::RenameBranch { new_name } => rename_branch::run(&new_name),
        Commands::PruneBranches { except } => prune_branches::run(except),
        Commands::Info => info::run(),
        Commands::Graph => graph::run(),
        Commands::ColorGraph => color_graph::run(),
        Commands::Health => health::run(),
        Commands::Since { reference } => since::run(reference),
        Commands::Undo => undo::run(),
        Commands::CleanBranches { dry_run } => clean_branches::run(dry_run),
        Commands::What { target } => what::run(target),
        Commands::Summary { since } => summary::run(since),
    }
}
