# GitHub Repository Setup Guide

## Step 1: Create GitHub Repository

1. **Go to GitHub:**
   - Visit: https://github.com/new
   - Or click the "+" icon in the top right → "New repository"

2. **Repository Settings:**
   - **Repository name:** `trackshift` (or your preferred name)
   - **Description:** "Rust-based telemetry AI decision system for network routing and scheduling"
   - **Visibility:** Choose Public or Private
   - **DO NOT** initialize with README, .gitignore, or license (we already have these)
   - Click **"Create repository"**

## Step 2: Push to GitHub

After creating the repository, GitHub will show you commands. Use these:

```powershell
# Add the remote repository (replace YOUR_USERNAME with your GitHub username)
git remote add origin https://github.com/YOUR_USERNAME/trackshift.git

# Rename main branch if needed (GitHub uses 'main' by default)
git branch -M main

# Push to GitHub
git push -u origin main
```

## Alternative: Using SSH

If you have SSH keys set up with GitHub:

```powershell
git remote add origin git@github.com:YOUR_USERNAME/trackshift.git
git branch -M main
git push -u origin main
```

## Step 3: Verify

After pushing, visit your repository on GitHub:
- `https://github.com/YOUR_USERNAME/trackshift`

You should see all your files there!

## Quick Command Reference

```powershell
# Check current remotes
git remote -v

# Push changes
git push

# Pull changes
git pull

# Check status
git status

# View commit history
git log --oneline
```

## Notes

- The `.gitignore` excludes:
  - `target/` directory (build artifacts)
  - `*.onnx` model files (too large for git)
  - Temporary files and IDE configs

- Model files (`slm.onnx` and `embedder.onnx`) are excluded because they're large.
  - Users should generate them using `scripts/create_onnx_models.py`
  - Or download them separately if you host them elsewhere

- If you want to include model files, you can:
  1. Use Git LFS: `git lfs track "*.onnx"`
  2. Or remove `models/*.onnx` from `.gitignore`

