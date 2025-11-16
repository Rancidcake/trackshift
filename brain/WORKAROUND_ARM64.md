# Workaround: Using Windows SDK Libraries Directly

## Current Situation

You have:
- ✅ ARM64 compiler (`cl.exe`)
- ✅ Windows SDK ARM64 libraries (`kernel32.lib`, etc. in Windows Kits)
- ❌ MSVC ARM64 runtime libraries (`msvcrt.lib` in `lib\arm64`)

## The Problem

Rust's linker is looking for `msvcrt.lib` in:
```
C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\14.44.35207\lib\arm64\msvcrt.lib
```

But this directory doesn't exist. Only `x64`, `x86`, and `onecore` exist.

## Solution: Install the Correct Component

You need to install: **"Windows SDK for Desktop C++ [ARM64]"**

This is different from:
- ❌ "Windows SDK" (general)
- ❌ "Windows SDK ARM64" (just headers/libs)
- ✅ "Windows SDK for Desktop C++ [ARM64]" (includes runtime libraries)

### How to Find It

1. Open Visual Studio Installer
2. Modify Build Tools 2022
3. Go to **Individual components**
4. Search for: `Desktop C++ ARM64` or `SDK Desktop ARM64`
5. Look for component with "[ARM64]" in the name
6. Make sure it says "Desktop C++" not just "SDK"

### Alternative Names

The component might be named:
- "Windows 11 SDK (10.0.xxxxx.0) for Desktop C++ [ARM64]"
- "Windows 10 SDK (10.0.xxxxx.0) for Desktop C++ [ARM64]"
- "Windows SDK for Desktop C++ development [ARM64]"

## Why This Component?

This component installs the MSVC runtime libraries (`msvcrt.lib`, `msvcp.lib`, `vcruntime.lib`) specifically compiled for ARM64, which go into the `lib\arm64` folder.

## Verification

After installation, verify:

```powershell
# Check if lib\arm64 directory exists
Test-Path "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\14.44.35207\lib\arm64"

# Check for msvcrt.lib
Test-Path "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\14.44.35207\lib\arm64\msvcrt.lib"
```

Both should return `True`.

## If Still Not Found

If you can't find this component:

1. **Update Visual Studio Installer** to the latest version
2. **Try Visual Studio 2022 Community** (has better component selection)
3. **Check if you need a different SDK version** - try installing multiple Windows SDK versions

## Temporary Workaround (Not Recommended)

If you absolutely cannot install the component, you could try copying x64 libraries, but this will NOT work correctly for ARM64 code. The ARM64 runtime libraries are architecture-specific and must be installed properly.




