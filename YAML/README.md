# YAML — BlackBox `.odc` corpus, lifted to YAML

This directory is a complete YAML projection of the BlackBox Component Builder
1.7 document tree. Every `.odc` file in the original BlackBox install was read
by [`newcp-odc`](../NewCP/src/newcp-odc) and serialized to a structural YAML
form alongside it (renamed `*.odc.yaml`). 675 files, ~25 MB.

## What it is

- A *reference corpus* for the legacy `Stores.Store` binary format that
  BlackBox uses for documentation, modules-as-rich-text, forms, and resources.
- A grep-friendly view across all of BlackBox's docu / mod / rsrc text — much
  faster than round-tripping through the binary reader for ad-hoc questions
  like "where does this view kind appear" or "what attributes does subsystem
  X actually use".
- A regression target for `newcp-odc` reader/writer parity work. When the
  reader changes, regenerate this corpus and diff against the committed copy
  to see what shifted.

## What it is *not*

- Not authoritative source. The authoritative form remains the original
  `.odc` files in a BlackBox install. YAML is a projection.
- Not byte-stable across reader versions. Each `newcp-odc` change to its
  decoded view representation can shift YAML output. Use `--check` (below)
  for byte-identical round-trip verification, not the YAML diff.
- Not a replacement for the BlackBox install. Some content (binary blobs,
  control payloads we have not decoded yet) is summarized rather than
  reproduced verbatim.

## Layout

The folder structure mirrors a BlackBox 1.7 install:

```
YAML/
  Com/        Comm/      Ctl/       Dev/        Docu/
  Form/       Host/      Obx/       Ole/        Sql/
  Std/        System/    Text/      Win/        Xhtml/
  Empty.odc.yaml
  Tour.odc.yaml
```

Each subsystem typically contains `Mod/`, `Docu/`, and `Rsrc/` subdirectories,
matching BlackBox's own conventions.

## How it was generated

Using the `odc_yaml` binary from the [newcp-odc](../NewCP/src/newcp-odc)
crate:

```
cargo run -p newcp-odc --bin odc_yaml -- <input.odc> -o <output.odc.yaml>
```

The corpus was produced by walking a BlackBox 1.7 tree and running the above
on every `.odc` file, preserving directory structure.

Other modes the tool supports:

| Flag | Purpose |
|---|---|
| (default) | Read `.odc`, emit structural YAML |
| `--tree` | Print a Stores tree view |
| `--rewrite` | Round-trip through the AST and write a new `.odc` |
| `--check` | Read + write + compare hashes — verifies byte-identical round-trip |

## Regenerating

From an updated BlackBox source tree:

```bash
# example walk; adjust paths for your install
find /path/to/BlackBox -name "*.odc" | while read f; do
  rel="${f#/path/to/BlackBox/}"
  out="YAML/${rel}.yaml"
  mkdir -p "$(dirname "$out")"
  cargo run -q -p newcp-odc --bin odc_yaml -- "$f" -o "$out"
done
```

Diff the result against the committed copy to see what the reader changed.

## Useful queries

```bash
# every view kind that appears in the corpus
grep -rh "kind:" YAML | sort -u

# every module that imports HostMenus
grep -rl "HostMenus" YAML/*/Mod

# resource strings referenced by the Std subsystem
grep -A1 "string:" YAML/Std/Rsrc
```
