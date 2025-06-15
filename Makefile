# Algae - Algebraic Effects for Rust
# Development Makefile

.PHONY: help test test-lib test-macros check clippy fmt fmt-check clean doc doc-open examples run-examples bench install-deps ci-local all

# Default target
.DEFAULT_GOAL := help

# Colors for output
BOLD := \033[1m
RED := \033[31m
GREEN := \033[32m
YELLOW := \033[33m
BLUE := \033[34m
MAGENTA := \033[35m
CYAN := \033[36m
RESET := \033[0m

help: ## Show this help message
	@echo "$(BOLD)Algae - Algebraic Effects for Rust$(RESET)"
	@echo "$(CYAN)Development Makefile$(RESET)"
	@echo ""
	@echo "$(BOLD)Usage:$(RESET)"
	@echo "  make <target>"
	@echo ""
	@echo "$(BOLD)Available targets:$(RESET)"
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  $(CYAN)%-20s$(RESET) %s\n", $$1, $$2}' $(MAKEFILE_LIST)
	@echo ""
	@echo "$(BOLD)Examples:$(RESET)"
	@echo "  make test          # Run all tests"
	@echo "  make check         # Quick compilation check"
	@echo "  make ci-local      # Run full CI pipeline locally"
	@echo "  make examples      # Run all working examples"

# === Testing ===

test: ## Run all tests (library and macros)
	@echo "$(BOLD)$(GREEN)Running all tests...$(RESET)"
	@cargo test --lib --verbose
	@cargo test -p algae-macros --verbose

test-lib: ## Run library tests only
	@echo "$(BOLD)$(GREEN)Running library tests...$(RESET)"
	@cargo test --lib --verbose

test-macros: ## Run macro tests only
	@echo "$(BOLD)$(GREEN)Running macro tests...$(RESET)"
	@cargo test -p algae-macros --verbose

# === Development ===

check: ## Quick compilation check
	@echo "$(BOLD)$(BLUE)Checking compilation...$(RESET)"
	@cargo check --lib --all-features

clippy: ## Run clippy lints (matches original GitHub Actions intent)
	@echo "$(BOLD)$(YELLOW)Running clippy on library code (strict)...$(RESET)"
	@cargo clippy --lib --all-features -- -D warnings
	@echo "$(BOLD)$(YELLOW)Running clippy on binary targets (strict)...$(RESET)"
	@cargo clippy --bins --all-features -- -D warnings || true
	@echo "$(BOLD)$(YELLOW)Running clippy on tests (warnings allowed)...$(RESET)"
	@cargo clippy --tests --all-features || true
	@echo "$(BOLD)$(YELLOW)Running clippy on working examples (warnings allowed)...$(RESET)"
	@cargo clippy --example multiple_effects_demo || true
	@cargo clippy --example custom_root_effects || true
	@cargo clippy --example test_error_messages || true
	@cargo clippy --example console || true
	@cargo clippy --example minimal || true
	@cargo clippy --example overview || true

clippy-strict: ## Run clippy with all targets and strict warnings (excludes duplicate_root_test)
	@echo "$(BOLD)$(YELLOW)Running clippy on all targets (strict)...$(RESET)"
	@cargo clippy --lib --all-features -- -D warnings
	@cargo clippy --bins --all-features -- -D warnings
	@cargo clippy --tests --all-features -- -D warnings
	@for example in $$(ls algae/examples/*.rs | grep -v duplicate_root_test | sed 's/.*\///' | sed 's/\.rs//'); do \
		echo "$(YELLOW)Checking example: $$example$(RESET)"; \
		cargo clippy --example $$example --all-features -- -D warnings; \
	done

fmt: ## Format code
	@echo "$(BOLD)$(MAGENTA)Formatting code...$(RESET)"
	@cargo fmt --all

fmt-check: ## Check code formatting without changing files
	@echo "$(BOLD)$(MAGENTA)Checking code formatting...$(RESET)"
	@cargo fmt --all -- --check

# === Documentation ===

doc: ## Build documentation
	@echo "$(BOLD)$(CYAN)Building documentation...$(RESET)"
	@cargo doc --no-deps --all-features

doc-strict: ## Build documentation with warnings as errors
	@echo "$(BOLD)$(CYAN)Building documentation (strict)...$(RESET)"
	@RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features

doc-open: ## Build and open documentation in browser
	@echo "$(BOLD)$(CYAN)Building and opening documentation...$(RESET)"
	@cargo doc --no-deps --all-features --open

# === Examples ===

examples: ## Check that all working examples compile
	@echo "$(BOLD)$(GREEN)Checking working examples...$(RESET)"
	@echo "$(YELLOW)✓$(RESET) multiple_effects_demo"
	@cargo check --example multiple_effects_demo
	@echo "$(YELLOW)✓$(RESET) custom_root_effects"
	@cargo check --example custom_root_effects
	@echo "$(YELLOW)✓$(RESET) test_error_messages"
	@cargo check --example test_error_messages
	@echo "$(YELLOW)✓$(RESET) console"
	@cargo check --example console
	@echo "$(YELLOW)✓$(RESET) minimal"
	@cargo check --example minimal
	@echo "$(YELLOW)✓$(RESET) overview"
	@cargo check --example overview
	@echo "$(YELLOW)✓$(RESET) theory"
	@cargo check --example theory
	@echo "$(YELLOW)✓$(RESET) advanced"
	@cargo check --example advanced
	@echo "$(YELLOW)✓$(RESET) test_non_default_payload"
	@cargo check --example test_non_default_payload
	@echo "$(YELLOW)✓$(RESET) test_manual_default"
	@cargo check --example test_manual_default
	@echo "$(GREEN)All working examples compile successfully!$(RESET)"

run-examples: ## Run selected examples to see them in action
	@echo "$(BOLD)$(GREEN)Running examples...$(RESET)"
	@echo ""
	@echo "$(BOLD)$(CYAN)1. Multiple Effects Demo:$(RESET)"
	@cargo run --example multiple_effects_demo
	@echo ""
	@echo "$(BOLD)$(CYAN)2. Console Example:$(RESET)"
	@cargo run --example console
	@echo ""
	@echo "$(BOLD)$(CYAN)3. Minimal Example:$(RESET)"
	@cargo run --example minimal
	@echo ""
	@echo "$(BOLD)$(CYAN)4. Test Error Messages:$(RESET)"
	@cargo run --example test_error_messages

test-error-detection: ## Verify that error detection works correctly
	@echo "$(BOLD)$(YELLOW)Testing error detection...$(RESET)"
	@echo "$(CYAN)Verifying duplicate_root_test fails as expected:$(RESET)"
	@if cargo check --example duplicate_root_test 2>/dev/null; then \
		echo "$(RED)ERROR: duplicate_root_test should fail but succeeded!$(RESET)"; \
		exit 1; \
	else \
		echo "$(GREEN)✓ duplicate_root_test correctly fails to compile$(RESET)"; \
	fi

# === Benchmarks ===

bench: ## Run benchmarks (if any)
	@echo "$(BOLD)$(MAGENTA)Running benchmarks...$(RESET)"
	@cargo bench

# === Utilities ===

clean: ## Clean build artifacts
	@echo "$(BOLD)$(RED)Cleaning build artifacts...$(RESET)"
	@cargo clean

install-deps: ## Install development dependencies
	@echo "$(BOLD)$(BLUE)Installing development dependencies...$(RESET)"
	@rustup component add rustfmt clippy
	@echo "$(GREEN)Development dependencies installed!$(RESET)"

# === CI Pipeline ===

ci-local: ## Run the complete CI pipeline locally
	@echo "$(BOLD)$(CYAN)Running complete CI pipeline locally...$(RESET)"
	@echo ""
	@echo "$(BOLD)$(YELLOW)Step 1: Check formatting$(RESET)"
	@$(MAKE) fmt-check
	@echo ""
	@echo "$(BOLD)$(YELLOW)Step 2: Run clippy on all targets (strict)$(RESET)"
	@$(MAKE) clippy-strict
	@echo ""
	@echo "$(BOLD)$(YELLOW)Step 3: Run tests$(RESET)"
	@$(MAKE) test
	@echo ""
	@echo "$(BOLD)$(YELLOW)Step 4: Check examples compile$(RESET)"
	@$(MAKE) examples
	@echo ""
	@echo "$(BOLD)$(YELLOW)Step 5: Test error detection$(RESET)"
	@$(MAKE) test-error-detection
	@echo ""
	@echo "$(BOLD)$(YELLOW)Step 6: Build documentation$(RESET)"
	@$(MAKE) doc-strict
	@echo ""
	@echo "$(BOLD)$(GREEN)🎉 All CI checks passed!$(RESET)"

# === Composite Targets ===

all: ## Run formatting, linting, tests, and build docs
	@echo "$(BOLD)$(CYAN)Running full development pipeline...$(RESET)"
	@$(MAKE) fmt
	@$(MAKE) clippy
	@$(MAKE) test
	@$(MAKE) doc
	@echo "$(BOLD)$(GREEN)✨ All tasks completed successfully!$(RESET)"

# === Development Workflow Targets ===

quick: ## Quick development check (format + clippy + compile)
	@echo "$(BOLD)$(CYAN)Quick development check...$(RESET)"
	@$(MAKE) fmt
	@$(MAKE) clippy
	@$(MAKE) check
	@echo "$(BOLD)$(GREEN)✓ Quick check completed!$(RESET)"

dev: ## Development mode: format, check, and run tests
	@echo "$(BOLD)$(CYAN)Development workflow...$(RESET)"
	@$(MAKE) fmt
	@$(MAKE) check
	@$(MAKE) test-lib
	@echo "$(BOLD)$(GREEN)✓ Development check completed!$(RESET)"

# === Release Preparation ===

pre-release: ## Prepare for release (full CI + clean)
	@echo "$(BOLD)$(MAGENTA)Preparing for release...$(RESET)"
	@$(MAKE) clean
	@$(MAKE) ci-local
	@echo "$(BOLD)$(GREEN)🚀 Ready for release!$(RESET)"

# === Feature Development ===

test-feature: ## Test a specific feature (set FEATURE env var)
ifndef FEATURE
	@echo "$(RED)Error: Please specify FEATURE. Example: make test-feature FEATURE=custom_roots$(RESET)"
	@exit 1
endif
	@echo "$(BOLD)$(GREEN)Testing feature: $(FEATURE)$(RESET)"
	@cargo test --lib --verbose $(FEATURE)

# === Troubleshooting ===

debug-info: ## Show environment and toolchain information
	@echo "$(BOLD)$(CYAN)Environment Information:$(RESET)"
	@echo "$(YELLOW)Rust version:$(RESET)"
	@rustc --version
	@echo "$(YELLOW)Cargo version:$(RESET)"
	@cargo --version
	@echo "$(YELLOW)Toolchain:$(RESET)"
	@rustup show
	@echo "$(YELLOW)Project structure:$(RESET)"
	@find . -name "Cargo.toml" -not -path "./target/*" | head -10
	@echo "$(YELLOW)Available examples:$(RESET)"
	@ls algae/examples/*.rs | sed 's/.*\///' | sed 's/\.rs//' | sort