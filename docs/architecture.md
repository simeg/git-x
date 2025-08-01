# Git-X Architecture Documentation

This document describes the layered architecture of the git-x CLI tool, designed for maintainability, testability, and extensibility.

## Architecture Overview

The git-x codebase is organized into distinct layers with clear responsibilities:

```
┌─────────────────────┐
│   CLI Layer         │  ← User interaction, argument parsing (cli.rs)
├─────────────────────┤
│   Command Layer     │  ← Command implementations (commands/)
├─────────────────────┤
│   Adapter Layer     │  ← Output formatting & CLI handling (adapters/)
├─────────────────────┤
│   Domain Layer      │  ← Business logic, workflows (domain/)
├─────────────────────┤
│   Core Layer        │  ← Git operations, utilities (core/)
└─────────────────────┘
```

## Layer Descriptions

### 1. Core Layer (`src/core/`)

**Purpose**: Low-level utilities and abstractions for git operations.

**Modules**:
- `traits.rs` - Common trait abstractions (`Command`, `GitRepository`, `Destructive`)
- `git.rs` - Git operation wrappers and safe command execution
- `output.rs` - Output formatting, buffering utilities, and progress indicators
- `validation.rs` - Input validation with security focus (shell injection prevention)
- `interactive.rs` - Interactive UI utilities with fuzzy search and confirmations
- `safety.rs` - Safety mechanisms for destructive operations

**Responsibilities**:
- Execute git commands safely with proper error handling
- Provide interactive prompts and fuzzy search functionality
- Validate user input to prevent security issues
- Implement safety checks for destructive operations
- Format and buffer output efficiently

### 2. Domain Layer (`src/domain/`)

**Purpose**: Business logic and domain-specific workflows.

**Current Modules**:
- `git_repository.rs` - Repository-level operations and state management
- `branch_manager.rs` - Branch lifecycle management and operations

**Responsibilities**:
- Implement domain-specific business rules
- Coordinate complex multi-step workflows
- Provide type-safe APIs for repository operations
- Handle domain-specific validation and constraints
- Manage operation state and context

**Key Concepts**:
- **Domain Services**: High-level operation coordinators (e.g., `BranchManager`)
- **Repository Abstraction**: Clean interface for git operations
- **Business Rules**: Domain-specific validation and logic

### 3. Adapter Layer (`src/adapters/`)

**Purpose**: Bridge between CLI and domain layers, handling output formatting.

**Modules**:
- `cli_handlers.rs` - CLI command handlers and workflow coordination
- `formatters.rs` - Rich output formatting with colors, emojis, and tables

**Responsibilities**:
- Convert CLI arguments to domain operations
- Handle CLI-specific concerns (interactive vs non-interactive mode)
- Format command output with consistent styling
- Provide reusable formatting utilities (tables, progress bars)
- Manage CLI workflow coordination and error presentation

### 4. Command Layer (`src/commands/`)

**Purpose**: Organized command implementations by functional area.

**Modules**:
- `branch.rs` - Branch management commands (clean, prune, rename, switch-recent, stash-branch)
- `commit.rs` - Commit operations (fixup, undo, bisect)
- `repository.rs` - Repository-level commands (info, health, sync, upstream, new)
- `analysis.rs` - Analysis and reporting (summary, graph, contributors, technical-debt, large-files, since, what)
- `stash.rs` - Stash operations (stash-branch)

**Command Count**: 21 total commands across 6 functional categories

**Key Features**:
- Each command implements the `Command` trait for consistency
- Commands are organized by domain area for maintainability
- Standardized error handling and output formatting
- Support for both interactive and non-interactive modes

### 5. CLI Layer (`src/cli.rs` + `src/main.rs`)

**Purpose**: Command-line interface definition, argument parsing, and application entry point.

**Key Files**:
- `cli.rs` - CLI structure definition with clap derive macros
- `main.rs` - Application entry point and command dispatch

**Responsibilities**:
- Define CLI structure with nested subcommands using clap
- Parse command-line arguments with validation
- Route commands to appropriate implementations
- Handle global CLI concerns (help, version, shell completions)
- Generate shell completions for bash, zsh, fish, PowerShell, and Elvish

## Design Patterns Used

### 1. Command Pattern
All commands implement the `Command` trait with standardized execution methods:

```rust
pub trait Command {
    fn execute(&self) -> Result<String>;
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
}
```

### 2. Trait-Based Architecture
Core functionality is built around traits for flexibility and testability:

```rust
pub trait GitRepository {
    fn current_branch(&self) -> Result<String>;
    fn branch_exists(&self) -> Result<bool>;
    // ... other git operations
}

pub trait Destructive {
    fn safety_checks(&self) -> Result<()>;
    fn requires_confirmation(&self) -> bool;
}
```

### 3. Module Organization Pattern
Commands are grouped by functional domain:
- **Repository Operations**: Info, health, sync, upstream management
- **Branch Management**: Create, clean, prune, rename, switch
- **Commit Operations**: Fixup, undo, bisect workflows
- **Analysis & Reporting**: Statistics, graphs, technical debt analysis

### 4. Safety-First Design
Destructive operations implement safety mechanisms:
- Confirmation prompts for destructive actions
- Non-interactive mode support for CI/CD
- Validation of git repository state before operations
- Clear error messages with recovery suggestions

## Benefits of This Architecture

### 1. **Clear Separation of Concerns**
- CLI parsing separated from command logic
- Git operations isolated in core layer
- Output formatting centralized in adapters
- Domain logic encapsulated in dedicated layer

### 2. **Enhanced Testability**
- Each command can be unit tested independently
- Core git operations mockable through traits
- Domain logic testable without CLI dependencies
- Integration tests validate end-to-end workflows

### 3. **Improved Maintainability**
- Commands organized by functional domain
- Consistent patterns across all implementations
- Clear module boundaries and responsibilities
- Reduced code duplication through shared utilities

### 4. **Type Safety & Reliability**
- Rust's type system prevents common errors
- Comprehensive error handling with `Result<T>`
- Input validation prevents security vulnerabilities
- Compile-time guarantees for data flow

### 5. **User Experience Focus**
- Rich output formatting with colors and emojis
- Interactive prompts with fuzzy search
- Consistent help system and error messages
- Shell completion support across platforms

### 6. **Extensibility**
- New commands follow established patterns
- Plugin architecture possible through trait system
- Alternative interfaces can reuse core and domain layers
- Easy to add new git workflow automations

## Current Implementation Status

### ✅ Completed Components
1. **Core Layer**: Complete with traits, git operations, validation, and safety
2. **Command Layer**: 21 commands implemented across 6 functional areas
3. **CLI Layer**: Full clap-based CLI with shell completion support
4. **Output System**: Rich formatting with colors, emojis, and interactive elements
5. **Testing**: Comprehensive test suite with integration and unit tests

### 🚧 Partial Implementation
1. **Domain Layer**: Basic branch manager implemented, extensible for more complex workflows
2. **Adapter Layer**: Output formatting complete, CLI handlers can be expanded

### 📋 Future Enhancements
1. **Enhanced Domain Services**: More sophisticated workflow coordination
2. **Configuration Management**: User preferences and project-specific settings
3. **Plugin System**: External command integration
4. **Performance Optimizations**: Parallel git operations and caching

## Implementation Examples

### Command Implementation Pattern

All commands follow this consistent pattern:

```rust
use crate::core::traits::Command;
use crate::core::git::GitOperations;

pub struct ExampleCommand {
    param: String,
}

impl ExampleCommand {
    pub fn new(param: String) -> Self {
        Self { param }
    }
}

impl Command for ExampleCommand {
    fn execute(&self) -> Result<String> {
        // 1. Validate input
        // 2. Perform git operations
        // 3. Format output
        Ok("Command completed successfully".to_string())
    }

    fn name(&self) -> &'static str {
        "example"
    }

    fn description(&self) -> &'static str {
        "Example command description"
    }
}
```

### Adding a New Command

1. **Create command struct** in appropriate `commands/` module
2. **Implement `Command` trait** with execute logic
3. **Add to CLI enum** in `cli.rs`
4. **Add dispatch logic** in `main.rs`
5. **Write integration tests** following existing patterns

### Testing Strategy

The project uses comprehensive testing across all layers:

#### Unit Tests
```rust
#[test]
#[serial]
fn test_command_execution() {
    let repo = common::basic_repo();
    let original_dir = std::env::current_dir().unwrap();
    
    std::env::set_current_dir(repo.path()).unwrap();
    
    let cmd = ExampleCommand::new("test".to_string());
    let result = cmd.execute();
    
    assert!(result.is_ok());
    let _ = std::env::set_current_dir(&original_dir);
}
```

#### Integration Tests
```rust
#[test]
#[serial]
fn test_cli_integration() {
    let repo = common::basic_repo();
    
    repo.run_git_x(&["example", "test"])
        .success()
        .stdout(contains("Expected output"));
}
```

## Code Quality & Security Features

### Error Handling
- Consistent `Result<String>` return types across all commands
- Descriptive error messages with recovery suggestions
- Graceful degradation in non-git directories
- Proper error propagation through all layers

### Security & Safety
- **Input Validation**: Prevents shell injection in git commands
- **Destructive Operation Guards**: Confirmation prompts for dangerous actions
- **Non-Interactive Mode**: Safe CI/CD execution with `GIT_X_NON_INTERACTIVE`
- **Branch Name Validation**: Prevents invalid git branch names
- **Working Directory Checks**: Validates git repository state before operations

### Performance Optimizations
- **Output Buffering**: Efficient terminal output with `BufferedOutput`
- **Optimized Git Commands**: Minimal git calls with proper argument handling
- **Interactive Search**: Fast fuzzy search for branch/commit selection
- **Parallel Test Execution**: Test suite optimized for CI/CD environments

### User Experience
- **Rich Output**: Colors, emojis, and formatted tables
- **Progress Indicators**: Visual feedback for long-running operations
- **Help System**: Comprehensive help text with examples
- **Shell Completions**: Auto-complete support for all major shells


## Summary

The git-x architecture demonstrates a mature, production-ready CLI tool with:

- **21 git workflow commands** organized across 6 functional domains
- **Layered architecture** with clear separation of concerns
- **Comprehensive testing** with 300+ tests ensuring reliability
- **Security-first design** with input validation and safety guards
- **Rich user experience** with interactive prompts and formatted output
- **Cross-platform support** including shell completions

This architecture provides a solid foundation for continued development while maintaining the tool's core focus: simplifying and enhancing git workflows for developers.

### Key Metrics
- **Commands**: 21 total across 6 categories
- **Test Coverage**: 300+ tests with comprehensive scenarios
- **File Organization**: ~50 source files in logical modules
- **Safety Features**: Confirmation prompts, validation, non-interactive mode
- **Platform Support**: Linux, macOS, Windows with shell completions