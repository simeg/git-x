use git_x::core::traits::*;
use git_x::{GitXError, Result};

// Mock implementations for testing traits

struct MockCommand {
    name: &'static str,
    description: &'static str,
    should_fail: bool,
}

impl MockCommand {
    fn new(name: &'static str, description: &'static str) -> Self {
        Self {
            name,
            description,
            should_fail: false,
        }
    }

    fn new_failing(name: &'static str, description: &'static str) -> Self {
        Self {
            name,
            description,
            should_fail: true,
        }
    }
}

impl Command for MockCommand {
    fn execute(&self) -> Result<String> {
        if self.should_fail {
            Err(GitXError::GitCommand("Mock command failed".to_string()))
        } else {
            Ok(format!("Executed command: {}", self.name))
        }
    }

    fn name(&self) -> &'static str {
        self.name
    }

    fn description(&self) -> &'static str {
        self.description
    }
}

struct MockDryRunnableCommand {
    base: MockCommand,
    dry_run: bool,
}

impl MockDryRunnableCommand {
    fn new(name: &'static str, description: &'static str, dry_run: bool) -> Self {
        Self {
            base: MockCommand::new(name, description),
            dry_run,
        }
    }
}

impl Command for MockDryRunnableCommand {
    fn execute(&self) -> Result<String> {
        self.base.execute()
    }

    fn name(&self) -> &'static str {
        self.base.name()
    }

    fn description(&self) -> &'static str {
        self.base.description()
    }
}

impl DryRunnable for MockDryRunnableCommand {
    fn execute_dry_run(&self) -> Result<String> {
        Ok(format!("(dry-run) Would execute: {}", self.name()))
    }

    fn is_dry_run(&self) -> bool {
        self.dry_run
    }
}

struct MockDestructiveCommand {
    base: MockCommand,
    destruction_desc: String,
}

impl MockDestructiveCommand {
    fn new(name: &'static str, description: &'static str, destruction_desc: String) -> Self {
        Self {
            base: MockCommand::new(name, description),
            destruction_desc,
        }
    }
}

impl Command for MockDestructiveCommand {
    fn execute(&self) -> Result<String> {
        self.base.execute()
    }

    fn name(&self) -> &'static str {
        self.base.name()
    }

    fn description(&self) -> &'static str {
        self.base.description()
    }
}

impl Destructive for MockDestructiveCommand {
    fn destruction_description(&self) -> String {
        self.destruction_desc.clone()
    }

    fn create_backup(&self) -> Result<Option<String>> {
        Ok(Some(format!("backup-{}", self.name())))
    }
}

struct MockInteractiveCommand {
    base: MockCommand,
}

impl MockInteractiveCommand {
    fn new(name: &'static str, description: &'static str) -> Self {
        Self {
            base: MockCommand::new(name, description),
        }
    }
}

impl Command for MockInteractiveCommand {
    fn execute(&self) -> Result<String> {
        self.base.execute()
    }

    fn name(&self) -> &'static str {
        self.base.name()
    }

    fn description(&self) -> &'static str {
        self.base.description()
    }
}

impl Interactive for MockInteractiveCommand {
    fn execute_non_interactive(&self) -> Result<String> {
        Ok(format!("Non-interactive execution: {}", self.name()))
    }
}

struct MockFormatter;

impl Formatter for MockFormatter {
    fn format(&self, content: &str) -> String {
        format!("Formatted: {content}")
    }
}

struct MockValidator {
    min_length: usize,
}

impl MockValidator {
    fn new(min_length: usize) -> Self {
        Self { min_length }
    }
}

impl Validator<str> for MockValidator {
    fn validate(&self, input: &str) -> Result<()> {
        if input.len() < self.min_length {
            Err(GitXError::Parse(format!(
                "Input too short: {} characters, minimum {}",
                input.len(),
                self.min_length
            )))
        } else {
            Ok(())
        }
    }

    fn validation_rules(&self) -> Vec<&'static str> {
        vec!["Must be at least N characters long", "Cannot be empty"]
    }
}

struct MockOptimizableCommand {
    base: MockCommand,
}

impl MockOptimizableCommand {
    fn new(name: &'static str, description: &'static str) -> Self {
        Self {
            base: MockCommand::new(name, description),
        }
    }
}

impl Command for MockOptimizableCommand {
    fn execute(&self) -> Result<String> {
        self.base.execute()
    }

    fn name(&self) -> &'static str {
        self.base.name()
    }

    fn description(&self) -> &'static str {
        self.base.description()
    }
}

impl Optimizable for MockOptimizableCommand {
    fn execute_optimized(&self) -> Result<String> {
        Ok(format!("Optimized execution: {}", self.name()))
    }
}

struct MockMultiFormatCommand {
    base: MockCommand,
}

impl MockMultiFormatCommand {
    fn new(name: &'static str, description: &'static str) -> Self {
        Self {
            base: MockCommand::new(name, description),
        }
    }
}

impl Command for MockMultiFormatCommand {
    fn execute(&self) -> Result<String> {
        self.base.execute()
    }

    fn name(&self) -> &'static str {
        self.base.name()
    }

    fn description(&self) -> &'static str {
        self.base.description()
    }
}

impl MultiFormat for MockMultiFormatCommand {
    fn supported_formats(&self) -> Vec<&'static str> {
        vec!["json", "yaml", "table"]
    }

    fn execute_with_format(&self, format: &str) -> Result<String> {
        match format {
            "json" => Ok(format!("{{\"result\": \"Executed {}\"}}", self.name())),
            "yaml" => Ok(format!("result: Executed {}", self.name())),
            "table" => Ok(format!("| Result | Executed {} |", self.name())),
            _ => Err(GitXError::Parse(format!("Unsupported format: {format}"))),
        }
    }
}

#[derive(Debug, Clone)]
struct MockConfig {
    timeout: u32,
    verbose: bool,
}

struct MockConfigurableCommand {
    base: MockCommand,
    config: Option<MockConfig>,
}

impl MockConfigurableCommand {
    fn new(name: &'static str, description: &'static str) -> Self {
        Self {
            base: MockCommand::new(name, description),
            config: None,
        }
    }
}

impl Command for MockConfigurableCommand {
    fn execute(&self) -> Result<String> {
        if let Some(config) = &self.config {
            Ok(format!(
                "Executed {} with config: timeout={}, verbose={}",
                self.name(),
                config.timeout,
                config.verbose
            ))
        } else {
            self.base.execute()
        }
    }

    fn name(&self) -> &'static str {
        self.base.name()
    }

    fn description(&self) -> &'static str {
        self.base.description()
    }
}

impl Configurable for MockConfigurableCommand {
    type Config = MockConfig;

    fn with_config(mut self, config: Self::Config) -> Self {
        self.config = Some(config);
        self
    }
}

// Tests for Command trait

#[test]
fn test_command_trait_basic() {
    let cmd = MockCommand::new("test", "Test command");

    assert_eq!(cmd.name(), "test");
    assert_eq!(cmd.description(), "Test command");

    let result = cmd.execute();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Executed command: test");
}

#[test]
fn test_command_trait_failure() {
    let cmd = MockCommand::new_failing("fail-test", "Failing test command");

    assert_eq!(cmd.name(), "fail-test");
    assert_eq!(cmd.description(), "Failing test command");

    let result = cmd.execute();
    assert!(result.is_err());

    match result {
        Err(GitXError::GitCommand(msg)) => {
            assert_eq!(msg, "Mock command failed");
        }
        _ => panic!("Expected GitCommand error"),
    }
}

// Tests for DryRunnable trait

#[test]
fn test_dry_runnable_trait() {
    let cmd = MockDryRunnableCommand::new("dry-test", "Dry run test", true);

    assert!(cmd.is_dry_run());

    let dry_result = cmd.execute_dry_run();
    assert!(dry_result.is_ok());
    assert_eq!(dry_result.unwrap(), "(dry-run) Would execute: dry-test");

    let normal_result = cmd.execute();
    assert!(normal_result.is_ok());
    assert_eq!(normal_result.unwrap(), "Executed command: dry-test");
}

#[test]
fn test_dry_runnable_trait_not_dry_run() {
    let cmd = MockDryRunnableCommand::new("normal-test", "Normal test", false);

    assert!(!cmd.is_dry_run());

    let dry_result = cmd.execute_dry_run();
    assert!(dry_result.is_ok());
    assert_eq!(dry_result.unwrap(), "(dry-run) Would execute: normal-test");
}

// Tests for Destructive trait

#[test]
fn test_destructive_trait() {
    let cmd = MockDestructiveCommand::new(
        "destroy-test",
        "Destructive test",
        "Will delete test files".to_string(),
    );

    assert_eq!(cmd.destruction_description(), "Will delete test files");

    let backup_result = cmd.create_backup();
    assert!(backup_result.is_ok());
    assert_eq!(
        backup_result.unwrap(),
        Some("backup-destroy-test".to_string())
    );
}

#[test]
fn test_destructive_trait_default_backup() {
    struct DefaultDestructiveCommand {
        base: MockCommand,
    }

    impl Command for DefaultDestructiveCommand {
        fn execute(&self) -> Result<String> {
            self.base.execute()
        }

        fn name(&self) -> &'static str {
            self.base.name()
        }

        fn description(&self) -> &'static str {
            self.base.description()
        }
    }

    impl Destructive for DefaultDestructiveCommand {
        fn destruction_description(&self) -> String {
            "Default destruction".to_string()
        }
        // Uses default create_backup implementation
    }

    let cmd = DefaultDestructiveCommand {
        base: MockCommand::new("default-destroy", "Default destructive"),
    };

    let backup_result = cmd.create_backup();
    assert!(backup_result.is_ok());
    assert_eq!(backup_result.unwrap(), None); // Default implementation returns None
}

// Tests for Interactive trait

#[test]
fn test_interactive_trait() {
    let cmd = MockInteractiveCommand::new("interactive-test", "Interactive test");

    // Test is_interactive (may vary based on environment)
    let _is_interactive = cmd.is_interactive();
    // We can't assert a specific value as it depends on the environment
    // but we can verify the method exists and returns a bool

    let non_interactive_result = cmd.execute_non_interactive();
    assert!(non_interactive_result.is_ok());
    assert_eq!(
        non_interactive_result.unwrap(),
        "Non-interactive execution: interactive-test"
    );
}

// Tests for Formatter trait

#[test]
fn test_formatter_trait() {
    let formatter = MockFormatter;

    let result = formatter.format("test content");
    assert_eq!(result, "Formatted: test content");

    let empty_result = formatter.format("");
    assert_eq!(empty_result, "Formatted: ");
}

// Tests for Validator trait

#[test]
fn test_validator_trait_valid_input() {
    let validator = MockValidator::new(3);

    let result = validator.validate("test");
    assert!(result.is_ok());

    let long_result = validator.validate("this is a long string");
    assert!(long_result.is_ok());
}

#[test]
fn test_validator_trait_invalid_input() {
    let validator = MockValidator::new(5);

    let result = validator.validate("hi");
    assert!(result.is_err());

    match result {
        Err(GitXError::Parse(msg)) => {
            assert!(msg.contains("Input too short"));
            assert!(msg.contains("2 characters"));
            assert!(msg.contains("minimum 5"));
        }
        _ => panic!("Expected Parse error"),
    }
}

#[test]
fn test_validator_trait_validation_rules() {
    let validator = MockValidator::new(1);

    let rules = validator.validation_rules();
    assert_eq!(rules.len(), 2);
    assert_eq!(rules[0], "Must be at least N characters long");
    assert_eq!(rules[1], "Cannot be empty");
}

// Tests for Optimizable trait

#[test]
fn test_optimizable_trait() {
    let cmd = MockOptimizableCommand::new("opt-test", "Optimizable test");

    let normal_result = cmd.execute();
    assert!(normal_result.is_ok());
    assert_eq!(normal_result.unwrap(), "Executed command: opt-test");

    let optimized_result = cmd.execute_optimized();
    assert!(optimized_result.is_ok());
    assert_eq!(optimized_result.unwrap(), "Optimized execution: opt-test");
}

// Tests for MultiFormat trait

#[test]
fn test_multi_format_trait() {
    let cmd = MockMultiFormatCommand::new("format-test", "Multi-format test");

    let formats = cmd.supported_formats();
    assert_eq!(formats, vec!["json", "yaml", "table"]);

    let json_result = cmd.execute_with_format("json");
    assert!(json_result.is_ok());
    assert_eq!(
        json_result.unwrap(),
        "{\"result\": \"Executed format-test\"}"
    );

    let yaml_result = cmd.execute_with_format("yaml");
    assert!(yaml_result.is_ok());
    assert_eq!(yaml_result.unwrap(), "result: Executed format-test");

    let table_result = cmd.execute_with_format("table");
    assert!(table_result.is_ok());
    assert_eq!(table_result.unwrap(), "| Result | Executed format-test |");
}

#[test]
fn test_multi_format_trait_unsupported() {
    let cmd = MockMultiFormatCommand::new("format-test", "Multi-format test");

    let result = cmd.execute_with_format("xml");
    assert!(result.is_err());

    match result {
        Err(GitXError::Parse(msg)) => {
            assert_eq!(msg, "Unsupported format: xml");
        }
        _ => panic!("Expected Parse error"),
    }
}

// Tests for Configurable trait

#[test]
fn test_configurable_trait() {
    let cmd = MockConfigurableCommand::new("config-test", "Configurable test");

    // Test without config
    let result = cmd.execute();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Executed command: config-test");
}

#[test]
fn test_configurable_trait_with_config() {
    let cmd = MockConfigurableCommand::new("config-test", "Configurable test");
    let config = MockConfig {
        timeout: 30,
        verbose: true,
    };

    let configured_cmd = cmd.with_config(config);

    let result = configured_cmd.execute();
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        "Executed config-test with config: timeout=30, verbose=true"
    );
}

// Test trait combinations

#[test]
fn test_trait_combination() {
    // Test that we can implement multiple traits on the same struct
    struct CombinedCommand {
        base: MockCommand,
        dry_run: bool,
    }

    impl Command for CombinedCommand {
        fn execute(&self) -> Result<String> {
            self.base.execute()
        }

        fn name(&self) -> &'static str {
            self.base.name()
        }

        fn description(&self) -> &'static str {
            self.base.description()
        }
    }

    impl DryRunnable for CombinedCommand {
        fn execute_dry_run(&self) -> Result<String> {
            Ok(format!("(dry-run) Combined: {}", self.name()))
        }

        fn is_dry_run(&self) -> bool {
            self.dry_run
        }
    }

    impl Interactive for CombinedCommand {
        fn execute_non_interactive(&self) -> Result<String> {
            Ok(format!("Non-interactive combined: {}", self.name()))
        }
    }

    let cmd = CombinedCommand {
        base: MockCommand::new("combined", "Combined command"),
        dry_run: true,
    };

    // Test Command trait
    assert_eq!(cmd.name(), "combined");
    assert!(cmd.execute().is_ok());

    // Test DryRunnable trait
    assert!(cmd.is_dry_run());
    assert!(cmd.execute_dry_run().is_ok());

    // Test Interactive trait
    assert!(cmd.execute_non_interactive().is_ok());
}
