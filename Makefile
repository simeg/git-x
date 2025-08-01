# Makefile for git-x CLI tool

BINARY_NAME = git-x
CARGO = cargo

.PHONY: all build ci run test coverage install uninstall fmt fmt-check lint lint-check clean publish help publish-optimized analyze-size

## Build and run the project (default)
all: run

## Build the release binary
build:
	$(CARGO) build --release

## Run all CI checks: formatting, linting, and tests
ci: fmt-check lint-check test

## Run the binary with arguments (pass with ARGS="xinfo --help")
run: build
	./target/release/$(BINARY_NAME) $(ARGS)

## Run unit and integration tests
test:
	$(CARGO) test -- --test-threads=1

## Run test coverage analysis using tarpaulin
coverage:
	$(CARGO) tarpaulin --workspace --timeout 120 --out Stdout --jobs 1

## Format all source files
fmt:
	$(CARGO) fmt --all && $(CARGO) clippy --fix --allow-dirty

## Check formatting without modifying files
fmt-check:
	$(CARGO) fmt --all -- --check

## Lint the code using Clippy
lint:
	$(CARGO) clippy --all-targets -- -D warnings

## Check for linting issues without modifying code
lint-check:
	$(CARGO) clippy --all-targets --all-features -- -D warnings

## Install the binary to ~/.cargo/bin (for git plugin use)
install: build
	$(CARGO) install --path .

## Uninstall the binary
uninstall:
	$(CARGO) uninstall $(BINARY_NAME)

## Clean all build artifacts
clean:
	$(CARGO) clean

## Analyze binary size and dependencies
analyze-size: build
	@echo "=== Performance-Optimized Binary Analysis ==="
	@ls -lh target/release/$(BINARY_NAME)
	@echo ""
	@echo "=== Crate Size Breakdown ==="
	$(CARGO) bloat --release --crates
	@echo ""
	@echo "=== Top Functions ==="
	$(CARGO) bloat --release -n 10

## Publish optimized binary to crates.io
publish-optimized: build test
	$(CARGO) publish

## Publish to crates.io (standard)
publish:
	$(CARGO) publish

## Show this help message
help:
	@echo ""
	@echo "git-x Makefile — available targets:"
	@echo "  make              Build and run (default)"
	@echo "  make build        Build release binary"
	@echo "  make run          Run binary with ARGS=\"info\""
	@echo "  make test         Run tests"
	@echo "  make coverage     Generate test coverage report"
	@echo "  make fmt          Format code"
	@echo "  make fmt-check    Check formatting"
	@echo "  make lint         Lint with Clippy"
	@echo "  make lint-check   Check for linting issues"
	@echo "  make install      Install to ~/.cargo/bin as 'git-x'"
	@echo "  make uninstall    Uninstall binary"
	@echo "  make clean        Remove build artifacts"
	@echo "  make ci           Run CI checks (formatting, linting, tests)"
	@echo ""
	@echo "Optimization Targets:"
	@echo "  make analyze-size      Analyze binary size and dependencies"
	@echo "  make publish-optimized Publish optimized binary to crates.io"
	@echo "  make publish           Publish to crates.io (standard)"
	@echo ""
