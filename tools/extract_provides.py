"""For each module, extract:
  - The first paragraph after the `(** ... **)` doc block (a short purpose blurb if any)
  - Exported (asterisk-suffixed) PROCEDURE names
  - Exported TYPE names
  - Exported CONST/VAR counts
"""

from __future__ import annotations
import json
import re
import sys
sys.path.insert(0, 'tools')
from extract_imports import extract_text
from pathlib import Path

ROOT = Path(r"E:/NewCP/YAML")

# Strip `(** ... **)` doc blocks (Component Pascal doc-block delimiters)
DOCBLOCK_RE = re.compile(r'\(\*\*.*?\*\*\)', re.DOTALL)
# Strip `(* ... *)` regular block comments
COMMENT_RE = re.compile(r'\(\*.*?\*\)', re.DOTALL)

PROC_RE = re.compile(r'\bPROCEDURE\s+(?:\([^)]*\)\s*)?([A-Za-z][A-Za-z0-9]*)\*')
TYPE_RE = re.compile(r'\b([A-Za-z][A-Za-z0-9]*)\*\s*=\s*(?:POINTER\s+TO\s+|EXTENSIBLE\s+|ABSTRACT\s+|RECORD\b|ARRAY\b|POINTER\b)')

def analyze(path: Path) -> dict:
    raw = extract_text(path)
    # First — try to grab the contents inside the leading (** ... **) for a description
    doc = ''
    m = re.search(r'\(\*\*(.*?)\*\*\)', raw, re.DOTALL)
    if m:
        doc = m.group(1)
    # Strip docblocks and comments to get clean source for symbol extraction
    src = DOCBLOCK_RE.sub('', raw)
    src = COMMENT_RE.sub('', src)
    procs = sorted(set(PROC_RE.findall(src)))
    types = sorted(set(TYPE_RE.findall(src)))
    return {
        'doc_excerpt': doc[:2000],
        'exported_procedures': procs,
        'exported_types': types,
        'src_chars': len(src),
    }

def main() -> int:
    out = {}
    for sub in sorted(ROOT.iterdir()):
        if not sub.is_dir():
            continue
        moddir = sub / 'Mod'
        if not moddir.is_dir():
            continue
        for f in sorted(moddir.glob('*.odc.yaml')):
            file_stem = f.name.removesuffix('.odc.yaml')
            key = f"{sub.name}/{file_stem}"
            out[key] = analyze(f)
    Path('tools/provides.json').write_text(json.dumps(out, indent=2))
    print(f"wrote {len(out)} entries")
    return 0

if __name__ == '__main__':
    sys.exit(main())
