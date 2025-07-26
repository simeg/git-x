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
}

// pub fn build_cli() -> Command {
//     Command::new("git-x")
//         .about("Supercharge your Git workflow")
//         .version("0.1.0")
//         .subcommand_required(true)
//         .arg_required_else_help(true)
//         .subcommand(
//             Command::new("rename-branch")
//                 .about("Rename the current branch")
//                 .arg(Arg::new("new_name").help("New name for the current branch").required(true)),
//         )
//         .subcommand(
//             Command::new("prune-branches")
//                 .about("Delete local branches that have been merged")
//                 .arg(
//                     Arg::new("except")
//                         .long("except")
//                         .value_name("branches")
//                         .help("Comma-separated list of branches to exclude")
//                         .required(false),
//                 ),
//         )
//         .subcommand(Command::new("info").about("Show a high-level summary of the repo"))
//         .subcommand(Command::new("graph").about("Pretty Git log with branches, remotes, and HEADs"))
//         .subcommand(
//             Command::new("since")
//                 .about("Show commits since a reference (e.g., 7ac9701, origin/main)")
//                 .arg(Arg::new("reference").help("Reference point").required(true)),
//         )
//         .subcommand(Command::new("undo").about("Undo the last commit (without losing changes)"))
//         .subcommand(
//             Command::new("clean-branches")
//                 .about("Delete all fully merged local branches (except protected ones)")
//                 .arg(
//                     Arg::new("dry_run")
//                         .long("dry-run")
//                         .help("Prints the branches it would delete instead of actually deleting them")
//                         .action(clap::ArgAction::SetTrue),
//                 ),
//         )
//         .subcommand(
//             Command::new("what")
//                 .about("Show what’s different between this branch and another (default: main)")
//                 .arg(
//                     Arg::new("target")
//                         .long("target")
//                         .help("Branch to compare to")
//                         .required(false),
//                 ),
//         )
//         .subcommand(
//             Command::new("summary")
//                 .about("Show a short, changelog-style summary of recent commits")
//                 .arg(
//                     Arg::new("since")
//                         .long("since")
//                         .help("Accepts flexible formats like \"yesterday\", \"3 days ago\", \"2025-07-01\", etc. (same as git log --since)")
//                         .required(true),
//                 ),
//         )
// }
