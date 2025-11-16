#!/usr/bin/env python3
"""
Comprehensive fix script for trackshift compilation errors
Uses Python for more reliable string manipulation
"""

import sys
import re
import os

def fix_mod_rs(filepath):
    """Fix all compilation errors in mod.rs"""
    with open(filepath, 'r') as f:
        content = f.read()
    
    original = content
    
    # 1. Fix ORT imports
    content = re.sub(
        r'use ort::\{Session, SessionBuilder, Value\};',
        'use ort::session::Session;\nuse ort::session::builder::SessionBuilder;\nuse ort::value::Value;',
        content
    )
    
    # 2. Add missing imports at the top (after existing use statements)
    imports_to_add = []
    if 'use std::sync::Arc;' not in content:
        imports_to_add.append('use std::sync::Arc;')
    if 'use anyhow::Context;' not in content:
        imports_to_add.append('use anyhow::Context;')
    if 'use anyhow::Result;' not in content:
        imports_to_add.append('use anyhow::Result;')
    
    if imports_to_add:
        # Find the first use statement
        first_use = content.find('use ')
        if first_use != -1:
            # Find the end of the first line
            first_line_end = content.find('\n', first_use)
            # Insert imports before the first use
            content = content[:first_use] + '\n'.join(imports_to_add) + '\n' + content[first_use:]
        else:
            # No use statements, add at the top
            content = '\n'.join(imports_to_add) + '\n' + content
    
    # 3. Replace Result< with anyhow::Result< (but not if already anyhow::Result)
    content = re.sub(r'-> Result<', '-> anyhow::Result<', content)
    content = re.sub(r'\) -> Result<', ') -> anyhow::Result<', content)
    content = re.sub(r'\(Result<', '(anyhow::Result<', content)
    
    # 4. Fix SessionBuilder - commit_from_file returns Result<Session>, so we can use ? directly
    # Remove .context() calls after commit_from_file and use ? instead
    # Pattern: .commit_from_file(path)\n                .context("...")?
    content = re.sub(
        r'\.commit_from_file\(([^)]+)\)\s*\n\s*\.context\([^)]+\)\?',
        r'.commit_from_file(\1)?',
        content,
        flags=re.MULTILINE
    )
    
    # 5. Fix file_size comparisons - remove .0 from float literals
    content = re.sub(r'file_size > 100_000_000\.0', 'file_size > 100_000_000', content)
    content = re.sub(r'file_size > 10_000_000\.0', 'file_size > 10_000_000', content)
    
    # 6. Remove unused SessionInputValue from import if present
    content = re.sub(r', SessionInputValue', '', content)
    content = re.sub(r'SessionInputValue, ', '', content)
    
    if content != original:
        with open(filepath, 'w') as f:
            f.write(content)
        return True
    return False

def fix_network_quality(filepath):
    """Fix network_quality.rs"""
    if not os.path.exists(filepath):
        return False
    
    with open(filepath, 'r') as f:
        content = f.read()
    
    original = content
    
    # Fix type ambiguity
    content = re.sub(r'let mut score = 1\.0;', 'let mut score: f32 = 1.0;', content)
    content = re.sub(r'score = score\.max\(0\.0\)\.min\(1\.0\);', 'score = score.max(0.0_f32).min(1.0_f32);', content)
    
    if content != original:
        with open(filepath, 'w') as f:
            f.write(content)
        return True
    return False

def main():
    brain_dir = sys.argv[1] if len(sys.argv) > 1 else 'brain'
    mod_rs = os.path.join(brain_dir, 'src', 'telemetry_ai', 'mod.rs')
    network_quality_rs = os.path.join(brain_dir, 'src', 'telemetry_ai', 'network_quality.rs')
    
    if not os.path.exists(mod_rs):
        print(f"Error: {mod_rs} not found")
        sys.exit(1)
    
    print("🔧 Applying comprehensive fixes with Python...")
    
    if fix_mod_rs(mod_rs):
        print("  ✅ Fixed mod.rs")
    
    if fix_network_quality(network_quality_rs):
        print("  ✅ Fixed network_quality.rs")
    
    print("✅ All fixes applied!")

if __name__ == '__main__':
    main()

