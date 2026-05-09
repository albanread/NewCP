"""Analyze module dependencies from imports.json.

Outputs the data needed to build the porting-order writeup:
  - per-module: subsystem, file, name, import list (with SYSTEM/COM split out)
  - reverse map: who-imports-X (for fan-in / impact ranking)
  - topological layers (modules with no internal deps first)
  - subsystem rollup (subsystem -> count, subsystem dep graph)
"""

from __future__ import annotations
import json
from collections import defaultdict, Counter
from pathlib import Path

PSEUDO = {'SYSTEM'}  # COM is a real module (Com/Mod/Object.odc.yaml? no — it's a built-in COM pseudo-module on Windows)
# Note: BlackBox's COM module is built-in; not in the YAML corpus. Treat as external.
EXTERNAL_BUILTINS = {'SYSTEM', 'COM'}

data = json.loads(Path('tools/imports.json').read_text())

# Build module index keyed on MODULE declaration name
modules: dict[str, dict] = {}
for d in data:
    name = d['module']
    if name in modules:
        # Duplicate module name across files — shouldn't happen
        print(f"WARN duplicate module name {name}")
    modules[name] = d

all_names = set(modules.keys())

# Categorize each import
internal = defaultdict(list)  # mod -> [imported internal mods]
external = defaultdict(list)  # mod -> [imports outside the corpus]

for name, d in modules.items():
    seen = set()
    for imp in d['imports']:
        if imp in seen:
            continue
        seen.add(imp)
        if imp in EXTERNAL_BUILTINS:
            external[name].append(imp)
        elif imp in all_names:
            internal[name].append(imp)
        else:
            external[name].append(imp)  # unresolved -> probably external (FFI etc) or dropped

# Reverse fan-in
fanin = defaultdict(list)
for name, deps in internal.items():
    for d in deps:
        fanin[d].append(name)

# Topological layers
remaining = {n: set(internal[n]) for n in modules}
layers: list[list[str]] = []
while remaining:
    layer = sorted(n for n, deps in remaining.items() if not deps)
    if not layer:
        # cycle — emit what's left and break
        layer = sorted(remaining)
        layers.append(layer)
        break
    layers.append(layer)
    layer_set = set(layer)
    for n in layer:
        del remaining[n]
    for n in remaining:
        remaining[n] -= layer_set

# Subsystem-level dependency graph
sub_of = {n: d['subsystem'] for n, d in modules.items()}
sub_deps = defaultdict(set)
for name, deps in internal.items():
    s = sub_of[name]
    for d in deps:
        ds = sub_of[d]
        if ds != s:
            sub_deps[s].add(ds)

# Output JSON for the writeup
out = {
    'modules': {n: {
        'subsystem': d['subsystem'],
        'file': d['file'],
        'internal_imports': sorted(internal[n]),
        'external_imports': sorted(set(external[n])),
        'fanin': sorted(fanin[n]),
        'fanin_count': len(fanin[n]),
    } for n, d in modules.items()},
    'layers': layers,
    'subsystem_deps': {k: sorted(v) for k, v in sub_deps.items()},
    'subsystem_counts': dict(Counter(sub_of.values())),
}
Path('tools/analysis.json').write_text(json.dumps(out, indent=2))

# Also print a human summary
print(f"Modules: {len(modules)}")
print(f"Topological layers: {len(layers)}")
print(f"Layer sizes: {[len(l) for l in layers]}")
print()
print("Layer 0 (no internal deps):")
for n in layers[0]:
    print(f"  {sub_of[n]:8s} {n:25s} ext={external[n]}")
print()
print("Top 20 fan-in (most-imported modules):")
top = sorted(fanin.items(), key=lambda kv: -len(kv[1]))[:20]
for n, importers in top:
    print(f"  {len(importers):3d}  {n}")
print()
print("Subsystem dep graph:")
for s in sorted(sub_deps):
    print(f"  {s:8s} -> {sub_deps[s]}")
print()
print("Subsystem module counts:")
for s, c in sorted(out['subsystem_counts'].items(), key=lambda kv: -kv[1]):
    print(f"  {s:8s} {c}")
