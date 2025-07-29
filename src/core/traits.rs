use crate::Result;

/// Trait for commands that can be executed
pub trait Command {
    /// Execute the command and return the result
    fn execute(&self) -> Result<String>;

    /// Get the command name for help and error messages
    fn name(&self) -> &'static str;

    /// Get a brief description of what the command does
    fn description(&self) -> &'static str;
}

/// Trait for commands that support dry-run mode
pub trait DryRunnable: Command {
    /// Execute the command in dry-run mode
    fn execute_dry_run(&self) -> Result<String>;

    /// Check if dry-run mode is enabled
    fn is_dry_run(&self) -> bool;
}

/// Trait for commands that perform destructive operations
pub trait Destructive: Command {
    /// Get a description of what will be destroyed/changed
    fn destruction_description(&self) -> String;

    /// Confirm the destructive operation with the user
    fn confirm_destruction(&self) -> Result<bool> {
        crate::core::safety::Safety::confirm_destructive_operation(
            self.name(),
            &self.destruction_description(),
        )
    }

    /// Create a backup before the destructive operation
    fn create_backup(&self) -> Result<Option<String>> {
        Ok(None) // Default: no backup
    }
}

/// Trait for commands that work with git repositories
pub trait GitCommand: Command {
    /// Validate that we're in a git repository
    fn validate_git_repo(&self) -> Result<()> {
        crate::core::validation::Validate::in_git_repo()
    }

    /// Get the repository root path
    fn repo_root(&self) -> Result<String> {
        crate::core::git::GitOperations::repo_root()
    }

    /// Get the current branch
    fn current_branch(&self) -> Result<String> {
        crate::core::git::GitOperations::current_branch()
    }
}

/// Trait for commands that support interactive mode
pub trait Interactive: Command {
    /// Check if the command should run in interactive mode
    fn is_interactive(&self) -> bool {
        crate::core::interactive::Interactive::is_interactive()
    }

    /// Run the command in non-interactive mode (for CI/testing)
    fn execute_non_interactive(&self) -> Result<String>;
}

/// Trait for formatting output
pub trait Formatter {
    /// Format output for display
    fn format(&self, content: &str) -> String;
}

/// Trait for validating inputs
pub trait Validator<T: ?Sized> {
    /// Validate the input
    fn validate(&self, input: &T) -> Result<()>;

    /// Get validation rules for display to users
    fn validation_rules(&self) -> Vec<&'static str>;
}

/// Trait for git operations that can be optimized
pub trait Optimizable {
    /// Execute with optimization (batching, caching, etc.)
    fn execute_optimized(&self) -> Result<String>;
}

/// Trait for commands that support different output formats
pub trait MultiFormat: Command {
    /// Available output formats
    fn supported_formats(&self) -> Vec<&'static str>;

    /// Execute with specific format
    fn execute_with_format(&self, format: &str) -> Result<String>;
}

/// Trait for commands that can be configured
pub trait Configurable: Command {
    /// Configuration type for this command
    type Config;

    /// Apply configuration to the command
    fn with_config(self, config: Self::Config) -> Self;
}
