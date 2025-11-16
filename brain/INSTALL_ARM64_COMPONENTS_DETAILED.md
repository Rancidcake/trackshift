# Detailed ARM64 Component Installation Guide

## Current Status
❌ **MSVC ARM64 Runtime Libraries**: Missing  
❌ **Windows SDK ARM64 Libraries**: Missing  
⚠️ **ARM64 Compiler Tools**: May be partially installed

## Step-by-Step Installation

### Method 1: Individual Components (Recommended)

1. **Open Visual Studio Installer**
   - Search for "Visual Studio Installer" in Start Menu
   - Or run: `C:\Program Files (x86)\Microsoft Visual Studio\Installer\vs_installer.exe`

2. **Modify Build Tools 2022**
   - Find "Visual Studio Build Tools 2022"
   - Click **Modify** button

3. **Go to Individual Components Tab**
   - Click the **"Individual components"** tab (not "Workloads")
   - This is critical - workloads may not include all ARM64 components

4. **Search and Select ARM64 Components**
   - In the search box, type: `ARM64`
   - Check ALL of these components:
     - ✅ **MSVC v143 - VS 2022 C++ ARM64 build tools** (or latest version)
     - ✅ **Windows 11 SDK (10.0.xxxxx.0) for Desktop C++ [ARM64]**
     - ✅ **Windows SDK for Desktop C++ [ARM64]**
     - ✅ **C++ ARM64 build tools** (if available)

5. **Also Check These Categories:**
   - Expand **"Compilers, build tools, and runtimes"**
     - Look for any ARM64-related entries
   - Expand **"SDKs, libraries, and frameworks"**
     - Look for Windows SDK ARM64 entries

6. **Click Modify**
   - Wait for installation to complete
   - This may take 10-30 minutes depending on your connection

7. **Restart Your Computer**
   - Required for library paths to be registered

8. **Verify Installation**
   - Run: `powershell -ExecutionPolicy Bypass -File scripts\diagnose_arm64.ps1`
   - All checks should show [FOUND] or [OK]

### Method 2: Full Visual Studio Community (Alternative)

If Build Tools doesn't work:

1. Download **Visual Studio 2022 Community** (free)
   - https://visualstudio.microsoft.com/downloads/
   - Select "Community" edition

2. During installation:
   - Select **"Desktop development with C++"** workload
   - In Installation details, expand and check:
     - ✅ **MSVC v143 - VS 2022 C++ ARM64 build tools**
     - ✅ **Windows 11 SDK (10.0.xxxxx.0) for Desktop C++ [ARM64]**

3. Complete installation and restart

## Verification

After installation and restart, verify:

```powershell
# Check MSVC ARM64 libraries
Test-Path "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\14.44.35207\lib\arm64\msvcrt.lib"

# Check Windows SDK ARM64 libraries  
Test-Path "C:\Program Files (x86)\Windows Kits\10\Lib\10.0.26100.0\arm64\kernel32.lib"
```

Both should return `True`.

## Troubleshooting

### If libraries still missing after installation:

1. **Check Installation Logs**
   - Visual Studio Installer → More → View Logs
   - Look for errors related to ARM64 components

2. **Try Repair**
   - Visual Studio Installer → More → Repair
   - This may fix incomplete installations

3. **Manual Component Selection**
   - Sometimes the workload doesn't include all sub-components
   - Always use "Individual components" tab for ARM64

4. **Check Disk Space**
   - ARM64 components require several GB
   - Ensure you have at least 5GB free

## Expected File Locations

After successful installation, these should exist:

- `C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\<version>\lib\arm64\msvcrt.lib`
- `C:\Program Files (x86)\Windows Kits\10\Lib\<version>\arm64\kernel32.lib`
- `C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\<version>\bin\Hostarm64\arm64\cl.exe`

## Next Steps

Once verified:
1. Run `cargo build` again
2. The build should succeed
3. Run `cargo run` to test the application



