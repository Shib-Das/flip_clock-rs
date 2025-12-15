@echo off
setlocal

echo ==========================================
echo Flip Clock Build Helper (Windows)
echo ==========================================

WHERE cargo >nul 2>nul
IF %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Rust (cargo) is not installed. Please install it from https://rustup.rs/
    pause
    exit /b 1
)

WHERE docker >nul 2>nul
IF %ERRORLEVEL% NEQ 0 (
    echo [WARNING] Docker is not found. Cross-compilation to Linux/Windows might fail if you don't use 'cross'.
    echo           Please install Docker Desktop if you intend to cross-compile.
    echo.
)

REM Check for 'cross'
WHERE cross >nul 2>nul
IF %ERRORLEVEL% NEQ 0 (
    echo [INFO] 'cross' tool not found. Installing via cargo...
    cargo install cross
    IF %ERRORLEVEL% NEQ 0 (
        echo [ERROR] Failed to install 'cross'.
        pause
        exit /b 1
    )
)

:MENU
echo.
echo Select target:
echo 1. Run Locally (Windows)
echo 2. Build for Windows (Release)
echo 3. Build for Linux (Release - requires Docker)
echo 4. Exit
echo.

set /p choice="Enter choice (1-4): "

if "%choice%"=="1" (
    echo Running locally...
    cargo run
    goto MENU
)

if "%choice%"=="2" (
    echo Building for Windows...
    cross build --target x86_64-pc-windows-gnu --release
    if %ERRORLEVEL% EQU 0 (
        echo [SUCCESS] Artifact at: target\x86_64-pc-windows-gnu\release\rust_flip-rs.exe
    )
    goto MENU
)

if "%choice%"=="3" (
    echo Building for Linux...
    cross build --target x86_64-unknown-linux-gnu --release
    if %ERRORLEVEL% EQU 0 (
        echo [SUCCESS] Artifact at: target\x86_64-unknown-linux-gnu\release\rust_flip-rs
    )
    goto MENU
)

if "%choice%"=="4" (
    exit /b 0
)

echo Invalid choice.
goto MENU
