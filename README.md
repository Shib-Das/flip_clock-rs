# flip_clock-rs

## Build Prerequisites

### Linux (Debian/Ubuntu)
You need to install the SDL2 development libraries:
```bash
sudo apt-get update && sudo apt-get install -y libsdl2-dev libsdl2-ttf-dev libsdl2-gfx-dev
```

### Windows (Native)
Since this project uses SDL2, the easiest way to build for Windows is to use the Rust toolchain on Windows directly.

1. **Install Rust**: Download from [rustup.rs](https://rustup.rs).
2. **Install SDL2**:
   - The easiest method is using `vcpkg`:
     ```powershell
     vcpkg install sdl2:x64-windows sdl2-ttf:x64-windows sdl2-gfx:x64-windows
     ```
   - Alternatively, download the VC development libraries from [libsdl.org](https://www.libsdl.org/) and set the `SDL2_DIR` environment variable.
3. **Build**:
   ```powershell
   ./build.ps1
   ```
   Or manually:
   ```powershell
   cargo build --release
   ```