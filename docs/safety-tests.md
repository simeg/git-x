# Safety Tests Documentation

## Overview

The `tests/test_safety.rs` module contains tests for git-x's safety and backup functionality. Some of these tests perform **DESTRUCTIVE git operations** that can modify your working directory.

## Destructive Operations

The following operations are considered destructive and are conditionally run:

- **Creating backup branches**: Adds new git branches to your repository
- **Creating checkpoints**: Creates git stashes that modify stash state
- **Restoring checkpoints**: Pops git stashes, potentially changing working directory
- **Cleaning up branches**: Can delete backup branches (though limited to very old ones)

## Test Execution Modes

### Local Development (Default - Safe Mode)
```bash
cargo test test_safety
```
- Destructive tests are **SKIPPED** automatically
- Only safe tests run (method signatures, builders, validation)
- Your git repository remains unchanged

### CI Environment (Automatic)
```bash
# In GitHub Actions or other CI
cargo test test_safety
```
- All tests run automatically (detected via `CI` or `GITHUB_ACTIONS` env vars)
- Safe for CI because repositories are disposable

### Manual Override (Use with Caution)
```bash
ENABLE_DESTRUCTIVE_TESTS=1 cargo test test_safety
```
- Forces ALL tests to run locally
- **WARNING**: Will modify your git repository
- Only use in test repositories or when you understand the risks

## Detection Logic

Tests are skipped locally unless one of these conditions is met:
- `CI` environment variable is set
- `GITHUB_ACTIONS` environment variable is set  
- `ENABLE_DESTRUCTIVE_TESTS` environment variable is set

## Safety Measures

Even when destructive tests run:
- Backup creation uses timestamp-based names to avoid conflicts
- Cleanup operations target only very old backups (365+ days)
- All operations use non-interactive mode to prevent hanging
- Tests are designed to be as safe as possible while still testing functionality

## Development Guidelines

When adding new safety tests:
1. Wrap potentially destructive operations with `if !should_run_destructive_tests() { return; }`
2. Use descriptive test names that indicate destructive nature
3. Test the minimal necessary functionality to verify behavior
4. Prefer testing business logic over actual git operations when possible