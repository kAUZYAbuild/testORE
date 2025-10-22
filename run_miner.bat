@echo off
REM TestORE Windows Miner Launcher
REM Automatically runs the miner in forever mode

echo.
echo ===============================
echo   TestORE Miner - Windows
echo ===============================
echo.

REM Check if binary exists
if not exist "target\release\testore.exe" (
    echo [ERROR] testore.exe not found!
    echo.
    echo Please run first:
    echo   cargo build --release
    echo.
    pause
    exit /b 1
)

echo Starting miner...
echo.
echo Configuration:
echo - Network: Solana Testnet ONLY
echo - Mode: Forever (auto-restart on errors)
echo - Threads: Auto-detect (all CPU cores)
echo - Difficulty: 8 bits (testnet default)
echo.
echo Press Ctrl+C to stop mining
echo.
echo ===============================
echo.

REM Run miner in forever mode
target\release\testore.exe mine --forever

pause
