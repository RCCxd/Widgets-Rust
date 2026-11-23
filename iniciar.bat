@echo off
cd /d "%~dp0src-tauri"
cargo run --release
pause
