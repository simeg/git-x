use clap::Parser;

#[derive(Parser)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(clap::Subcommand)]
pub enum Commands {
    #[clap(about = "Rename the current branch")]
    RenameBranch {
        #[clap(help = "New name for the current branch")]
        new_name: String,
    },
    #[clap(about = "Delete merged local branches (except protected ones)")]
    PruneBranches {
        #[clap(
            long = "except",
            value_name = "branches",
            help = "Comma-separated list of branches to exclude"
        )]
        except: Option<String>,
    },
    #[clap(about = "Show a high-level overview of the current repo")]
    Info,
    #[clap(about = "Pretty Git log with branches, remotes, and HEADs")]
    Graph,
    #[clap(about = "Colorized Git log with branches, remotes, and HEADs")]
    ColorGraph,
    #[clap(about = "Check repository health and show potential issues")]
    Health,
    #[clap(about = "Show commits since a reference (e.g., cb676ec, origin/main)")]
    Since {
        #[clap(help = "Reference point")]
        reference: String,
    },
    #[clap(about = "Undo the last commit (without losing changes)")]
    Undo,
    #[clap(about = "Delete all fully merged local branches (except protected ones)")]
    CleanBranches {
        #[clap(long = "dry-run", help = "Prints the branches it would delete instead of actually deleting them", action = clap::ArgAction::SetTrue)]
        dry_run: bool,
    },
    #[clap(about = "Show what’s different between this branch and another (default: main)")]
    What {
        #[clap(long = "target", help = "Branch to compare to")]
        target: Option<String>,
    },
    #[clap(about = "Show a short, changelog-style summary of recent commits")]
    Summary {
        #[clap(
            long = "since",
            help = "Accepts flexible formats like \"yesterday\", \"3 days ago\", \"2025-07-01\", etc. (same as git log --since)"
        )]
        since: String,
    },
    #[clap(about = "Sync current branch with upstream (fetch + rebase)")]
    Sync {
        #[clap(long = "merge", help = "Use merge instead of rebase", action = clap::ArgAction::SetTrue)]
        merge: bool,
    },
    #[clap(about = "Create and switch to a new branch")]
    New {
        #[clap(help = "Name of the new branch")]
        branch_name: String,
        #[clap(
            long = "from",
            help = "Base branch to create from (default: current branch)"
        )]
        from: Option<String>,
    },
    #[clap(about = "Find largest files in repository history")]
    LargeFiles {
        #[clap(long = "limit", default_value = "10", help = "Number of files to show")]
        limit: usize,
        #[clap(
            long = "threshold",
            help = "Minimum file size in MB (default: show all)"
        )]
        threshold: Option<f64>,
    },
}
