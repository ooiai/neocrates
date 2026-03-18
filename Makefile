# ─────────────────────────────────────────────
#  Neocrates — Makefile
#  Copyright © 2026 ooiai. MIT License.
# ─────────────────────────────────────────────

GIT   := git
CARGO := cargo

# Commit message (override: make <target> m="your message")
m ?= chore: update

# ─── Helpers ─────────────────────────────────

define git_commit_if_needed
	@if [ -n "$$($(GIT) status --porcelain)" ]; then \
		$(GIT) add .; \
		$(GIT) commit -m "$(m)"; \
	else \
		echo "Nothing to commit"; \
	fi
endef

define git_push_if_needed
	@if [ -n "$$($(GIT) status --porcelain)" ]; then \
		$(GIT) add .; \
		$(GIT) commit -m "$(m)"; \
		$(GIT) push; \
	else \
		echo "Nothing to commit"; \
	fi
endef

# ─── Default ─────────────────────────────────

.DEFAULT_GOAL := help

.PHONY: help build build-full build-release check test lint fmt fmt-check doc clean \
        dry-run publish git-commit git-push audit

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) \
		| awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-18s\033[0m %s\n", $$1, $$2}'

# ─── Build ───────────────────────────────────

build: ## Build (default features)
	@echo "===> cargo build"
	$(CARGO) build -p neocrates

build-full: ## Build with all features enabled
	@echo "===> cargo build --features full"
	$(CARGO) build -p neocrates --features full

build-release: ## Release build (default features)
	@echo "===> cargo build --release"
	$(CARGO) build --release -p neocrates

check: ## Fast type-check without codegen
	@echo "===> cargo check"
	$(CARGO) check -p neocrates --features full

# ─── Test ────────────────────────────────────

test: ## Run all tests
	@echo "===> cargo test"
	$(CARGO) test -p neocrates

test-full: ## Run tests with all features
	@echo "===> cargo test --features full"
	$(CARGO) test -p neocrates --features full

# ─── Lint & Format ───────────────────────────

lint: ## Run Clippy (zero-warning policy)
	@echo "===> cargo clippy"
	$(CARGO) clippy -p neocrates --features full -- -D warnings

fmt: ## Auto-format source code
	@echo "===> cargo fmt"
	$(CARGO) fmt

fmt-check: ## Check formatting without modifying files
	@echo "===> cargo fmt --check"
	$(CARGO) fmt --check

# ─── Docs ────────────────────────────────────

doc: ## Generate and open documentation
	@echo "===> cargo doc"
	$(CARGO) doc -p neocrates --features full --no-deps --open

doc-check: ## Check documentation without opening
	@echo "===> cargo doc (no open)"
	$(CARGO) doc -p neocrates --features full --no-deps

# ─── Security ────────────────────────────────

audit: ## Run cargo-audit for known vulnerabilities
	@echo "===> cargo audit"
	$(CARGO) audit

# ─── Maintenance ─────────────────────────────

clean: ## Remove build artifacts
	@echo "===> cargo clean"
	$(CARGO) clean

# ─── Publish ─────────────────────────────────

dry-run: ## Dry-run publish to crates.io
	@echo "===> dry-run publish"
	$(call git_commit_if_needed)
	$(CARGO) publish -p neocrates --dry-run --registry crates-io

publish: ## Publish to crates.io (requires cargo login)
	@echo "===> publish neocrates"
	$(call git_commit_if_needed)
	$(CARGO) publish -p neocrates --registry crates-io
	$(CARGO) clean

# ─── Git ─────────────────────────────────────

git-commit: ## Stage, commit (requires m="message")
	$(call git_commit_if_needed)

git-run: ## Stage, commit, and push (requires m="message")
	$(call git_push_if_needed)
