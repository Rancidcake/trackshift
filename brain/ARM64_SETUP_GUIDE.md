# ARM64 Build Setup Guide

## Problem
Cannot find Windows SDK for ARM64 in Visual Studio Installer.

## Solutions

### Option 1: Install Windows SDK Standalone (Recommended)

The Windows SDK can be installed separately from Visual Studio:

1. **Download Windows SDK Installer:**
   - Visit: https://developer.microsoft.com/en-us/windows/downloads/windows-sdk/
   - Download the latest Windows 11 SDK (or Windows 10 SDK if needed)
   - Run the installer

2. **During Installation:**
   - Select "Windows SDK for Desktop C++ development"
   - **IMPORTANT:** Make sure to check "ARM64" architecture components
   - Complete the installation

3. **Verify Installation:**
   ```powershell
   Get-ChildItem "C:\Program Files (x86)\Windows Kits\10\Lib" | Select-Object Name
   ```
   You should see folders like `10.0.22621.0\um\arm64\` or similar.

### Option 2: Find ARM64 Components in Visual Studio Installer

The ARM64 components might be under different names:

1. **Open Visual Studio Installer**
2. **Click "Modify" on Build Tools 2022**
3. **Look for these sections:**
   - **Individual components** tab
   - Search for: "ARM64", "arm64", "aarch64"
   - Look for:
     - "MSVC v143 - VS 2022 C++ ARM64 build tools"
     - "Windows 11 SDK (10.0.xxxxx.0) for Desktop C++ [ARM64]"
     - "C++ ARM64 build tools"
     - "Windows SDK for ARM64"

4. **Alternative locations:**
   - Under "Desktop development with C++" workload
   - Under "Individual components" → "SDKs, libraries, and frameworks"
   - Under "Individual components" → "Compilers, build tools, and runtimes"

### Option 3: Use x86_64 Target (Temporary Workaround)

If you need to test immediately, you can build for x86_64 (which you already have):

```powershell
# Build for x86_64 instead
cargo build --target x86_64-pc-windows-msvc

# Run examples
cargo run --target x86_64-pc-windows-msvc --example scheduler_integration
```

**Note:** This will create x86_64 binaries, not ARM64. Use only for testing.

### Option 4: Manual SDK Installation via Command Line

1. **Download Windows SDK ISO or installer**
2. **Extract/Install with ARM64 components:**
   ```powershell
   # If you have the SDK installer, run it with:
   # Select "ARM64" during installation
   ```

### Option 5: Check if SDK is Already Installed

Run this to check what's available:

```powershell
# Check Windows Kits
Get-ChildItem "C:\Program Files (x86)\Windows Kits\10\Lib" -Recurse -Filter "*.lib" -ErrorAction SilentlyContinue | 
    Where-Object { $_.DirectoryName -like "*arm64*" } | 
    Select-Object DirectoryName -Unique

# Check Visual Studio MSVC tools
Get-ChildItem "C:\Program Files\Microsoft Visual Studio\2022\*\VC\Tools\MSVC" -Recurse -Filter "*arm64*" -ErrorAction SilentlyContinue | 
    Select-Object FullName -Unique
```

## After Installation

Once ARM64 SDK is installed, verify:

```powershell
# Check for ARM64 libraries
Test-Path "C:\Program Files (x86)\Windows Kits\10\Lib\*\um\arm64\kernel32.lib"

# Try building
cargo build
```

## If Still Not Found

If you still can't find ARM64 components:

1. **Update Visual Studio Installer** to the latest version
2. **Check Visual Studio version** - ARM64 support improved in recent versions
3. **Consider Visual Studio 2022 Community/Professional** instead of Build Tools (has better component selection UI)
4. **Install Windows 11 SDK separately** (most reliable method)

## Quick Test

After setup, test with:

```powershell
cargo check
```

If it works, you're ready to build!




