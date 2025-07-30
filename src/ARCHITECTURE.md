# Git-X Architecture Documentation

This document describes the new layered architecture implemented for the git-x CLI tool, designed to improve maintainability, testability, and extensibility.

## Architecture Overview

The codebase is now organized into distinct layers with clear responsibilities:

```
┌─────────────────────┐
│   CLI Layer         │  ← User interaction, argument parsing
├─────────────────────┤
│   Adapter Layer     │  ← Bridges CLI and Domain
├─────────────────────┤
│   Domain Layer      │  ← Business logic, workflows
├─────────────────────┤
│   Core Layer        │  ← Git operations, utilities
└─────────────────────┘
```

## Layer Descriptions

### 1. Core Layer (`src/core/`)

**Purpose**: Low-level utilities and abstractions for git operations.

**Modules**:
- `traits.rs` - Common trait abstractions (Command, GitCommand, Destructive, etc.)
- `git.rs` - Git operation wrappers (GitOperations, BranchOperations, etc.)
- `output.rs` - Output formatting and buffering utilities
- `validation.rs` - Input validation with security focus
- `interactive.rs` - Interactive UI utilities with fuzzy search
- `safety.rs` - Safety mechanisms for destructive operations

**Responsibilities**:
- Execute git commands safely
- Provide consistent error handling
- Handle user input validation
- Manage interactive prompts
- Implement safety checks

### 2. Domain Layer (`src/domain/`)

**Purpose**: Business logic and domain-specific workflows.

**Modules**:
- `git_repository.rs` - Repository-level operations and state
- `branch_manager.rs` - Branch lifecycle management
- `commit_manager.rs` - Commit-related operations
- `analysis_engine.rs` - Repository analysis and reporting

**Responsibilities**:
- Implement business rules
- Coordinate complex workflows
- Provide type-safe APIs
- Handle domain-specific validation
- Manage operation state

**Key Concepts**:
- **Request/Response DTOs**: Strongly-typed data transfer objects
- **Domain Services**: High-level operation coordinators
- **Business Rules**: Domain-specific validation and logic

### 3. Adapter Layer (`src/adapters/`)

**Purpose**: Bridge between CLI and domain layers.

**Modules**:
- `cli_handlers.rs` - CLI command handlers
- `formatters.rs` - Output formatting for CLI

**Responsibilities**:
- Convert CLI arguments to domain requests
- Handle CLI-specific concerns (interactive vs non-interactive)
- Format domain responses for CLI output
- Manage CLI workflow coordination

### 4. Command Layer (`src/commands/`)

**Purpose**: Organized command implementations by functional area.

**Modules**:
- `branch.rs` - Branch-related commands
- `commit.rs` - Commit-related commands
- `repository.rs` - Repository-level commands
- `analysis.rs` - Analysis and reporting commands

### 5. CLI Layer (`src/cli.rs`)

**Purpose**: Command-line interface definition and argument parsing.

**Responsibilities**:
- Define CLI structure with clap
- Parse command-line arguments
- Route commands to appropriate handlers
- Handle global CLI concerns

## Design Patterns Used

### 1. Command Pattern
Commands implement the `Command` trait with standardized execution methods.

```rust
pub trait Command {
    fn execute(&self) -> Result<String>;
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
}
```

### 2. Request/Response Pattern
Domain operations use typed request/response objects:

```rust
pub struct CreateBranchRequest {
    pub name: String,
    pub from: Option<String>,
    pub create_backup: bool,
}

pub struct BranchCreationResult {
    pub branch_name: String,
    pub backup_branch: Option<String>,
    pub switched: bool,
}
```

### 3. Builder Pattern
Complex operations use builders for configuration:

```rust
SafetyBuilder::new("delete branches")
    .with_backup()
    .with_confirmation()
    .execute(|| { /* operation */ })
```

### 4. Factory Pattern
Handlers are created through factories:

```rust
let handler = CliHandlerFactory::create_branch_handler()?;
```

## Benefits of This Architecture

### 1. **Separation of Concerns**
- CLI logic separated from business logic
- Git operations isolated from domain rules
- Output formatting decoupled from data processing

### 2. **Testability**
- Each layer can be unit tested independently
- Domain logic testable without CLI dependencies
- Mock implementations possible at layer boundaries

### 3. **Maintainability**
- Changes to git operations don't affect CLI
- Business logic changes isolated to domain layer
- Clear responsibility boundaries

### 4. **Type Safety**
- Strongly-typed request/response objects
- Compile-time validation of data flow
- Reduced runtime errors

### 5. **Extensibility**
- New commands easily added through existing patterns
- Alternative interfaces (GUI, API) can reuse domain layer
- Plugin architecture possible

## Migration Strategy



1. **Phase 1**: Core utilities and traits (✅ Complete)
2. **Phase 2**: Domain layer for key operations (✅ Complete)
3. **Phase 3**: Adapter layer and CLI handlers (✅ Complete)
4. **Phase 4**: Migrate existing commands incrementally
5. **Phase 5**: Remove legacy code

## Usage Examples

### Creating a New Command

1. **Define domain operation** in appropriate manager:
```rust
impl BranchManager {
    pub fn create_feature_branch(&self, request: FeatureBranchRequest) -> Result<FeatureBranchResult> {
        // Business logic here
    }
}
```

2. **Add CLI handler** method:
```rust
impl BranchCliHandler {
    pub fn handle_feature_branch(&self, name: String) -> Result<String> {
        let request = FeatureBranchRequest { name };
        let result = self.branch_manager.create_feature_branch(request)?;
        Ok(self.formatter.format_feature_result(&result))
    }
}
```

3. **Add CLI command** definition:
```rust
#[derive(Subcommand)]
pub enum BranchCommands {
    Feature { name: String },
    // ... other commands
}
```

### Testing Domain Logic

```rust
#[test]
fn test_branch_creation_validation() {
    let repo = MockRepository::new();
    let manager = BranchManager::new(repo);
    
    let request = CreateBranchRequest {
        name: "invalid..name".to_string(),
        from: None,
        create_backup: false,
    };
    
    let result = manager.create_branch(request);
    assert!(result.is_err());
}
```

## Code Quality Improvements

### Error Handling
- Consistent `Result<T>` patterns
- Domain-specific error types
- Proper error propagation

### Input Validation
- Security-focused validation (shell injection prevention)
- Type-safe validation with clear error messages
- Validation at appropriate layer boundaries

### Safety Features
- Destructive operation confirmations
- Automatic backup creation
- Working directory state validation
- Recovery mechanisms

### Performance
- Output buffering for better performance
- Optimized git command execution
- Reduced redundant git calls

## Future Enhancements



1. **Configuration System**: Domain-driven configuration management
2. **Plugin System**: Extensible command architecture
3. **API Layer**: REST or GraphQL API using domain layer
4. **GUI Interface**: Desktop or web interface using domain layer
5. **Async Operations**: Parallel git operations for better performance
6. **Caching Layer**: Intelligent caching of git operations
7. **Event System**: Hook system for operation notifications

## Conclusion

This architecture transformation provides a solid foundation for future development while maintaining backward compatibility. The clear separation of concerns, improved testability, and type safety will significantly improve the long-term maintainability of the git-x project.