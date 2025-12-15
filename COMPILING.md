# Compiling Flip Clock

This project uses `cross` for cross-platform compilation and `just` for command management.

## Prerequisites

1.  **Rust**: [Install Rust](https://rustup.rs/)
2.  **Cross**: `cargo install cross`
3.  **Just** (Optional but recommended): `cargo install just`
4.  **Docker**: Required for `cross` to work (Desktop or Engine).

## commands

If you have `just` installed:

-   **Run locally**: `just run`
-   **Build for Linux**: `just build-linux` (Produces executable in `target/x86_64-unknown-linux-gnu/release/`)
-   **Build for Windows**: `just build-windows` (Produces executable in `target/x86_64-pc-windows-gnu/release/`)
-   **Build for MacOS**: `just build-mac` (Must be run **on** MacOS)

Without `just`, run the commands manually:

-   **Linux**: `cross build --target x86_64-unknown-linux-gnu --release`
-   **Windows**: `cross build --target x86_64-pc-windows-gnu --release`

## Cross-Compilation Notes

-   **Dependencies**: The project now uses `macroquad`.
    -   **Linux**: `cross` will install `libxi-dev`, `libgl1-mesa-dev`, `libasound2-dev`.
    -   **Windows**: Generally works out of the box with standard MinGW dynamic linking.

## Local Development (Windows)

If you are developing locally on Windows, ensure you have SDL2 development libraries installed (e.g., via `vcpkg` or by placing `SDL2.dll` in your path).
