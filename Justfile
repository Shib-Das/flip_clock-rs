# Justfile for flip_clock-rs

# Default recipe
default:
    @just --list

# Run locally
run:
    cargo run

# Build for Linux (using cross)
build-linux:
    cross build --target x86_64-unknown-linux-gnu --release

# Build for Windows (using cross)
build-windows:
    cross build --target x86_64-pc-windows-gnu --release

# Build for MacOS (native only - run this ON MacOS)
build-mac:
    cargo build --target x86_64-apple-darwin --release

# Clean project
clean:
    cargo clean
