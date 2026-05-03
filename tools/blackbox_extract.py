#!/usr/bin/env python3
from __future__ import annotations

import argparse
import sys
from pathlib import Path


MAGIC = b"CDOo"
PRINTABLE = set(range(32, 127)) | {9, 10, 13}
OBERON_HINTS = (
    "MODULE",
    "IMPORT",
    "TYPE",
    "VAR",
    "CONST",
    "PROCEDURE",
    "BEGIN",
    "END",
    "RETURN",
    "POINTER TO",
    "RECORD",
    "ARRAY",
    "IF",
    "THEN",
    "ELSIF",
    "WHILE",
    "REPEAT",
    "UNTIL",
    "FOR",
    "CASE",
    "WITH",
    ":=",
    "(**",
    "*)",
)


def extract_ascii_runs(data: bytes, min_length: int) -> list[str]:
    runs: list[str] = []
    current = bytearray()
    for byte in data:
        if byte in PRINTABLE:
            current.append(byte)
            continue
        if len(current) >= min_length:
            runs.append(current.decode("latin-1"))
        current.clear()
    if len(current) >= min_length:
        runs.append(current.decode("latin-1"))
    return runs


def source_like_runs(runs: list[str]) -> list[str]:
    filtered: list[str] = []
    for run in runs:
        upper = run.upper()
        if any(hint in upper for hint in OBERON_HINTS):
            filtered.append(run)
    return filtered


def header_strings(data: bytes) -> list[str]:
    return extract_ascii_runs(data[:512], 4)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Inspect BlackBox .odc/.ocf/.osf files and extract embedded readable text."
    )
    parser.add_argument("path", type=Path, help="Path to a BlackBox file")
    parser.add_argument(
        "--min-length",
        type=int,
        default=4,
        help="Minimum ASCII run length to keep (default: 4)",
    )
    parser.add_argument(
        "--mode",
        choices=("summary", "strings", "source"),
        default="summary",
        help="summary: header plus sample strings, strings: all printable runs, source: Oberon-like runs only",
    )
    parser.add_argument(
        "--limit",
        type=int,
        default=200,
        help="Maximum number of extracted lines to print (default: 200)",
    )
    parser.add_argument(
        "--output",
        type=Path,
        help="Optional output file. Defaults to stdout.",
    )
    return parser.parse_args()


def build_output(path: Path, data: bytes, mode: str, min_length: int, limit: int) -> str:
    runs = extract_ascii_runs(data, min_length)
    header = header_strings(data)

    lines = [
        f"file: {path}",
        f"size: {len(data)} bytes",
        f"magic: {data[:4].decode('latin-1', errors='replace') if len(data) >= 4 else '<short file>'}",
        f"header_match: {'yes' if data.startswith(MAGIC) else 'no'}",
        "",
        "header strings:",
    ]
    lines.extend(f"  {entry}" for entry in header[:20])

    if mode == "summary":
        body = source_like_runs(runs)
        title = "sample source-like strings"
    elif mode == "source":
        body = source_like_runs(runs)
        title = "source-like strings"
    else:
        body = runs
        title = "printable strings"

    lines.extend(["", f"{title}:" ])
    if not body:
        lines.append("  <none>")
    else:
        for entry in body[:limit]:
            lines.append(entry)
        if len(body) > limit:
            lines.append("")
            lines.append(f"... truncated after {limit} lines ...")

    return "\n".join(lines) + "\n"


def main() -> int:
    args = parse_args()
    if not args.path.exists():
        print(f"error: file not found: {args.path}", file=sys.stderr)
        return 1

    data = args.path.read_bytes()
    output = build_output(args.path, data, args.mode, args.min_length, args.limit)

    if args.output is not None:
        args.output.write_text(output, encoding="utf-8")
    else:
        sys.stdout.write(output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())