# Push to GitHub - Authentication Guide

## Current Issue
Git is using cached credentials for a different GitHub account (`mardromus` instead of `kushgoyalmard`).

## Solution: Use Personal Access Token

### Step 1: Generate GitHub Personal Access Token

1. **Go to GitHub Settings:**
   - Visit: https://github.com/settings/tokens
   - Or: GitHub → Settings → Developer settings → Personal access tokens → Tokens (classic)

2. **Generate New Token:**
   - Click "Generate new token" → "Generate new token (classic)"
   - **Note:** Give it a name like "trackshift-push"
   - **Expiration:** Choose your preferred duration (90 days, 1 year, or no expiration)
   - **Scopes:** Check `repo` (this gives full repository access)
   - Click "Generate token"

3. **Copy the Token:**
   - ⚠️ **IMPORTANT:** Copy it immediately - you won't see it again!
   - It will look like: `ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx`

### Step 2: Clear Old Credentials

**Option A: Using Windows Credential Manager (GUI)**
1. Press `Win + R`, type `control /name Microsoft.CredentialManager`
2. Go to "Windows Credentials"
3. Find entries with "github.com" or "git:https://github.com"
4. Click on them and select "Remove"

**Option B: Using Command Line**
```powershell
# List GitHub credentials
cmdkey /list | Select-String -Pattern "github"

# Remove specific credential (replace with actual target name)
cmdkey /delete:git:https://github.com
```

### Step 3: Push with Token

When you push, Git will prompt for credentials:
- **Username:** `kushgoyalmard`
- **Password:** Paste your Personal Access Token (not your GitHub password!)

```powershell
git push -u origin main
```

### Alternative: Store Token in URL (Less Secure)

If you want to avoid entering credentials each time:

```powershell
git remote set-url origin https://kushgoyalmard:YOUR_TOKEN@github.com/kushgoyalmard/vinod.git
git push -u origin main
```

⚠️ **Warning:** This stores the token in plain text in `.git/config`. Only use if you're comfortable with this.

### Alternative: Use GitHub CLI

If you have GitHub CLI installed:

```powershell
gh auth login
git push -u origin main
```

## Verify Push

After successful push, visit:
- https://github.com/kushgoyalmard/vinod

You should see all your files there!

## Troubleshooting

**If you still get permission errors:**
1. Make sure the token has `repo` scope
2. Make sure you're using the correct username (`kushgoyalmard`)
3. Try clearing all GitHub credentials: `cmdkey /delete:git:https://github.com`
4. Restart your terminal/PowerShell

**If token doesn't work:**
- Check token expiration
- Regenerate token if needed
- Make sure `repo` scope is selected

