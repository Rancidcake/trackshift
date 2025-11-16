# Setup ARM64 Build Environment for Rust
# This script configures environment variables for ARM64 compilation

Write-Host "Setting up ARM64 build environment..." -ForegroundColor Cyan

# Find latest Windows SDK
$sdkVersions = Get-ChildItem "C:\Program Files (x86)\Windows Kits\10\Lib" -Directory | 
    Where-Object { $_.Name -match '^10\.0\.\d+\.\d+$' } | 
    Sort-Object { [Version]$_.Name } -Descending | 
    Select-Object -First 1

if (-not $sdkVersions) {
    Write-Host "ERROR: Could not find Windows SDK" -ForegroundColor Red
    exit 1
}

$sdkVersion = $sdkVersions.Name
$sdkBase = "C:\Program Files (x86)\Windows Kits\10"
Write-Host "Using Windows SDK version: $sdkVersion" -ForegroundColor Green

# Find latest MSVC version
$msvcVersions = Get-ChildItem "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC" -Directory -ErrorAction SilentlyContinue | 
    Sort-Object Name -Descending | 
    Select-Object -First 1

if (-not $msvcVersions) {
    Write-Host "ERROR: Could not find MSVC tools" -ForegroundColor Red
    exit 1
}

$msvcPath = $msvcVersions.FullName
Write-Host "Using MSVC version: $($msvcVersions.Name)" -ForegroundColor Green

# Check for ARM64 libraries
$arm64UmLib = Join-Path $sdkBase "Lib\$sdkVersion\um\arm64"
$arm64UcrtLib = Join-Path $sdkBase "Lib\$sdkVersion\ucrt\arm64"
$msvcArm64Lib = Join-Path $msvcPath "lib\arm64"

if (-not (Test-Path $arm64UmLib)) {
    Write-Host "ERROR: ARM64 UM libraries not found at: $arm64UmLib" -ForegroundColor Red
    exit 1
}

if (-not (Test-Path $msvcArm64Lib)) {
    Write-Host "WARNING: MSVC ARM64 libraries not found at: $msvcArm64Lib" -ForegroundColor Yellow
    Write-Host "This might cause linking issues with msvcrt.lib" -ForegroundColor Yellow
}

# Set up environment variables
$env:LIB = "$msvcArm64Lib;$arm64UmLib;$arm64UcrtLib;$env:LIB"
$env:INCLUDE = "$msvcPath\include;$sdkBase\Include\$sdkVersion\um;$sdkBase\Include\$sdkVersion\ucrt;$sdkBase\Include\$sdkVersion\shared;$env:INCLUDE"
$env:PATH = "$msvcPath\bin\Hostx64\arm64;$msvcPath\bin\Hostx64\x64;$env:PATH"

Write-Host ""
Write-Host "Environment configured:" -ForegroundColor Green
Write-Host "  LIB = $env:LIB" -ForegroundColor Gray
Write-Host "  INCLUDE = $env:INCLUDE" -ForegroundColor Gray
Write-Host ""
Write-Host "You can now run: cargo build" -ForegroundColor Cyan
