# How to Install ARM64 Build Components

## Summary

You have Windows SDK ARM64 libraries, but you're missing **MSVC ARM64 runtime libraries** (`msvcrt.lib`).

## What You Need to Install

**Component Name:** `MSVC v143 - VS 2022 C++ ARM64 build tools`

This component includes:
- ARM64 compiler (`cl.exe`)
- ARM64 runtime libraries (`msvcrt.lib`, `msvcp.lib`, etc.)
- ARM64 linker (`link.exe`)

## Where to Find It in Visual Studio Installer

### Method 1: Individual Components Tab (Recommended)

1. Open **Visual Studio Installer**
2. Click **Modify** on **Build Tools 2022**
3. Go to **Individual components** tab
4. In the search box, type: `ARM64`
5. Look for:
   - ✅ **"MSVC v143 - VS 2022 C++ ARM64 build tools"** ← CHECK THIS
   - ✅ "Windows 11 SDK (10.0.xxxxx.0) for Desktop C++ [ARM64]" (optional, you already have SDK)

### Method 2: Desktop Development Workload

1. Open **Visual Studio Installer**
2. Click **Modify** on **Build Tools 2022**
3. Check **"Desktop development with C++"** workload
4. Click **Installation details** (right side)
5. Expand **"MSVC v143 - VS 2022 C++ build tools"**
6. Check **"ARM64 build tools"**

### Method 3: Search All Components

1. Open **Visual Studio Installer**
2. Click **Modify** on **Build Tools 2022**
3. Go to **Individual components** tab
4. Search for: `MSVC ARM64` or `C++ ARM64`
5. Check any component that mentions "ARM64" and "MSVC" or "C++"

## Alternative: Install via Command Line

If Visual Studio Installer UI doesn't show it, try:

```powershell
# Download Visual Studio Installer command-line tool
# Then run:
vs_buildtools.exe modify --installPath "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools" --add Microsoft.VisualStudio.Component.VC.Tools.ARM64 --quiet
```

## Verify Installation

After installation, check:

```powershell
Test-Path "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\14.44.35207\lib\arm64\msvcrt.lib"
```

If this returns `True`, you're ready!

## If Still Not Found

1. **Update Visual Studio Installer** - Get the latest version
2. **Try Visual Studio 2022 Community** instead of Build Tools (better component selection)
3. **Check Visual Studio version** - ARM64 support improved in recent versions
4. **Install Windows SDK separately** - Download from Microsoft website

## Current Status

✅ Windows SDK ARM64: **INSTALLED**  
❌ MSVC ARM64 Runtime: **MISSING** (needs `msvcrt.lib`)

## After Installation

Once installed, restart your terminal and run:

```powershell
cargo build
```

This should now work!




