#!/bin/bash
set -e

echo "=========================================="
echo "Flip Clock Build Helper (MacOS)"
echo "=========================================="

# Check for Cargo
if ! command -v cargo &> /dev/null; then
    echo "[ERROR] Rust (cargo) is not installed. Please install it from https://rustup.rs/"
    exit 1
fi

# Check for Docker
if ! command -v docker &> /dev/null; then
    echo "[WARNING] Docker is not found. Cross-compilation requires Docker."
    echo "          Please install Docker Desktop for Mac."
fi

# Check for Cross
if ! command -v cross &> /dev/null; then
    echo "[INFO] 'cross' tool not found. Installing via cargo..."
    cargo install cross
fi

PS3='Please enter your choice: '
options=("Run Locally (Mac)" "Build for Mac (Release)" "Build for Windows (Release)" "Build for Linux (Release)" "Exit")
select opt in "${options[@]}"
do
    case $opt in
        "Run Locally (Mac)")
            echo "Running locally..."
            cargo run
            break
            ;;
        "Build for Mac (Release)")
            echo "Building for MacOS (Native)..."
            cargo build --target x86_64-apple-darwin --release
            echo "[SUCCESS] Artifact at: target/x86_64-apple-darwin/release/rust_flip-rs"
            break
            ;;
        "Build for Windows (Release)")
            echo "Building for Windows..."
            cross build --target x86_64-pc-windows-gnu --release
            echo "[SUCCESS] Artifact at: target/x86_64-pc-windows-gnu/release/rust_flip-rs.exe"
            break
            ;;
        "Build for Linux (Release)")
            echo "Building for Linux..."
            cross build --target x86_64-unknown-linux-gnu --release
            echo "[SUCCESS] Artifact at: target/x86_64-unknown-linux-gnu/release/rust_flip-rs"
            break
            ;;
        "Exit")
            break
            ;;
        *) echo "invalid option $REPLY";;
    esac
done
