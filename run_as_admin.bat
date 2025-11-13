@echo off
:: Проверка прав администратора
net session >nul 2>&1
if %errorLevel% == 0 (
    echo Running with administrator privileges...
    cargo run --release
) else (
    echo Requesting administrator privileges...
    powershell -Command "Start-Process cmd -ArgumentList '/c cd /d %CD% && cargo run --release && pause' -Verb RunAs"
)
