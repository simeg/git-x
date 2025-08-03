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

// Examples showing architecture migration
#[cfg(test)]
pub mod examples;

/// Common error type for git-x operations
#[derive(Debug)]
pub enum GitXError {
    GitCommand(String),
    Io(std::io::Error),
    Parse(String),
    Dialog(String),
    Join(String),
    Other(String),
}

impl std::fmt::Display for GitXError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GitXError::GitCommand(cmd) => write!(f, "Git command failed: {cmd}"),
            GitXError::Io(err) => write!(f, "IO error: {err}"),
            GitXError::Parse(msg) => write!(f, "Parse error: {msg}"),
            GitXError::Dialog(msg) => write!(f, "Dialog error: {msg}"),
            GitXError::Join(msg) => write!(f, "Join error: {msg}"),
            GitXError::Other(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for GitXError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            GitXError::Io(err) => Some(err),
            GitXError::GitCommand(_)
            | GitXError::Parse(_)
            | GitXError::Dialog(_)
            | GitXError::Join(_)
            | GitXError::Other(_) => None,
        }
    }
}

impl From<std::io::Error> for GitXError {
    fn from(err: std::io::Error) -> Self {
        GitXError::Io(err)
    }
}

impl From<dialoguer::Error> for GitXError {
    fn from(err: dialoguer::Error) -> Self {
        GitXError::Dialog(err.to_string())
    }
}

impl From<tokio::task::JoinError> for GitXError {
    fn from(err: tokio::task::JoinError) -> Self {
        GitXError::Join(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, GitXError>;
