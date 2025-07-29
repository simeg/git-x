use crate::Result;

/// Trait for all git-x commands
///
/// This trait provides a unified interface for all git-x commands, allowing for
/// consistent error handling, testing, and potential plugin architecture.
pub trait Command {
    /// The input type for this command (can be () for no input)
    type Input;

    /// The output type for this command
    type Output;

    /// Execute the command with the given input
    fn execute(&self, input: Self::Input) -> Result<Self::Output>;

    /// Get the name of this command (for logging/debugging)
    fn name(&self) -> &'static str;

    /// Get a description of what this command does
    fn description(&self) -> &'static str;

    /// Whether this command requires a git repository to function
    fn requires_git_repo(&self) -> bool {
        true
    }

    /// Whether this command performs destructive operations
    fn is_destructive(&self) -> bool {
        false
    }
}

/// Trait for commands that don't return data but perform actions (print output)
pub trait ActionCommand: Command<Output = ()> {
    /// Execute the command and handle output internally
    fn run(&self, input: Self::Input) -> Result<()> {
        self.execute(input)
    }
}

/// Trait for commands that return formatted output
pub trait QueryCommand: Command<Output = String> {
    /// Execute the command and return formatted output
    fn query(&self, input: Self::Input) -> Result<String> {
        self.execute(input)
    }
}

/// Auto-implement ActionCommand for commands that output ()
impl<T> ActionCommand for T where T: Command<Output = ()> {}

/// Auto-implement QueryCommand for commands that output String  
impl<T> QueryCommand for T where T: Command<Output = String> {}

/// Helper trait for commands with no input parameters
pub trait SimpleCommand: Command<Input = ()> {
    fn run_simple(&self) -> Result<Self::Output> {
        self.execute(())
    }
}

/// Auto-implement SimpleCommand for commands with () input
impl<T> SimpleCommand for T where T: Command<Input = ()> {}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockActionCommand;

    impl Command for MockActionCommand {
        type Input = String;
        type Output = ();

        fn execute(&self, input: String) -> Result<()> {
            println!("Mock action: {input}");
            Ok(())
        }

        fn name(&self) -> &'static str {
            "mock-action"
        }

        fn description(&self) -> &'static str {
            "A mock action command for testing"
        }
    }

    struct MockQueryCommand;

    impl Command for MockQueryCommand {
        type Input = ();
        type Output = String;

        fn execute(&self, _input: ()) -> Result<String> {
            Ok("Mock query result".to_string())
        }

        fn name(&self) -> &'static str {
            "mock-query"
        }

        fn description(&self) -> &'static str {
            "A mock query command for testing"
        }
    }

    #[test]
    fn test_action_command_trait() {
        let cmd = MockActionCommand;
        assert_eq!(cmd.name(), "mock-action");
        assert_eq!(cmd.description(), "A mock action command for testing");
        assert!(cmd.requires_git_repo());
        assert!(!cmd.is_destructive());

        let result = cmd.run("test input".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn test_query_command_trait() {
        let cmd = MockQueryCommand;
        assert_eq!(cmd.name(), "mock-query");
        assert_eq!(cmd.description(), "A mock query command for testing");

        let result = cmd.query(());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Mock query result");
    }

    #[test]
    fn test_simple_command_trait() {
        let cmd = MockQueryCommand;
        let result = cmd.run_simple();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Mock query result");
    }
}
