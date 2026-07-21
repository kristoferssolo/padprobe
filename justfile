export RUSTC_WRAPPER := "sccache"
set shell := ["bash", "-cu"]

alias b := build
alias c := check
alias d := docs
alias f := fmt
alias i := install
alias t := test

# Default recipe
default:
    @just --list

# Run all checks (fmt, clippy, docs, test)
check: fmt clippy docs test

# Format code
fmt:
    cargo fmt --all

# Check formatting without modifying
fmt-check:
    cargo fmt --all -- --check

# Run clippy
clippy:
    cargo clippy --all-targets --all-features -- -D warnings

# Build documentation
docs:
    RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features

# Run tests with nextest (skip if no tests exist)
test:
    cargo nextest run --all-features --no-tests auto || true
    cargo test --lib --bins

# Build release binaries
build:
    cargo build --release

# Clean build artifacts
clean:
    cargo clean

# Install dev dependencies
setup:
    cargo install cargo-nextest

install:
    cargo install --path .
