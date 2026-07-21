export RUSTC_WRAPPER := "sccache"
set shell := ["bash", "-cu"]

# Default recipe
default:
    @just --list

alias c := check
# Run all checks (fmt, clippy, docs, test)
check: fmt clippy docs test

alias f := fmt
# Format code
fmt:
    cargo fmt --all

# Check formatting without modifying
fmt-check:
    cargo fmt --all -- --check

# Run clippy
clippy:
    cargo clippy --all-targets --all-features -- -D warnings

alias d := docs
# Build documentation
docs:
    RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features

alias t := test
# Run tests with nextest (skip if no tests exist)
test:
    cargo nextest run --all-features --no-tests auto || true
    cargo test --lib --bins

alias b := build
# Build release binaries
build:
    cargo build --release

# Clean build artifacts
clean:
    cargo clean

# Install dev dependencies
setup:
    cargo install cargo-nextest
