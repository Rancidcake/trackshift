#!/bin/bash
# SSH Setup Script for GitHub

echo "=== 🔐 GitHub SSH Key Setup ==="
echo ""

# Check if key already exists
if [ -f ~/.ssh/id_ed25519 ]; then
    echo "⚠️  SSH key already exists: ~/.ssh/id_ed25519"
    echo "   Public key:"
    cat ~/.ssh/id_ed25519.pub
    echo ""
    echo "If you want to generate a new key, delete the old one first:"
    echo "  rm ~/.ssh/id_ed25519 ~/.ssh/id_ed25519.pub"
    exit 0
fi

# Generate new key
echo "Generating new SSH key..."
ssh-keygen -t ed25519 -C "github-pitlinkpqc" -f ~/.ssh/id_ed25519 -N ""

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ SSH key generated successfully!"
    echo ""
    echo "📋 Your public key (copy this to GitHub):"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    cat ~/.ssh/id_ed25519.pub
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "📋 Next steps:"
    echo "  1. Copy the public key above"
    echo "  2. Go to: https://github.com/settings/keys"
    echo "  3. Click 'New SSH key'"
    echo "  4. Paste the key and click 'Add SSH key'"
    echo "  5. Test: ssh -T git@github.com"
    echo "  6. Push: git push origin main"
    echo ""
    
    # Try to copy to clipboard (macOS)
    if command -v pbcopy > /dev/null; then
        cat ~/.ssh/id_ed25519.pub | pbcopy
        echo "✅ Public key copied to clipboard!"
    fi
else
    echo "❌ Failed to generate SSH key"
    exit 1
fi
