# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`git-x` is a Rust CLI tool that provides Git workflow extensions as subcommands. It's designed to work as a Git plugin - when installed, commands like `git x info` automatically invoke the `git-x` binary.

## Development Commands

### Build and Test
- `make ci` - Run all CI checks (formatting, linting, tests)
- `make build` - Build release binary
- `make test` - Run unit and integration tests
- `cargo test` - Alternative test command
- `cargo test <test_name>` - Run specific test
- `make coverage` - Generate test coverage using tarpaulin

### Code Quality
- `make fmt` - Format code with rustfmt
- `make fmt-check` - Check formatting without changes
- `make lint` - Lint with Clippy
- `make lint-check` - Check linting without changes

### Installation and Running
- `make install` - Install to ~/.cargo/bin for Git integration
- `make run ARGS="info"` - Build and run with arguments
- `cargo run -- <command>` - Run directly with cargo

## Architecture

### Core Structure
- `src/main.rs` - Entry point with command dispatch and completion generation
- `src/cli.rs` - Clap-based command definitions and argument parsing
- `src/lib.rs` - Module exports for all commands
- `src/<command>.rs` - Individual command implementations

### Command Implementation Pattern
Each Git workflow command is implemented as a separate module:
- Commands execute native Git operations via `std::process::Command`
- Output is formatted for human readability with emojis and colors
- Error handling uses `expect()` with descriptive messages

### Key Dependencies
- `clap` - Command-line parsing with derive macros and subcommands
- `console` - Terminal output formatting with colors and emojis
- `chrono` - Date/time handling for summary command

### Dev Dependencies
- `assert_cmd` - CLI testing framework for integration tests
- `tempfile` - Temporary Git repositories for testing
- `predicates` - Assertion helpers for test conditions
- `criterion` - Benchmarking framework

### Testing
- Integration tests in `tests/` directory
- Each command has corresponding `test_<command>.rs` file
- Uses `assert_cmd` for CLI testing and `tempfile` for test repos

### Git Plugin Integration
The binary name `git-x` enables Git's plugin system - any executable named `git-<name>` can be invoked as `git <name>`. Commands like `git x info` work automatically once installed.

### Error Handling and Types
- Common error type `GitXError` defined in `src/lib.rs` with variants for Git commands, IO, and parsing
- Most commands use `expect()` with descriptive messages for quick failure feedback
- Type alias `Result<T>` available for consistent error handling

### Shell Completion Generation
Shell completions can be generated via `--generate-completions <shell>` flag using `clap_complete`. The README provides setup instructions for bash, zsh, fish, PowerShell, and Elvish.