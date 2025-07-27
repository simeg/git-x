use clap::Parser;
use git_x::cli::{Cli, Commands};

#[test]
fn test_cli_parse_rename_branch() {
    let cli = Cli::try_parse_from(["git-x", "rename-branch", "new-name"]).unwrap();
    match cli.command {
        Commands::RenameBranch { new_name } => {
            assert_eq!(new_name, "new-name");
        }
        _ => panic!("Expected RenameBranch command"),
    }
}

#[test]
fn test_cli_parse_prune_branches() {
    let cli = Cli::try_parse_from(["git-x", "prune-branches"]).unwrap();
    match cli.command {
        Commands::PruneBranches { except } => {
            assert!(except.is_none());
        }
        _ => panic!("Expected PruneBranches command"),
    }
}

#[test]
fn test_cli_parse_prune_branches_with_except() {
    let cli = Cli::try_parse_from(["git-x", "prune-branches", "--except", "main,develop"]).unwrap();
    match cli.command {
        Commands::PruneBranches { except } => {
            assert_eq!(except, Some("main,develop".to_string()));
        }
        _ => panic!("Expected PruneBranches command"),
    }
}

#[test]
fn test_cli_parse_info() {
    let cli = Cli::try_parse_from(["git-x", "info"]).unwrap();
    match cli.command {
        Commands::Info => {}
        _ => panic!("Expected Info command"),
    }
}

#[test]
fn test_cli_parse_graph() {
    let cli = Cli::try_parse_from(["git-x", "graph"]).unwrap();
    match cli.command {
        Commands::Graph => {}
        _ => panic!("Expected Graph command"),
    }
}

#[test]
fn test_cli_parse_color_graph() {
    let cli = Cli::try_parse_from(["git-x", "color-graph"]).unwrap();
    match cli.command {
        Commands::ColorGraph => {}
        _ => panic!("Expected ColorGraph command"),
    }
}

#[test]
fn test_cli_parse_health() {
    let cli = Cli::try_parse_from(["git-x", "health"]).unwrap();
    match cli.command {
        Commands::Health => {}
        _ => panic!("Expected Health command"),
    }
}

#[test]
fn test_cli_parse_since() {
    let cli = Cli::try_parse_from(["git-x", "since", "main"]).unwrap();
    match cli.command {
        Commands::Since { reference } => {
            assert_eq!(reference, "main");
        }
        _ => panic!("Expected Since command"),
    }
}

#[test]
fn test_cli_parse_undo() {
    let cli = Cli::try_parse_from(["git-x", "undo"]).unwrap();
    match cli.command {
        Commands::Undo => {}
        _ => panic!("Expected Undo command"),
    }
}

#[test]
fn test_cli_parse_clean_branches() {
    let cli = Cli::try_parse_from(["git-x", "clean-branches"]).unwrap();
    match cli.command {
        Commands::CleanBranches { dry_run } => {
            assert!(!dry_run);
        }
        _ => panic!("Expected CleanBranches command"),
    }
}

#[test]
fn test_cli_parse_clean_branches_dry_run() {
    let cli = Cli::try_parse_from(["git-x", "clean-branches", "--dry-run"]).unwrap();
    match cli.command {
        Commands::CleanBranches { dry_run } => {
            assert!(dry_run);
        }
        _ => panic!("Expected CleanBranches command"),
    }
}

#[test]
fn test_cli_parse_what() {
    let cli = Cli::try_parse_from(["git-x", "what"]).unwrap();
    match cli.command {
        Commands::What { target } => {
            assert!(target.is_none());
        }
        _ => panic!("Expected What command"),
    }
}

#[test]
fn test_cli_parse_what_with_target() {
    let cli = Cli::try_parse_from(["git-x", "what", "--target", "develop"]).unwrap();
    match cli.command {
        Commands::What { target } => {
            assert_eq!(target, Some("develop".to_string()));
        }
        _ => panic!("Expected What command"),
    }
}

#[test]
fn test_cli_parse_summary() {
    let cli = Cli::try_parse_from(["git-x", "summary", "--since", "3 days ago"]).unwrap();
    match cli.command {
        Commands::Summary { since } => {
            assert_eq!(since, "3 days ago");
        }
        _ => panic!("Expected Summary command"),
    }
}
