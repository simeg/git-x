// Core abstractions and utilities
pub mod core;

// Domain layer - business logic
pub mod domain;

// Adapter layer - connects CLI to domain
pub mod adapters;

// Command implementations organized by domain
pub mod commands;

// CLI interface
pub mod cli;

// Test utilities for direct command testing (improves test coverage)
pub mod test_utils;

// Examples showing architecture migration
#[cfg(test)]
pub mod examples;

// Legacy module exports for backward compatibility
// These will eventually be removed as we migrate to the new structure

// Module exports
pub mod bisect;
pub mod clean_branches;
pub mod color_graph;
pub mod command;
pub mod contributors;
pub mod fixup;
pub mod graph;
pub mod health;
pub mod info;
pub mod large_files;
pub mod new_branch;
pub mod prune_branches;
pub mod rename_branch;
pub mod since;
pub mod stash_branch;
pub mod summary;
pub mod switch_recent;
pub mod sync;
pub mod technical_debt;
pub mod undo;
pub mod upstream;
pub mod what;

/// Common error type for git-x operations
#[derive(Debug)]
pub enum GitXError {
    GitCommand(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for GitXError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GitXError::GitCommand(cmd) => write!(f, "Git command failed: {cmd}"),
            GitXError::Io(err) => write!(f, "IO error: {err}"),
            GitXError::Parse(msg) => write!(f, "Parse error: {msg}"),
        }
    }
}

impl std::error::Error for GitXError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            GitXError::Io(err) => Some(err),
            GitXError::GitCommand(_) | GitXError::Parse(_) => None,
        }
    }
}

impl From<std::io::Error> for GitXError {
    fn from(err: std::io::Error) -> Self {
        GitXError::Io(err)
    }
}

pub type Result<T> = std::result::Result<T, GitXError>;
