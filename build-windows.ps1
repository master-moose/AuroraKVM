# AuroraKVM Windows Build Script (PowerShell)
# Builds server and client binaries on Windows

Write-Host "AuroraKVM Windows Build Script" -ForegroundColor Cyan
Write-Host "==============================" -ForegroundColor Cyan
Write-Host ""

Write-Host "Building release binaries..." -ForegroundColor Yellow
Write-Host ""

# Build server
Write-Host "Building aurora_server.exe..." -ForegroundColor Green
cargo build --release --bin aurora_server
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: Server build failed!" -ForegroundColor Red
    exit 1
}

# Build client
Write-Host "Building aurora_client.exe..." -ForegroundColor Green
cargo build --release --bin aurora_client
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: Client build failed!" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "Build complete!" -ForegroundColor Green
Write-Host ""
Write-Host "Binaries located at:"
Write-Host "  Server: .\target\release\aurora_server.exe"
Write-Host "  Client: .\target\release\aurora_client.exe"
Write-Host ""
