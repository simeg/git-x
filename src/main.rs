mod cli;

use clap::Parser;
use git_x::cli::{Cli, Commands};

use git_x::commands::analysis::{
    AsyncSummaryCommand, ColorGraphCommand, GraphCommand, ParallelContributorsCommand,
    ParallelLargeFilesCommand, ParallelTechnicalDebtCommand, SinceCommand as NewSinceCommand,
    WhatCommand,
};
use git_x::commands::branch::AsyncCleanBranchesCommand;
use git_x::commands::commit::{BisectCommand, FixupCommand, UndoCommand as NewUndoCommand};
use git_x::commands::repository::{
    AsyncHealthCommand, AsyncInfoCommand, AsyncUpstreamCommand, NewBranchCommand,
};
use git_x::core::traits::Command as NewCommand;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::RenameBranch { new_name } => {
            use git_x::commands::branch::RenameBranchCommand;
            let cmd = RenameBranchCommand::new(new_name);
            match NewCommand::execute(&cmd) {
                Ok(output) => println!("{output}"),
                Err(e) => eprintln!("❌ {e}"),
            }
        }

        Commands::PruneBranches { except: _, dry_run } => {
            use git_x::commands::branch::PruneBranchesCommand;
            let cmd = PruneBranchesCommand::new(dry_run);
            match NewCommand::execute(&cmd) {
                Ok(output) => println!("{output}"),
                Err(e) => eprintln!("❌ {e}"),
            }
        }

        Commands::Info => {
            let cmd = AsyncInfoCommand::new();
            match cmd.execute_parallel().await {
                Ok(output) => println!("{output}"),
                Err(e) => eprintln!("❌ {e}"),
            }
        }

        Commands::Graph => {
            let cmd = GraphCommand::new();
            match NewCommand::execute(&cmd) {
                Ok(output) => println!("{output}"),
                Err(e) => eprintln!("❌ {e}"),
            }
        }
        Commands::ColorGraph => {
            let cmd = ColorGraphCommand::new();
            match NewCommand::execute(&cmd) {
                Ok(output) => println!("{output}"),
                Err(e) => eprintln!("❌ {e}"),
            }
        }

        Commands::Health => {
            let cmd = AsyncHealthCommand::new();
            match cmd.execute_parallel().await {
                Ok(output) => println!("{output}"),
                Err(e) => eprintln!("❌ {e}"),
            }
        }

        Commands::Since { reference } => {
            let cmd = NewSinceCommand::new(reference);
            match NewCommand::execute(&cmd) {
                Ok(output) => println!("{output}"),
                Err(e) => eprintln!("❌ {e}"),
            }
        }

        Commands::Undo => {
            let cmd = NewUndoCommand::new();
            match NewCommand::execute(&cmd) {
                Ok(output) => println!("{output}"),
                Err(e) => eprintln!("❌ {e}"),
            }
        }

        Commands::CleanBranches { dry_run } => {
            let cmd = AsyncCleanBranchesCommand::new(dry_run);
            match cmd.execute_parallel().await {
                Ok(output) => println!("{output}"),
                Err(e) => eprintln!("❌ {e}"),
            }
        }

        Commands::What { target } => {
            let cmd = WhatCommand::new(target);
            match NewCommand::execute(&cmd) {
                Ok(output) => println!("{output}"),
                Err(e) => eprintln!("❌ {e}"),
            }
        }

        Commands::Summary { since } => {
            let cmd = AsyncSummaryCommand::new(since);
            match cmd.execute_parallel().await {
                Ok(output) => println!("{output}"),
                Err(e) => eprintln!("❌ {e}"),
            }
        }

        Commands::Sync { merge } => {
            use git_x::commands::repository::{SyncCommand, SyncStrategy};
            let strategy = if merge {
                SyncStrategy::Merge
            } else {
                SyncStrategy::Rebase
            };
            let cmd = SyncCommand::new(strategy);
            match NewCommand::execute(&cmd) {
                Ok(output) => println!("{output}"),
                Err(e) => eprintln!("❌ {e}"),
            }
        }

        Commands::New { branch_name, from } => {
            let cmd = NewBranchCommand::new(branch_name, from);
            match NewCommand::execute(&cmd) {
                Ok(output) => println!("{output}"),
                Err(e) => eprintln!("❌ {e}"),
            }
        }

        Commands::LargeFiles { limit, threshold } => {
            let cmd = ParallelLargeFilesCommand::new(threshold, Some(limit));
            match cmd.execute_parallel() {
                Ok(output) => println!("{output}"),
                Err(e) => eprintln!("❌ {e}"),
            }
        }

        Commands::Fixup {
            commit_hash,
            rebase,
        } => {
            let cmd = FixupCommand::new(commit_hash, rebase);
            match NewCommand::execute(&cmd) {
                Ok(output) => println!("{output}"),
                Err(e) => eprintln!("❌ {e}"),
            }
        }
        Commands::StashBranch { action } => {
            use git_x::commands::stash::{StashBranchAction as StashAction, StashCommand};

            // Convert CLI action to stash action
            let stash_action = match action {
                git_x::cli::StashBranchAction::Create {
                    branch_name,
                    stash_ref,
                } => StashAction::Create {
                    branch_name,
                    stash_ref,
                },
                git_x::cli::StashBranchAction::Clean {
                    older_than,
                    dry_run,
                } => StashAction::Clean {
                    older_than,
                    dry_run,
                },
                git_x::cli::StashBranchAction::ApplyByBranch {
                    branch_name,
                    list_only,
                } => StashAction::ApplyByBranch {
                    branch_name,
                    list_only,
                },
                git_x::cli::StashBranchAction::Interactive => StashAction::Interactive,
                git_x::cli::StashBranchAction::Export {
                    output_dir,
                    stash_ref,
                } => StashAction::Export {
                    output_dir,
                    stash_ref,
                },
            };

            let cmd = StashCommand::new(stash_action);
            match NewCommand::execute(&cmd) {
                Ok(output) => println!("{output}"),
                Err(e) => eprintln!("❌ {e}"),
            }
        }

        Commands::Upstream { action } => {
            // Convert CLI action to repository action
            let repo_action = match action {
                git_x::cli::UpstreamAction::Set { upstream } => {
                    // Parse "remote/branch" format
                    let parts: Vec<&str> = upstream.split('/').collect();
                    if parts.len() >= 2 {
                        git_x::commands::repository::UpstreamAction::Set {
                            remote: parts[0].to_string(),
                            branch: parts[1..].join("/"),
                        }
                    } else {
                        git_x::commands::repository::UpstreamAction::Set {
                            remote: "origin".to_string(),
                            branch: upstream,
                        }
                    }
                }
                git_x::cli::UpstreamAction::Status => {
                    git_x::commands::repository::UpstreamAction::Status
                }
                git_x::cli::UpstreamAction::SyncAll {
                    dry_run: _,
                    merge: _,
                } => git_x::commands::repository::UpstreamAction::SyncAll,
            };

            let cmd = AsyncUpstreamCommand::new(repo_action);
            match cmd.execute_parallel().await {
                Ok(output) => println!("{output}"),
                Err(e) => eprintln!("❌ {e}"),
            }
        }

        Commands::SwitchRecent => {
            use git_x::commands::branch::SwitchRecentCommand;
            let cmd = SwitchRecentCommand::new();
            match NewCommand::execute(&cmd) {
                Ok(output) => println!("{output}"),
                Err(e) => eprintln!("❌ {e}"),
            }
        }

        Commands::Contributors => {
            let cmd = ParallelContributorsCommand::new(None);
            match cmd.execute_parallel() {
                Ok(output) => println!("{output}"),
                Err(e) => eprintln!("❌ {e}"),
            }
        }

        Commands::TechnicalDebt => {
            let cmd = ParallelTechnicalDebtCommand::new();
            match cmd.execute_parallel() {
                Ok(output) => println!("{output}"),
                Err(e) => eprintln!("❌ {e}"),
            }
        }

        Commands::Bisect { action } => {
            use git_x::commands::commit::BisectAction as CommitBisectAction;

            // Convert CLI action to commit action (parameter order difference)
            let commit_action = match action {
                git_x::cli::BisectAction::Start { good, bad } => {
                    CommitBisectAction::Start { bad, good } // Note: swapped order
                }
                git_x::cli::BisectAction::Good => CommitBisectAction::Good,
                git_x::cli::BisectAction::Bad => CommitBisectAction::Bad,
                git_x::cli::BisectAction::Skip => CommitBisectAction::Skip,
                git_x::cli::BisectAction::Reset => CommitBisectAction::Reset,
                git_x::cli::BisectAction::Status => CommitBisectAction::Status,
            };

            let cmd = BisectCommand::new(commit_action);
            match NewCommand::execute(&cmd) {
                Ok(output) => println!("{output}"),
                Err(e) => eprintln!("❌ {e}"),
            }
        }

        Commands::CompletionInstall { shell } => {
            use git_x::commands::completion::CompletionInstallCommand;
            let cmd = CompletionInstallCommand::new(shell);
            match git_x::core::traits::Command::execute(&cmd) {
                Ok(output) => println!("{output}"),
                Err(e) => eprintln!("❌ {e}"),
            }
        }
    }
}
