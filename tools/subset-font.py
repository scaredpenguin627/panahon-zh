"""Rebuild the bundled CJK subset from the characters the app actually uses.

Usage (from the repository root):

    python tools/subset-font.py --source path/to/NotoSansCJKsc-Regular.otf

The subset keeps every character that appears in `src/` or `core/src/`, plus
printable ASCII, so adding UI copy only requires re-running this script.
"""

from __future__ import annotations

import argparse
import pathlib
import sys

ROOTS = ("src", "core/src")
TARGET = pathlib.Path("assets/NotoSansCJKsc-Subset.otf")
CHARS_FILE = pathlib.Path("assets/ui-font-chars.txt")

EXTRA = "°·…—–、，。：；！？（）《》【】“”‘’％℃㎍±×÷≈≤≥→←↑↓"


def collect() -> str:
    chars: set[str] = set(chr(code) for code in range(0x20, 0x7F))
    chars.update(EXTRA)
    for root in ROOTS:
        for path in pathlib.Path(root).rglob("*.rs"):
            chars.update(path.read_text(encoding="utf-8"))
    printable = {ch for ch in chars if ch.isprintable() and not ch.isspace()}
    return "".join(sorted(printable))


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--source", required=True, help="full Noto Sans CJK SC OTF")
    args = parser.parse_args()

    source = pathlib.Path(args.source)
    if not source.exists():
        print(f"source font not found: {source}", file=sys.stderr)
        return 1

    text = collect()
    CHARS_FILE.parent.mkdir(parents=True, exist_ok=True)
    CHARS_FILE.write_text(text, encoding="utf-8")

    from fontTools import subset

    subset.main(
        [
            str(source),
            f"--text={text}",
            f"--output-file={TARGET}",
            "--layout-features=*",
            "--drop-tables+=DSIG",
            "--no-hinting",
            "--desubroutinize",
            "--name-IDs=*",
            "--recalc-bounds",
        ]
    )

    size = TARGET.stat().st_size
    print(f"{TARGET}: {size} bytes, {len(text)} characters")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
