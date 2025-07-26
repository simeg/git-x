# Makefile for git-x CLI tool

BINARY_NAME = git-x

.PHONY: all build ci run test install uninstall fmt fmt-check lint lint-check clean help

## Build and run the project (default)
all: run

## Build the release binary
build:
	cargo build --release

## Run all CI checks: formatting, linting, and tests
ci: fmt-check lint-check test

## Run the binary with arguments (pass with ARGS="xinfo --help")
run: build
	./target/release/$(BINARY_NAME) $(ARGS)

## Run unit and integration tests
test:
	cargo test

## Format all source files
fmt:
	cargo fmt --all

## Check formatting without modifying files
fmt-check:
	cargo fmt --all -- --check

## Lint the code using Clippy
lint:
	cargo clippy --all-targets -- -D warnings

## Check for linting issues without modifying code
lint-check:
	cargo clippy --all-targets --all-features -- -D warnings

## Install the binary to ~/.cargo/bin (for git plugin use)
install: build
	cargo install --path .

## Uninstall the binary
uninstall:
	cargo uninstall $(BINARY_NAME)

## Clean all build artifacts
clean:
	cargo clean

## Show this help message
help:
	@echo ""
	@echo "git-x Makefile — available targets:"
	@echo "  make           Build and run (default)"
	@echo "  make build     Build release binary"
	@echo "  make run       Run binary with ARGS=\"xinfo\""
	@echo "  make test      Run tests"
	@echo "  make fmt       Format code"
	@echo "  make fmt-check  Check formatting"
	@echo "  make lint      Lint with Clippy"
	@echo "  make lint-check Check for linting issues"
	@echo "  make install   Install to ~/.cargo/bin as 'git-x'"
	@echo "  make uninstall Uninstall binary"
	@echo "  make clean     Remove build artifacts"
	@echo "  make help      Show this help message"
	@echo "  make ci        Run CI checks (formatting, linting, tests)"
	@echo ""
