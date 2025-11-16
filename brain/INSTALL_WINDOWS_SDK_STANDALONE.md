# Install Windows SDK Standalone (With ARM64 Support)

## Problem
Your Visual Studio Installer doesn't show "Windows 11 SDK for Desktop C++ [ARM64]" component.

## Solution: Install Windows SDK Separately

The Windows SDK can be installed independently from Visual Studio, and it includes ARM64 support.

### Step 1: Download Windows SDK

1. **Go to Microsoft's Windows SDK download page:**
   - Visit: https://developer.microsoft.com/en-us/windows/downloads/windows-sdk/
   - Or direct link: https://developer.microsoft.com/en-us/windows/downloads/windows-11-sdk/

2. **Download the Windows 11 SDK installer:**
   - Look for "Windows 11 SDK (10.0.26100.0)" or latest version
   - Download the installer (it's a large file, ~1-2 GB)

### Step 2: Run the Installer

1. **Run the downloaded installer** (e.g., `winsdksetup.exe`)

2. **During installation, you'll see options:**
   - ✅ **Windows SDK for Desktop C++ development** (CHECK THIS)
   - ✅ **Windows SDK for UWP C++ development** (optional, but check if you want)
   - ✅ **Windows SDK for .NET development** (optional)

3. **IMPORTANT: Select Architecture Components**
   - Look for "Architecture" or "Architecture components" section
   - Make sure these are checked:
     - ✅ **x64** (for x86_64 builds)
     - ✅ **ARM64** (CRITICAL - this is what we need!)
     - ✅ **x86** (optional, for 32-bit builds)

4. **Complete the installation**
   - Click "Install" and wait (this may take 10-30 minutes)

### Step 3: Verify Installation

After installation, verify ARM64 libraries are installed:

```powershell
# Check for ARM64 libraries in Windows SDK
Get-ChildItem "C:\Program Files (x86)\Windows Kits\10\Lib\*\um\arm64" -Directory
Get-ChildItem "C:\Program Files (x86)\Windows Kits\10\Lib\*\ucrt\arm64" -Directory
```

You should see directories like:
- `C:\Program Files (x86)\Windows Kits\10\Lib\10.0.26100.0\um\arm64\`
- `C:\Program Files (x86)\Windows Kits\10\Lib\10.0.26100.0\ucrt\arm64\`

### Step 4: Check for kernel32.lib

```powershell
Test-Path "C:\Program Files (x86)\Windows Kits\10\Lib\10.0.26100.0\um\arm64\kernel32.lib"
```

Should return `True`.

### Step 5: Still Need MSVC Runtime Libraries

Even after installing Windows SDK, you still need the MSVC ARM64 runtime libraries (`msvcrt.lib`).

**Check Visual Studio Installer again for:**
- Search for: **"MSVC v143 - VS 2022 C++ ARM64 build tools"**
- This component should include the runtime libraries

**If still not found, try:**
- Update Visual Studio Installer to latest version
- Or install Visual Studio 2022 Community (full version) instead of Build Tools

## Alternative: Check Component Names

In Visual Studio Installer, try searching for these exact terms:
- `ARM64`
- `arm64`
- `aarch64`
- `Desktop C++ ARM64`
- `SDK ARM64`

Look under:
- **Individual components** → **SDKs, libraries, and frameworks**
- **Individual components** → **Compilers, build tools, and runtimes**

## After Installation

1. **Restart your computer** (important for library paths)

2. **Run diagnostic:**
   ```powershell
   powershell -ExecutionPolicy Bypass -File scripts\diagnose_arm64.ps1
   ```

3. **Try building:**
   ```powershell
   cargo build
   ```

## What Gets Installed

After installing Windows SDK with ARM64:
- ✅ Windows SDK ARM64 libraries (`kernel32.lib`, `advapi32.lib`, etc.)
- ✅ UCRT ARM64 libraries (`ucrt.lib`)
- ❌ MSVC runtime libraries (`msvcrt.lib`) - still need from Visual Studio

The MSVC runtime libraries (`msvcrt.lib`) come from Visual Studio Build Tools, not the Windows SDK.

## Next Steps

Once Windows SDK ARM64 is installed:
1. Check if MSVC ARM64 build tools component is available
2. If not, we may need to configure Rust to use alternative library paths
3. Or install Visual Studio 2022 Community (full version)



