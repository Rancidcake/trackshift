# ARM64 Build Environment Diagnostic Script

Write-Host "=== ARM64 Build Environment Diagnostic ===" -ForegroundColor Cyan
Write-Host ""

# Check MSVC ARM64 runtime libraries
Write-Host "1. Checking MSVC ARM64 Runtime Libraries..." -ForegroundColor Yellow
$msvcBase = "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC"
if (Test-Path $msvcBase) {
    $versions = Get-ChildItem $msvcBase -Directory | Sort-Object Name -Descending
    $found = $false
    foreach ($ver in $versions) {
        $arm64Lib = Join-Path $ver.FullName "lib\arm64"
        if (Test-Path $arm64Lib) {
            Write-Host "  [FOUND] Version $($ver.Name): $arm64Lib" -ForegroundColor Green
            $msvcrt = Join-Path $arm64Lib "msvcrt.lib"
            if (Test-Path $msvcrt) {
                Write-Host "    [OK] msvcrt.lib exists" -ForegroundColor Green
                $found = $true
            } else {
                Write-Host "    [MISSING] msvcrt.lib" -ForegroundColor Red
            }
        }
    }
    if (-not $found) {
        Write-Host "  [NOT FOUND] No MSVC ARM64 runtime libraries found" -ForegroundColor Red
        Write-Host "  Expected location: $msvcBase\<version>\lib\arm64\msvcrt.lib" -ForegroundColor Gray
    }
} else {
    Write-Host "  [ERROR] MSVC base directory not found" -ForegroundColor Red
}

Write-Host ""

# Check Windows SDK ARM64 libraries
Write-Host "2. Checking Windows SDK ARM64 Libraries..." -ForegroundColor Yellow
$sdkBase = "C:\Program Files (x86)\Windows Kits\10\Lib"
if (Test-Path $sdkBase) {
    $sdkVersions = Get-ChildItem $sdkBase -Directory | Sort-Object Name -Descending
    $found = $false
    foreach ($ver in $sdkVersions) {
        $arm64Lib = Join-Path $ver.FullName "arm64"
        if (Test-Path $arm64Lib) {
            Write-Host "  [FOUND] SDK Version $($ver.Name): $arm64Lib" -ForegroundColor Green
            $kernel32 = Join-Path $arm64Lib "kernel32.lib"
            if (Test-Path $kernel32) {
                Write-Host "    [OK] kernel32.lib exists" -ForegroundColor Green
                $found = $true
            } else {
                Write-Host "    [MISSING] kernel32.lib" -ForegroundColor Red
            }
        }
    }
    if (-not $found) {
        Write-Host "  [NOT FOUND] No Windows SDK ARM64 libraries found" -ForegroundColor Red
        Write-Host "  Expected location: $sdkBase\<version>\arm64\kernel32.lib" -ForegroundColor Gray
    }
} else {
    Write-Host "  [ERROR] Windows SDK base directory not found" -ForegroundColor Red
}

Write-Host ""

# Check ARM64 compiler tools
Write-Host "3. Checking ARM64 Compiler Tools..." -ForegroundColor Yellow
$compilerBase = "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC"
if (Test-Path $compilerBase) {
    $versions = Get-ChildItem $compilerBase -Directory | Sort-Object Name -Descending
    $found = $false
    foreach ($ver in $versions) {
        $arm64Bin = Join-Path $ver.FullName "bin\Hostarm64\arm64"
        if (Test-Path $arm64Bin) {
            Write-Host "  [FOUND] Version $($ver.Name): ARM64 compiler tools" -ForegroundColor Green
            $cl = Join-Path $arm64Bin "cl.exe"
            if (Test-Path $cl) {
                Write-Host "    [OK] cl.exe exists" -ForegroundColor Green
                $found = $true
            }
        }
    }
    if (-not $found) {
        Write-Host "  [NOT FOUND] No ARM64 compiler tools found" -ForegroundColor Red
    }
} else {
    Write-Host "  [ERROR] Compiler base directory not found" -ForegroundColor Red
}

Write-Host ""
Write-Host "=== Summary ===" -ForegroundColor Cyan
Write-Host ""
Write-Host "To fix missing ARM64 libraries:" -ForegroundColor Yellow
Write-Host "1. Open Visual Studio Installer" -ForegroundColor White
Write-Host "2. Modify Build Tools 2022" -ForegroundColor White
Write-Host "3. Go to 'Individual components' tab" -ForegroundColor White
Write-Host "4. Search for 'ARM64' and ensure these are checked:" -ForegroundColor White
Write-Host "   - MSVC v143 - VS 2022 C++ ARM64 build tools" -ForegroundColor Gray
Write-Host "   - Windows 11 SDK (10.0.xxxxx.0) for Desktop C++ [ARM64]" -ForegroundColor Gray
Write-Host "   - Windows SDK for Desktop C++ [ARM64]" -ForegroundColor Gray
Write-Host "5. Click Modify and wait for installation" -ForegroundColor White
Write-Host "6. Restart your computer" -ForegroundColor White
Write-Host "7. Run this script again to verify" -ForegroundColor White



