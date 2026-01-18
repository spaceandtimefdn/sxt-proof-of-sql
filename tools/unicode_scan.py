#!/usr/bin/env python3
"""
Unicode scan tool to detect hidden/bidi control characters.

Scans for dangerous codepoints:
- U+202A..U+202E (bidi embedding/override)
- U+2066..U+2069 (bidi isolates)
- U+200E, U+200F (LRM, RLM)
- U+061C (Arabic Letter Mark)

Usage: python3 unicode_scan.py [directory]
"""

import os
import sys
import subprocess

# Dangerous unicode codepoints
DANGEROUS_CODEPOINTS = {
    0x202A: "LEFT-TO-RIGHT EMBEDDING",
    0x202B: "RIGHT-TO-LEFT EMBEDDING",
    0x202C: "POP DIRECTIONAL FORMATTING",
    0x202D: "LEFT-TO-RIGHT OVERRIDE",
    0x202E: "RIGHT-TO-LEFT OVERRIDE",
    0x2066: "LEFT-TO-RIGHT ISOLATE",
    0x2067: "RIGHT-TO-LEFT ISOLATE",
    0x2068: "FIRST STRONG ISOLATE",
    0x2069: "POP DIRECTIONAL ISOLATE",
    0x200E: "LEFT-TO-RIGHT MARK",
    0x200F: "RIGHT-TO-LEFT MARK",
    0x061C: "ARABIC LETTER MARK",
}

def get_tracked_files(directory):
    """Get list of git-tracked text files."""
    try:
        result = subprocess.run(
            ["git", "ls-files"],
            cwd=directory,
            capture_output=True,
            text=True,
            check=True
        )
        return [f.strip() for f in result.stdout.strip().split('\n') if f.strip()]
    except subprocess.CalledProcessError:
        return []

def is_text_file(filepath):
    """Check if file is likely a text file."""
    text_extensions = {
        '.rs', '.py', '.js', '.ts', '.jsx', '.tsx', '.md', '.txt', '.toml',
        '.json', '.yaml', '.yml', '.html', '.css', '.sh', '.bash', '.zsh',
        '.gitignore', '.tape', '.lock', '.cfg', '.ini', '.env'
    }
    _, ext = os.path.splitext(filepath)
    basename = os.path.basename(filepath)
    
    if ext.lower() in text_extensions:
        return True
    if basename in {'Cargo.toml', 'Cargo.lock', 'README', 'LICENSE', 'Makefile'}:
        return True
    return False

def scan_file(filepath):
    """Scan a file for dangerous unicode codepoints."""
    findings = []
    try:
        with open(filepath, 'r', encoding='utf-8', errors='replace') as f:
            for line_num, line in enumerate(f, 1):
                for col, char in enumerate(line, 1):
                    codepoint = ord(char)
                    if codepoint in DANGEROUS_CODEPOINTS:
                        findings.append({
                            'file': filepath,
                            'line': line_num,
                            'col': col,
                            'codepoint': codepoint,
                            'name': DANGEROUS_CODEPOINTS[codepoint]
                        })
    except Exception as e:
        pass  # Skip files that can't be read
    return findings

def main():
    directory = sys.argv[1] if len(sys.argv) > 1 else '.'
    
    print(f"Scanning for hidden/bidi unicode control characters in: {directory}")
    print("=" * 70)
    
    files = get_tracked_files(directory)
    total_findings = []
    scanned_count = 0
    
    for filepath in files:
        full_path = os.path.join(directory, filepath)
        if os.path.isfile(full_path) and is_text_file(filepath):
            findings = scan_file(full_path)
            total_findings.extend(findings)
            scanned_count += 1
    
    print(f"Scanned {scanned_count} text files")
    print()
    
    if total_findings:
        print(f"FOUND {len(total_findings)} DANGEROUS UNICODE CHARACTERS:")
        print("-" * 70)
        for f in total_findings:
            print(f"  {f['file']}:{f['line']}:{f['col']} - U+{f['codepoint']:04X} ({f['name']})")
        print()
        print("STATUS: FAIL - Hidden unicode detected!")
        return 1
    else:
        print("STATUS: PASS - No hidden/bidi unicode control characters found")
        return 0

if __name__ == "__main__":
    sys.exit(main())
