@echo off
chcp 65001 >nul
title 资产管理器
echo ========================================
echo   资产管理器 - Asset Manager
echo ========================================
echo.

:: 确保 Cargo 和 Node 在 PATH 中
set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
set "PATH=%ProgramFiles%\nodejs;%PATH%"

:: 检查 cargo
where cargo >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo [错误] 未找到 cargo，请先安装 Rust: https://rustup.rs
    pause
    exit /b 1
)

:: 检查 node
where node >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo [错误] 未找到 Node.js，请先安装: https://nodejs.org
    pause
    exit /b 1
)

:: 检查 tauri-cli
cargo tauri --version >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo [提示] 正在安装 Tauri CLI...
    cargo install tauri-cli
)

:: 检查前端依赖
if not exist "ui\node_modules" (
    echo [提示] 正在安装前端依赖...
    cd ui && npm install && cd ..
)

echo.
echo [启动] 正在编译并启动应用...
echo.
cargo tauri dev
pause
