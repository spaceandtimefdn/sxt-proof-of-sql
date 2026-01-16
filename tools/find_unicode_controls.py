#!/usr/bin/env python3
"""
Scan tracked files for hidden Unicode control characters (bidi, formatting).
"""

from __future__ import annotations

import subprocess
import sys
import unicodedata
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, List


# Codepoints/ranges to flag explicitly.
CONTROL_POINTS = {
    0x200E,  # LEFT-TO-RIGHT MARK
    0x200F,  # RIGHT-TO-LEFT MARK
    0x061C,  # ARABIC LETTER MARK
}
CONTROL_RANGES = [
    (0x202A, 0x202E),  # LRE..RLO, PDF
    (0x2066, 0x2069),  # LRI..PDI
]
BIDI_CLASSES = {"LRE", "RLE", "LRO", "RLO", "PDF", "LRI", "RLI", "FSI", "PDI"}
BINARY_SUFFIXES = {
    ".png",
    ".jpg",
    ".jpeg",
    ".gif",
    ".bmp",
    ".pdf",
    ".webp",
    ".ico",
    ".svgz",
}


@dataclass
class Finding:
    path: Path
    line: int
    column: int
    codepoint: int
    bidi: str
    name: str

    def format(self) -> str:
        cp_hex = f"U+{self.codepoint:04X}"
        return (
            f"{self.path}:{self.line}:{self.column} "
            f"{cp_hex} {self.name} (bidi={self.bidi})"
        )


def iter_tracked_files() -> List[Path]:
    result = subprocess.run(
        ["git", "ls-files"],
        check=True,
        stdout=subprocess.PIPE,
        text=True,
    )
    return [Path(line) for line in result.stdout.splitlines() if line]


def is_control(ch: str) -> bool:
    cp = ord(ch)
    if cp in CONTROL_POINTS:
        return True
    if any(start <= cp <= end for start, end in CONTROL_RANGES):
        return True
    return unicodedata.bidirectional(ch) in BIDI_CLASSES


def is_binary(path: Path) -> bool:
    return path.suffix.lower() in BINARY_SUFFIXES


def scan_file(path: Path) -> Iterable[Finding]:
    if is_binary(path):
        return []
    try:
        content = path.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        content = path.read_text(encoding="utf-8", errors="ignore")
    for line_no, line in enumerate(content.splitlines(), start=1):
        for col_no, ch in enumerate(line, start=1):
            if is_control(ch):
                yield Finding(
                    path=path,
                    line=line_no,
                    column=col_no,
                    codepoint=ord(ch),
                    bidi=unicodedata.bidirectional(ch),
                    name=unicodedata.name(ch, "UNKNOWN"),
                )


def main() -> int:
    findings: List[Finding] = []
    for path in iter_tracked_files():
        findings.extend(scan_file(path))
    if findings:
        for finding in findings:
            print(finding.format())
        return 1
    print("No Unicode control characters found.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
