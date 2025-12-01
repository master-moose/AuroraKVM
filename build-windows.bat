@echo off
REM AuroraKVM Windows Build Script
REM Builds server and client binaries on Windows

echo AuroraKVM Windows Build Script
echo ==============================
echo.

echo Building release binaries...
echo.

echo Building aurora_server.exe...
cargo build --release --bin aurora_server
if errorlevel 1 (
    echo ERROR: Server build failed!
    exit /b 1
)

echo Building aurora_client.exe...
cargo build --release --bin aurora_client
if errorlevel 1 (
    echo ERROR: Client build failed!
    exit /b 1
)

echo.
echo Build complete!
echo.
echo Binaries located at:
echo   Server: .\target\release\aurora_server.exe
echo   Client: .\target\release\aurora_client.exe
echo.
pause
