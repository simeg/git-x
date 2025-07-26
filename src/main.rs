use clap::{Parser, Subcommand};

mod clean_branches;
mod graph;
mod info;
mod prune_branches;
mod rename_branch;
mod since;
mod summary;
mod undo;
mod what;

#[derive(Parser)]
#[command(name = "git-x")]
#[command(version = "0.1.0")]
#[command(about = "Supercharge your Git workflow", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Rename the current branch")]
    RenameBranch {
        #[arg(help = "New name for the current branch")]
        new_name: String,
    },

    #[command(about = "Delete local branches that have been merged")]
    PruneBranches {
        #[arg(long, help = "Comma-separated list of branches to exclude")]
        except: Option<String>,
    },

    #[command(about = "Show a high-level summary of the repo")]
    Info,

    #[command(about = "Pretty Git log with branches, remotes, and HEADs")]
    Graph,

    #[command(about = "Show commits since a reference (e.g., 7ac9701, origin/main)")]
    Since { reference: String },

    #[command(about = "Undo the last commit (without losing changes)")]
    Undo,

    #[command(about = "Delete all fully merged local branches (except protected ones)")]
    CleanBranches {
        #[arg(
            long,
            help = "Prints the branches it would delete instead of actually deleting them"
        )]
        dry_run: bool,
    },

    #[command(about = "Show what’s different between this branch and another (default: main)")]
    What {
        #[arg(long, help = "Branch to compare to")]
        target: Option<String>,
    },

    #[command(about = "Show a short, changelog-style summary of recent commits")]
    Summary {
        #[arg(
            long,
            help = "Accepts flexible formats like \"yesterday\", \"3 days ago\", \"2025-07-01\", etc. (same as git log --since)"
        )]
        since: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::RenameBranch { new_name } => rename_branch::run(&new_name),
        Commands::PruneBranches { except } => prune_branches::run(except),
        Commands::Info => info::run(),
        Commands::Graph => graph::run(),
        Commands::Since { reference } => since::run(reference),
        Commands::Undo => undo::run(),
        Commands::CleanBranches { dry_run } => clean_branches::run(dry_run),
        Commands::What { target } => what::run(target),
        Commands::Summary { since } => summary::run(since),
    }
}
