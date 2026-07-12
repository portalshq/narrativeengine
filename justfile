# Justfile for NAP project commands
# Install: cargo install just
# Run: just <command>

# Default recipe - show available commands
default:
    @just --list

# =============================================================================
# Build Commands
# =============================================================================

build-all:
    @./scripts/build-all.sh

build-rust:
    @./scripts/build-rust.sh

build-python:
    @./scripts/build-python.sh

build-typescript:
    @./scripts/build-typescript.sh

# =============================================================================
# Test Commands
# =============================================================================

# Run all tests (unit + integration parity)
test-all:
    @./scripts/test-all.sh

# Run parity tests across Rust, Python, TypeScript
test-parity:
    @./scripts/test-parity.sh

# Run local lore server integration tests
test-integration-local:
    @./scripts/test-integration-local.sh

# Run Portals Cloud integration tests
test-integration-cloud:
    @./scripts/test-integration-cloud.sh

# =============================================================================
# Integration Test Commands (Direct Cargo)
# =============================================================================

# Run local integration tests directly via cargo
test-local:
    cargo test -p nap-cli --test local_lore_suite --features lore-e2e -- --test-threads=1

# Run cloud integration tests directly via cargo
test-cloud:
    cargo test -p nap-cli --test cloud_lore_suite --features lore-e2e -- --test-threads=1

# =============================================================================
# Publish Commands
# =============================================================================

publish-all:
    @./scripts/publish-all.sh

pre-publish-check:
    @./scripts/pre-publish-check.mjs

# =============================================================================
# Install Commands
# =============================================================================

install:
    @./scripts/install.sh

# =============================================================================
# Development Commands
# =============================================================================

# Generate types across all languages
generate-types:
    @./scripts/generate-types.sh

# Watch for changes and rebuild (requires cargo-watch)
watch:
    cargo watch -x build -x test

# Format code
fmt:
    cargo fmt
    @./typescript/narrativeengine/npm run format
    @./typescript/nap-sdk/npm run format

# Lint code
lint:
    cargo clippy -- -D warnings
    @./typescript/narrativeengine/npm run lint
    @./typescript/nap-sdk/npm run lint

# Clean build artifacts
clean:
    cargo clean
    rm -rf node_modules/*/node_modules
    rm -rf python/narrativeengine/build
    rm -rf python/nap-sdk/build

# =============================================================================
# Documentation
# =============================================================================

# Open documentation in browser
docs:
    cargo doc --open

# =============================================================================
# Quick Start Examples
# =============================================================================

# Example: Initialize NAP with local provider
init-local:
    cargo run -p nap-cli -- init --provider local

# Example: Initialize NAP with cloud provider
init-cloud:
    cargo run -p nap-cli -- init --provider portals-cloud

# Example: Create a test universe
create-universe universe:
    cargo run -p nap-cli -- init {{universe}}

# Example: Create a character entity
create-character universe id name:
    cargo run -p nap-cli -- create --universe {{universe}} character {{id}} --name {{name}}
