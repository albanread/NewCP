"""Extract MODULE / IMPORT statements from YAML-lifted .odc files.

The Component Pascal source text in a `.odc.yaml` is split across many
`- text: "..."` runs (each with its own font attribute). To recover the
module declaration and its IMPORT list we just concatenate every text
run in order, then run a tolerant regex over the result.

Output: TSV rows on stdout: subsystem<TAB>module<TAB>import1,import2,...
"""

from __future__ import annotations
import os
import re
import sys
import json
from pathlib import Path

ROOT = Path(r"E:/NewCP/YAML")

# Match any `- text: <value>` line and capture the value.
# The YAML emitter uses either bare strings or double-quoted strings
# with C-style escapes. We undo \r, \n, \t and \" / \\ for our purposes.
TEXT_RE = re.compile(r'^\s*-\s+text:\s+(.*?)\s*$')

ESCAPES = {
    '\\r': '\n',   # CR -> newline (BlackBox uses CR as line term)
    '\\n': '\n',
    '\\t': '\t',
    '\\"': '"',
    '\\\\': '\\',
}

def unescape(s: str) -> str:
    if s.startswith('"') and s.endswith('"'):
        s = s[1:-1]
    out = []
    i = 0
    while i < len(s):
        if s[i] == '\\' and i + 1 < len(s):
            two = s[i:i+2]
            out.append(ESCAPES.get(two, s[i+1]))
            i += 2
        else:
            out.append(s[i])
            i += 1
    return ''.join(out)

def extract_text(path: Path) -> str:
    chunks: list[str] = []
    with path.open('r', encoding='utf-8', errors='replace') as f:
        for line in f:
            m = TEXT_RE.match(line)
            if not m:
                continue
            chunks.append(unescape(m.group(1).rstrip()))
    return ''.join(chunks)

# Match: MODULE <Name>; (with optional leading whitespace, possibly preceded by IN keyword forms)
MODULE_RE = re.compile(r'\bMODULE\s+([A-Za-z][A-Za-z0-9]*)\b')

# Match: IMPORT ... ;  (non-greedy up to the next semicolon, allowing
# newlines and aliases)
IMPORT_RE = re.compile(r'\bIMPORT\b([^;]*);', re.DOTALL)

# Inside an IMPORT clause, each item is either `Name` or `Alias := Name`.
ITEM_RE = re.compile(r'(?:[A-Za-z][A-Za-z0-9]*\s*:=\s*)?([A-Za-z][A-Za-z0-9]*)')

def parse_imports(src: str) -> tuple[str | None, list[str]]:
    mod_m = MODULE_RE.search(src)
    mod = mod_m.group(1) if mod_m else None
    imp_m = IMPORT_RE.search(src)
    imports: list[str] = []
    if imp_m:
        clause = imp_m.group(1)
        # Strip line comments (* ... *) — Component Pascal block comments
        clause = re.sub(r'\(\*.*?\*\)', '', clause, flags=re.DOTALL)
        # SYSTEM is a pseudo-module; keep it but flag it
        for m in ITEM_RE.finditer(clause):
            name = m.group(1)
            # Skip the alias half is already handled; ITEM_RE captures the
            # *target* name in group(1)
            imports.append(name)
    return mod, imports

def main() -> int:
    rows = []
    for sub in sorted(ROOT.iterdir()):
        if not sub.is_dir():
            continue
        moddir = sub / 'Mod'
        if not moddir.is_dir():
            continue
        for f in sorted(moddir.glob('*.odc.yaml')):
            src = extract_text(f)
            mod, imports = parse_imports(src)
            file_stem = f.name.removesuffix('.odc.yaml')
            # Qualified module name = subsystem + file stem (BlackBox convention)
            qualified = f"{sub.name}{file_stem}"
            rows.append({
                'subsystem': sub.name,
                'file': file_stem,
                'module': mod,
                'qualified_guess': qualified,
                'imports': imports,
            })
    json.dump(rows, sys.stdout, indent=2)
    return 0

if __name__ == '__main__':
    sys.exit(main())
