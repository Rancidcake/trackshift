#!/bin/bash
# Script to push with Personal Access Token
# Usage: ./git_push_token.sh YOUR_TOKEN

if [ -z "$1" ]; then
    echo "Usage: ./git_push_token.sh YOUR_TOKEN"
    echo ""
    echo "Get your token from: https://github.com/settings/tokens"
    echo "Select scope: repo (full control)"
    exit 1
fi

TOKEN=$1
REPO="mardromus/PitlinkPQC"

echo "Setting remote URL with token..."
git remote set-url origin https://${TOKEN}@github.com/${REPO}.git

echo "Pushing to origin/main..."
git push origin main

echo ""
echo "✅ Done! If successful, your changes are now on GitHub."
echo "🔒 Security: Consider removing token from remote URL after push:"
echo "   git remote set-url origin git@github.com:${REPO}.git"
