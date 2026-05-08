# BlackBox `.odc` Documents — YAML Serialization Design

## Purpose

The legacy BlackBox document format `.odc` is a binary serialization of a `View` tree produced by the `Stores` subsystem. It is opaque to every tool outside BlackBox: diff tools cannot read it, version control cannot merge it, search cannot index it cleanly, and the `review-md` rendering throws away the structure (attribute pool, anchors, view boundaries, paragraph rulers) just to surface text.

This document specifies a **YAML projection** of the same document model. The goal is a textual representation that:

- preserves the View tree, the attribute pool, and the piece list — i.e. enough structure to round-trip
- is readable and editable by hand for the common cases (Mod source, Docu prose, Rsrc menus)
- is line-oriented and merges cleanly under git
- degrades gracefully when a View kind is not yet specified — unknown views appear as opaque blobs rather than blocking the rest of the document
- is independent of any particular Component Pascal runtime — it is a file format, not an in-memory ABI

It **is** a goal — and one the implementation now achieves — to byte-equal the original `.odc` after a `read → write` cycle, so the format and tooling can be hash-verified against the legacy binary as ground truth. YAML→bin→YAML round-trip identity (Stage C) is a stronger property; the design supports it but the YAML parser is still pending — see [Implementation status](#implementation-status).

## Implementation status

A reference reader / writer crate lives at [`NewCP/src/newcp-odc/`](../src/newcp-odc/). It exposes a CLI (`odc-yaml`) and a library API matching the schema in this document, and is staged as follows:

| Stage | Scope | Status |
|---|---|---|
| **A** | Stores envelope reader and writer. Walks the store tree via `down`/`next` offsets without understanding any view body. Verbatim body copies + structured envelope reconstruction. | ✅ Done. **675 / 675** files in the BlackBox 1.7 tree (`C:\projects\BlackBox Component Builder 1.7-a1\`) round-trip byte-identically with `odc-yaml --check`. |
| **B** | Structured encoders for every decoded view kind: `TextModels.StdModel`, `TextModels.Attributes`, `StdLinks.Link`, `StdLinks.Target`, `StdFolds.Fold`, `TextRulers.StdRuler`, `TextRulers.StdStyle`, `TextRulers.Attributes`, `TextViews.StdView`, `Controls.*`. Each encoder produces wire bytes from decoded fields, not from copied source. | ✅ Done. Same 675 / 675 sweep continues to pass with the structured encoders active, validating that the AST is lossless. |
| **C** | YAML round-trip — parser from YAML → AST and end-to-end `bin → YAML → bin` hash verification. | ⏳ Pending. The encoders are ready; only the YAML parser and the schema's lossless-mode metadata remain. |

The CLI exposes:

```text
odc-yaml <input.odc> [-o output.yaml]   YAML output (default)
odc-yaml --tree <input.odc>             ASCII tree view of the store hierarchy
odc-yaml --rewrite <input.odc> -o out   re-serialize binary from the parsed AST
odc-yaml --check <input.odc>            read, write, compare; reports ok / MISMATCH
```

## Implementation notes — byte-level findings

Several aspects of the binary format are worth recording precisely because they don't appear in the published BlackBox documentation and were learned by reading the legacy `Stores.Reader` source against real files. They affect any future implementation of this format.

### The Stores envelope is more nuanced than the schema implies

Per-store on-wire layout (after the 4-byte `CDOo` magic and 4 zero bytes):

```text
1 byte    kind:           0x80 nil | 0x81 link | 0x82 store | 0x83 elem | 0x84 newlink
var       path:           type-chain encoding (see below) — only for store/elem
4 bytes   comment:        Int — caller-defined annotation; round-trip must preserve
4 bytes   next:           Int — relative offset to next sibling (or special-case 0 with
                          `(comment & 1) == 1` meaning "next exists right after"), only
                          for store/elem; for nil/link/newlink it's a different formula
4 bytes   down:           Int — relative offset to first child (store/elem only)
4 bytes   len:            Int — length of body bytes that follow (store/elem only)
len bytes body:            store-specific data; may itself contain inline child stores
```

For nil / link / newlink kinds the layout is shorter (`kind + comment + next` for nil, `kind + id + comment + next` for the two link kinds) and they have no body or children.

### Path encoding has multiple wire-equivalent forms

The same logical type chain can be written as any of:

- a sequence of `newExt` entries (`0xF1` + UTF-8 NUL-terminated name) followed by exactly one terminator: either `newBase` (`0xF0` + name) for an entirely-new chain or `oldType` (`0xF2` + 4-byte Int id) splicing into a previously-recorded chain;
- a single `oldType` reference at the very start, when the entire chain matches an existing dictionary entry's base walk;
- different mixes of the above when only a tail of the chain is already known.

The reader has an open-ended type-dictionary that grows in encounter order. **Two distinct files writing the same logical type chain may use different on-wire encodings.** The reconstructed `type_path` (with `Stores.ElemDesc` filtered, names normalised) is *lossy* — to round-trip exactly, the AST must preserve the wire encoding per store. The implementation calls this `wire_path: Option<WirePath>` with variants `Reference(i32) | Extension { extensions: Vec<String>, terminator: NewBase(String) | OldType(i32) }`.

### `Stores.ElemDesc` is a marker, not a class

The reader records `Stores.ElemDesc` in the type dictionary but filters it out of the visible `type_path`. Its purpose is to mark — within the dictionary chain — that the chain belongs to an `elem` store rather than a regular one. **Some BlackBox writers emit it as a `newExt` entry between the user's class chain and the `Stores.Store` base; some writers emit it elsewhere; some don't emit it at all** (e.g. when types are written using `*^` / "POINTER TO" naming rather than `*Desc`). The wire varies. Capturing the wire encoding per-store handles all cases.

### Type names use two parallel naming conventions

Within the same BlackBox 1.7 tree:

- most files use `*Desc` names for type descriptors — `Documents.ModelDesc`, `Stores.StoreDesc`, etc.
- some files use `*^` names for the dereferenced record types — `Documents.Model^`, `Stores.Store^`, etc.

Both encodings are valid and identical in semantics; the reader must accept both, the writer must reproduce whichever the source used. The implementation captures and re-emits them verbatim.

### Every Component Pascal class layer adds a version byte

The legacy `Internalize` pattern is `super^.Internalize(rd); ReadVersion(...); ...`. Each class in the inheritance chain that overrides `Internalize` writes one byte in the body. So a body's prefix is a stack of version bytes, one per layer. For `TextModels.StdModel` (an elem) that's six bytes:

| layer | version byte |
|---|---|
| `Stores.Store` | 1 |
| `Stores.Store` (elem flag adds another) | 1 |
| `Models.Model` | 1 |
| `Containers.Model` | 1 |
| `TextModels.Model` | 1 |
| `TextModels.StdModel` | 1 |

After the version chain comes the type-specific fields. Every structured decoder must capture every version byte for round-trip — discarding any of them breaks `--check`.

### Two-pass init: `Internalize` and `Internalize2`

`TextViews.View` and its descendants use Component Pascal's two-pass init pattern. `Containers.View.Internalize` reads the model and controller as inline child stores, then calls `v.Internalize2(rd)` which dispatches through a separate vtable. `TextViews.View.Internalize2` and `TextViews.StdView.Internalize2` each write their own version byte, but `TextViews.View.Internalize2` does **not** call its `super^.Internalize2` (the call is commented out in the legacy source: `(*v.Internalize^(rd);*)`). So the body for a `TextViews.StdView` is:

```text
1 byte  Stores.Store version
1 byte  Views.View version
1 byte  Containers.View version
store   inline Model         (TextModels.StdModel child)
store   inline Controller    (TextControllers.StdCtrl, or a nil store)
1 byte  TextViews.View Internalize2 version
1 byte  TextViews.StdView Internalize2 version
1 byte  hideMarks Bool
store   inline default Ruler (TextRulers.StdRuler child)
store   inline default Attrs (TextModels.Attributes child)
4 bytes origin Int
4 bytes dy Int
```

### View-piece placeholders in the character buffer

A `TextModels.StdModel` body has a run-list section followed by a character buffer. The run list addresses each piece by `(ano: byte, len: Int, optional w/h Ints, optional inline view store)`. **Each embedded view (run with `len = 0`) consumes exactly one byte in the character buffer** — the legacy reader does `INC(org)` without reading the byte's value. The byte itself sits at the slot position in the char buffer; conventionally `0x02` (STX) but **captured per-piece** by our decoder for round-trip safety. Without this, the placeholder bytes leak into the next text piece's content (the bug we hit during initial decoding showed up as stray `\x02` characters in YAML output).

### `TextRulers.Attributes` has a v0 quirk and a tab-overflow corner case

- **v0 rightFixed patch**: the legacy `Internalize` synthetically sets the `rightFixed` opt bit when reading a v0 file, even though the bit isn't on the wire. Round-trip must NOT apply this transform — store `opts` as-read and let any "behavioural" interpretation happen at display time only.
- **Tab overflow**: the body declares `n` (XInt tab count) but the runtime stores at most `MAX_TABS = 32` tabs. If `n > MAX_TABS`, the trailing `(n - 32)` stops (and types, for v≥2) are read into the cursor but discarded. The implementation captures them in `trash_stops` / `trash_types` and re-emits them so old files with corrupted tab arrays still round-trip.

### Strings: don't transcode through `String`

Three string forms appear on the wire:

- `XString` — 1-byte chars terminated by `0x00` (Latin-1 in legacy files)
- `String` — 2-byte LE codepoints terminated by `0x0000` (UTF-16 in modern files)
- `SString` — 1-byte chars, used for path components (UTF-8 in 1.7+ via `Kernel.Utf8ToString`)

Decoding through Rust's `String` (e.g. `String::from_utf16_lossy`) is **lossy** for malformed bytes and surrogate pairs, and the resulting `String` cannot be encoded back as the same wire bytes. The implementation stores raw bytes / `u16`s in dedicated enums (`LinkString::{Narrow|Wide}`, `CtrlString::{Narrow|Wide}`, `FoldLabel::{Narrow|Wide}`) and provides a `to_string()` for display. The encoder writes the raw form unchanged.

### Unknown view kinds round-trip via verbatim splice

When the writer hits a store kind it has no structured encoder for (and it has children), it falls back to copying the parent's body bytes verbatim while recursively writing each child store via its dispatch. This means an unknown leaf type's bytes are copied verbatim from `src`; an unknown parent type's bytes are reconstructed as `(verbatim primitive sections) + (recursive writes of children)`. The recursive write naturally re-encodes any *known* descendant via its structured encoder. The 675 / 675 sweep was achieved with this fallback covering the few view kinds we don't yet decode (`HostBitmaps.StdView`, `StdHeaders.View`, `TextControllers.StdCtrl`, `Documents.Controller`, …).

### Round-trip strategy

The verification approach: read `.odc` → AST → write `.odc`, hash both, compare. With the type-dictionary mirrored in encounter order between reader and writer, every encoding decision (newExt / newBase / oldType) lines up byte-for-byte. The body's primitive sections (version chain, ints, bools, strings, padding) are reproduced from captured fields; inline child stores recurse through the same dispatch. Any discrepancy surfaces as a `MISMATCH` line on the `--check` sweep.

The implementation has been validated against:

- the entire `BlackBox Component Builder 1.7-a1` distribution: 675 `.odc` files spanning resources, source modules, documentation, samples, tutorials, and the `Tour.odc` (49 KB)
- file kinds: `Empty.odc` (951 bytes — minimal envelope) at the small end, `Std/Docu/TabViews.odc` (207 KB) at the large end
- both `*Desc`- and `*^`-named type chains
- elem stores with and without `Stores.ElemDesc` markers
- folds in collapsed and expanded states
- legacy `Controls.Control` v0–v2 wire shapes alongside modern v3+

## Recap of the binary model

An `.odc` file holds, in order:

1. a 4-byte magic `CDOo`
2. a `Stores` envelope: a chain of type descriptors leading down from the root container's outermost type to `Stores.StoreDesc`, each terminated by `00`, with `f0`/`f1`/`f2` markers distinguishing first-reference / repeat-reference / null
3. the root document's serialized state — recursive: a store writes its own fields, then writes any sub-stores it owns

The outermost store is almost always `Documents.StdDocument`. A `StdDocument` wraps **one root View** plus a `Documents.Model` and `Documents.Controller`. The root view is what carries the visible content — for nearly every file in the BlackBox tree it is a `TextViews.StdView`.

A `TextViews.StdView` is itself a thin display layer over a separable model. The interesting structure lives in:

- `TextModels.StdModel` — the actual text plus a piece list. Each piece is either
  - a **text run**: an attribute-pool reference plus a length, with text bytes in the model's character buffer, or
  - an **embedded view**: an attribute-pool reference plus a child View
- `TextModels.Attributes` — pool entries describing character attribute combinations (font, colour, offset, weight, slant, underline, struck-through)
- `TextRulers.StdRuler` + `TextRulers.StdStyle` — paragraph rulers (margins, tabs, alignment, leading, default font), pooled and referenced by paragraphs
- `Views.View` instances embedded as pieces — frequently `StdLinks.Link`, `StdLinks.Target`, `StdFolds.Fold`, `Controls.PushButton`, `Controls.Caption`, `HostBitmaps.StdView`, `StdHeaders.View`, `TextControllers.StdCtrl`

Two view kinds are central to making the BlackBox docs *readable*, and both are missing from the existing `review-md` extractions:

- **`StdLinks.Link`** / **`StdLinks.Target`** are how every cross-reference in the docs works. A `Link` is an inline view wrapping its visible content (text and/or icon) plus a target command string — almost always `StdCmds.OpenBrowser('Path/To/Doc', 'Caption')` or a direct anchor reference. A `Target` is the matching anchor — a zero-width view at a specific text offset.
- **`StdFolds.Fold`** is a collapsible region. BlackBox documentation uses folds heavily — a "see source" fold hides the example code, a "details" fold hides the deeper paragraphs, etc. The doc is scannable in the IDE precisely because folds compress out detail. A markdown rendering that ignores folds (which the current `review-md` does) deletes that structure and produces walls of unstratified prose.

There is no `MenuViews.View` and no `FormViews.View` as a distinct top-level kind in any `.odc` file in the tree. Inspection confirms:

- `Rsrc/Menus.odc` is a `Documents.StdDocument` wrapping a `TextViews.StdView` whose text content is the menu source (`MENU "Name" ... END` grammar). The framework parses the text at load time. There is no structured menu type in the binary.
- `Rsrc/About.odc` (and other form resources) is a `Documents.StdDocument` wrapping a `TextViews.StdView` that embeds `Controls.PushButton`, `Controls.Caption`, `HostBitmaps.StdView` etc. directly as pieces. The "form layout" is achieved by absolute-positioned views inside the text flow rather than by a separate form view type.

The YAML format mirrors this: every document has the same top envelope, the root is a TextView, and forms / menus / dialogs are just particular populations of pieces inside the TextView.

## Top-level envelope

Every YAML document starts with the same header:

```yaml
odc:
  format: 1                   # YAML schema version, not document version
  source: System/Rsrc/About   # optional: original repo-relative path, for context
  generator: bbcb-yaml/0.1    # optional: tool that produced this YAML
  document:                   # always Documents.StdDocument in practice
    kind: Documents.StdDocument
    version: 0
    root:                     # the single root view the document wraps
      kind: TextViews.StdView
      version: 0
      body: { ... }           # view-specific payload, schema depends on kind
```

`kind` is the namespaced runtime type the binary records (without the trailing `Desc` — `TextViews.StdView`, not `TextViews.StdViewDesc`; the descriptor suffix is a Component Pascal naming convention for type descriptors and is irrelevant to the YAML). Unknown `kind` values are not an error — see [Unknown views](#unknown-views).

`document.kind` is permitted to be anything in the `Documents.*` family; in practice the BlackBox tree uses only `Documents.StdDocument`. The wrapper exists in the YAML even though it carries little useful information, so that future `.odc` files using a different document container (a hypothetical `Documents.SignedDocument`, or a future NewCP variant) round-trip without a schema break.

The remaining sections specify `body` for the root view kinds that appear in the tree. Every one of them is `TextViews.StdView` — what differs is what the TextView contains.

## TextViews.StdView — the universal case

Every `.odc` file in the BlackBox repo is a `Documents.StdDocument` whose root is a `TextViews.StdView`. The body has three parts: a **defs** pool of named attribute sets, a **flow** of pieces, and an optional **anchors** map.

### Attribute pool (`defs`)

Three sub-pools, each a YAML map keyed by a short stable name. Names are local to the document — they only matter because pieces reference them — so the writer is free to choose them (`p1`, `c1`, `body`, `code`, …). Names that *survive* across edits diff better, so a converter should prefer stable names derived from a hash of the attribute content.

```yaml
defs:
  fonts:
    body:    { family: Arial,   size: 10pt }
    bold:    { family: Arial,   size: 10pt, weight: bold }
    italic:  { family: Arial,   size: 10pt, slant: italic }
    bi:      { family: Arial,   size: 10pt, weight: bold, slant: italic }
    code:    { family: Courier, size: 10pt }
    mono-b:  { family: Courier, size: 10pt, weight: bold }
  chars:
    plain:    { font: body }
    em:       { font: italic }
    strong:   { font: bold }
    underline:{ font: body, underline: true }
    link:     { font: bold, underline: true, color: "#000080" }
    super:    { font: body, baseline: super }   # 4pt up, half-size in BB
  paras:
    body:    { left: 0,  right: 0, first: 0,  align: left,    leading: 1.0 }
    indent1: { left: 36, right: 0, first: 0,  align: left,    leading: 1.0 }
    code:    { left: 36, right: 0, first: 0,  align: left,    leading: 1.0,
               tabs: [36, 72, 108, 144], font: code }
```

Notes:

- `chars` references `fonts` by name; this avoids repeating font specs across every char-attr variant.
- `paras` covers what BlackBox calls a *ruler*: margins, tabs, alignment, leading. A paragraph optionally pins a default char-attr (`font`) which applies to runs that don't override.
- Lengths are written with explicit units (`pt`, `px`, `mm`). Internally BlackBox uses millipoints; the YAML uses plain points (`10pt` = 10000 millipoints) and is responsible for round-tripping.
- Colours use CSS `#rrggbb`. A few BlackBox semantic colours (`auto`, `transparent`) are kept as those words.
- Any field omitted means "inherit from the document default", not "empty".

### Flow

The `flow` is an ordered sequence of pieces. Each piece is one of:

- a **paragraph break** that switches the active ruler — `- p: body` (just the ruler name, on its own line)
- a **text run** — `- t: { c: em, text: "Component Pascal" }`
- an **embedded view** — `- view: { ... nested view envelope ... }`
- an **anchor** — `- a: "1.2"` (a zero-width target named `1.2`)
- a **line break inside a paragraph** — `- br:` (rare; most line breaks come from new paragraphs)
- a **tab** — `- tab:` (used inside `code` paragraphs)

The active paragraph is sticky — once `p:` is set, every following `t:`/`view:` belongs to that paragraph until the next `p:`.

The active char-attr is *not* sticky: every `t:` carries its own `c:`. This is more verbose but it makes diffs local — changing one run's style doesn't shift attributes for everything after it. A writer may emit `c: plain` explicitly or omit it; if omitted, the run inherits the paragraph's default font.

### Text content rules

- The `text` field is a normal YAML string. Multiline runs use YAML's literal block scalar (`|`) or folded scalar (`>`), preserving newlines exactly. Soft-wrapped paragraphs should *not* embed `\n` — use sequential paragraph pieces instead.
- Tabs inside text are written as `\t` in flow scalars or as the `- tab:` piece. The `tab:` piece is preferred when the tab is structural (in `code` paragraphs).
- Non-ASCII text is written verbatim; YAML files are UTF-8.
- A run never contains a paragraph break. Paragraphs are always explicit `p:` pieces.

### Anchors and links

Both anchors and hyperlinks are embedded views, and both follow the same paired-view pattern as folds: a `leftSide` view with a payload string + a `rightSide` view with no payload, with the **visible content of the link or anchor sitting between them in the parent text**. This matches BlackBox's hand-coded `<<cmd>>...<>` syntactic representation directly.

`StdLinks.Link.Externalize` writes:

| field          | type   | meaning |
|----------------|--------|---------|
| (super)        | View   | inherited |
| version        | XInt   | 0 = ASCII cmd, no close; 1 = adds close; 2 = Unicode cmd |
| sideBool       | Bool   | TRUE if `cmd # NIL` (leftSide), FALSE if rightSide |
| cmdLen         | Int    | length of `cmd`, or 0 for rightSide |
| cmd            | String | leftSide only; XString for v0/1, full String for v2 |
| close          | Int    | version ≥ 1 only: 0 = always, 1 = ifShiftDown, 2 = never |

`StdLinks.Target.Externalize` writes:

| field      | type   | meaning |
|------------|--------|---------|
| (super)    | View   | inherited |
| version    | XInt   | 0 = ASCII ident, 1 = Unicode ident |
| sideBool   | Bool   | TRUE if `ident # NIL` (leftSide), FALSE if rightSide |
| identLen   | Int    | length, or 0 for rightSide |
| ident      | String | leftSide only |

Two YAML forms again, mirroring the fold approach.

**Lifted form** (recommended):

```yaml
- link:
    target: "StdCmds.OpenBrowser('Docu/BB-License', 'License')"
    close: ifShiftDown                # always | ifShiftDown | never; default ifShiftDown
    body:                             # the visible clickable region
      - t: { c: link, text: "License" }

- target:
    name: "1.2"                       # the anchor identifier
    body:                             # the region the target marks; often a single run
      - t: { c: heading, text: "1.2 Type Declarations" }
```

A converter writes the leftSide view, emits `body` into the parent flow, then writes the matching rightSide view.

**Pair form** (for round-trip cases that resist lifting):

```yaml
- view: { kind: StdLinks.Link, body: { side: left, target: "...", close: ifShiftDown } }
- t: { c: link, text: "License" }
- view: { kind: StdLinks.Link, body: { side: right } }
```

Three `target:` styles for `link` are accepted verbatim:

- a BlackBox command string — `"StdCmds.OpenBrowser('Docu/DTC-Help', 'Help Contents')"`
- an in-document target reference, conventionally invoked through `StdLinks.ShowTarget` — `"StdLinks.ShowTarget('1.2')"`
- a path to another `.odc`, encoded as a command — `"StdCmds.OpenDoc('../../Docu/BB-License.odc')"`

The `close:` field defaults to `ifShiftDown`, which is also the default the BlackBox runtime picks for cmds containing `StdLinks.ShowTarget`. The two encoded versions (cmd-as-XString vs cmd-as-String) are a binary-level concern; the YAML always stores the cmd as a UTF-8 string and the converter chooses the version on output.

Because targets are now real ranges with identifiers, no separate `anchors:` offset map is needed. An optional informational map at the body level — regenerated on save — may help human readers:

```yaml
anchors:
  "1.2": "1.2 Type Declarations"   # informational only, never load-bearing
```

### Folds — `StdFolds.Fold`

A fold is a collapsible region. Folds nest. They are the single biggest reason BlackBox documentation is readable in the IDE — and the single biggest reason the existing `review-md` extractions are not, since the markdown rendering loses the fold structure entirely and pours the hidden body inline as if it were always visible.

The binary representation is interestingly mechanical. A fold is **a pair of `StdFolds.Fold` views** in the parent text's piece list, not a single wrapper:

- a **left fold** — `leftSide = TRUE`, carries the visible label and (when collapsed) owns a `hidden: TextModels.Model` containing the foldable content
- a **right fold** — `leftSide = FALSE`, has no label and `hidden = NIL`

The "body" lives in one of two places depending on state:

- when **collapsed**, the body sits inside the left fold's `hidden` model and the parent text holds nothing between the two fold pieces
- when **expanded**, the body has been moved into the parent text between the two fold pieces and the left fold's `hidden` model is empty (but still present)

`StdFolds.Fold.Externalize` writes, in order:

| field            | type      | meaning                             |
|------------------|-----------|-------------------------------------|
| (super)          | View base | inherited Views.View fields         |
| version          | XInt      | 0 (`WriteXString`) or 1 (`WriteString`) for the label |
| sideMarker       | XInt      | `0` if `hidden # NIL`, else `1` — encodes leftSide |
| collapsedMarker  | XInt      | `0` if collapsed, else `1`          |
| label            | String    | `ARRAY 32 OF CHAR`, narrow or wide per version |
| hidden           | Store     | the `TextModels.Model` or `NIL`     |

The YAML supports two forms. Both must round-trip; converters write the lifted form by default and the pair form when round-trip fidelity to a binary edit cannot otherwise be preserved.

**Lifted form** (recommended for editing):

```yaml
- fold:
    label: "syntax"            # ARRAY 32 OF CHAR — UTF-8, ≤31 visible chars
    collapsed: true            # initial state, mirrors fold.collapsed
    body:                      # full piece flow, may itself contain folds
      - p: code
      - t: { c: code, text: "PROCEDURE Demo;" }
      - p: code
      - t: { c: code, text: "BEGIN ... END Demo;" }
```

When the converter writes binary from this, it emits a left-fold piece (with `body` packed into `hidden` if `collapsed: true`, or empty `hidden` if `collapsed: false`), the body pieces in the parent text (when not collapsed), and a matching right-fold piece. When reading, it pairs adjacent left/right folds and lifts the content from whichever side holds it.

**Pair form** (used when something between the two fold pieces cannot be lifted cleanly — for example, when an outer fold's pair brackets a region that already carries unrelated structure):

```yaml
- view:
    kind: StdFolds.Fold
    body: { side: left,  collapsed: true, label: "syntax",
            hidden: { ... full TextView body ... } }
- view:
    kind: StdFolds.Fold
    body: { side: right, collapsed: true }
```

The pair form is unambiguous and always-correct; the lifted form is shorter and hand-editable.

Notes:

- BlackBox uses fold version 0 for ASCII-only labels and version 1 for labels with non-ASCII characters. The YAML stores the label as a normal UTF-8 string and lets the converter pick the version on output.
- The `label` field is `ARRAY 32 OF CHAR` in the binary, so it is bounded to 31 characters plus terminator. The converter MUST validate.
- `collapsed` on the left and right fold are kept in sync by the runtime. The lifted form has one `collapsed:` value; the pair form has two and a converter rejects mismatches.
- A reader producing a "trimmed" projection of the document (just what BlackBox shows by default) emits the label runs of collapsed folds inline and skips their `body`. A reader producing a full projection emits both. Either way the fold structure is recoverable.

### A worked TextView example

A small fragment from the `Component Pascal Language Report` (`System/Docu/CP-Lang.odc`), with its leading section heading and one collapsible "syntax" fold:

```yaml
odc:
  format: 1
  source: System/Docu/CP-Lang.odc
  document:
    kind: Documents.StdDocument
    version: 0
    root:
      kind: TextViews.StdView
      version: 0
      body:
        defs:
          fonts:
            body:   { family: Arial, size: 10pt }
            bold:   { family: Arial, size: 10pt, weight: bold }
            italic: { family: Arial, size: 10pt, slant: italic }
            h1:     { family: Arial, size: 14pt, weight: bold }
            code:   { family: Courier, size: 10pt }
          chars:
            plain:  { font: body }
            em:     { font: italic }
            heading: { font: h1 }
            link:   { font: bold, underline: true, color: "#000080" }
            code:   { font: code }
          paras:
            body:    { left: 0,  align: left }
            indent1: { left: 36, align: left }
            code:    { left: 36, align: left, tabs: [36, 72, 108], font: code }
        flow:
          - p: body
          - target:
              name: "1"
              body:
                - t: { c: heading, text: "1. Introduction" }
          - p: body
          - t: "Component Pascal is "
          - t: { c: em, text: "Oberon microsystems'" }
          - t: " refinement of the Oberon-2 language."
          - p: indent1
          - t: { c: em, text: "Type extension" }
          - t: " makes Component Pascal an object-oriented language."
          - p: body
          - fold:
              collapsed: true
              label: "syntax"
              body:
                - p: code
                - t: { c: code, text: "Module = MODULE ident \";\" [ImportList] DeclSeq" }
                - p: code
                - t: { c: code, text: "         [BEGIN StatementSeq] [CLOSE StatementSeq] END ident \".\"." }
```

Note `t: "literal string"` is shorthand for `t: { c: <paragraph default>, text: "..." }`. This keeps prose paragraphs almost as readable as plain text.

A note on what the example demonstrates that the `review-md` rendering cannot:

- the section is anchored explicitly (`StdLinks.Target`), so cross-references resolve
- the heading carries a real heading character attribute, distinguishable from bold body text
- the syntax block is wrapped in a fold with a clickable label — readers see "syntax" as a link, click it to expand. The current markdown rendering instead pours the syntax inline on every page, which is the dominant reason the docs feel walls-of-text-y.

## Menu resource files — `Rsrc/Menus.odc`

These are *not* a separate view kind. Each menu resource is a `TextViews.StdView` whose text content is the menu source in BlackBox's `MENU "Name" ... END` grammar. The framework parses the text at load time. The YAML therefore stores it as a TextView like any other.

There are two acceptable serialization choices:

1. **Faithful TextView** (what a generic round-tripping converter produces). Every line of menu source becomes a paragraph, the `MENU`, `SEPARATOR`, `END` keywords get the bold character attribute they have in the IDE, item labels and command strings are plain runs. This preserves byte-level fidelity but is no easier to author than the original.

2. **Lifted-grammar shortcut** (recommended for hand-authored or hand-edited menu resources). The TextView body is replaced by a structured `menu:` element. A converter recognises this on output and emits the equivalent text-view binary; on input it reads either form.

   ```yaml
   odc:
     format: 1
     source: Com/Rsrc/Menus.odc
     document:
       kind: Documents.StdDocument
       root:
         kind: TextViews.StdView
         body:
           menu:
             - name: "COM"
               items:
                 - { label: "Show Error",      cmd: "DevComDebug.ShowError",           guard: "TextCmds.SelectionGuard" }
                 - { label: "Show Interfaces", cmd: "DevComDebug.ShowInterfaceRecords" }
                 - separator
                 - { label: "Interface Info",  cmd: "DevBrowser.ShowInterface('+!')",  guard: "TextCmds.SelectionGuard" }
                 - separator
                 - { label: "New GUID",        cmd: "DevComDebug.NewGuid" }
                 - separator
                 - { label: "Collect",         cmd: "HostMenus.Collect" }
                 - separator
                 - { label: "DTC Help",        cmd: "StdCmds.OpenBrowser('Docu/DTC-Help', 'Help Contents')" }
                 - { label: "DTC Examples",    cmd: "StdCmds.OpenBrowser('Com/Docu/Sys-Map', 'DTC Examples')" }
   ```

Field rules for the lifted form:

- `label` — displayed string. `&` marks the Alt-underline key, kept verbatim.
- `accel` — optional accelerator key (`"F5"`, `"Ctrl+S"`).
- `cmd` — BlackBox command string identical to the binary form.
- `guard` — optional guard procedure that enables/disables the item.
- `separator` — a bare scalar.
- A submenu is `{ label: "...", items: [...] }`, recursive.
- A context-restricted menu uses `context: "TextViews.View"` (or whatever view type), matching the binary form `MENU "Name" ("Module.View")`.

The lifted form is one-way clearly preferable for *editing*. A converter is responsible for translating between the two on save / load.

## Form / dialog resource files — `Rsrc/About.odc`, `Rsrc/Strings.odc`, etc.

These are also TextViews in the binary, not a distinct form-view kind. The form layout is achieved by embedding `Controls.PushButton`, `Controls.Caption`, `HostBitmaps.StdView`, `TextControllers.StdCtrl` and similar views as pieces inside the text flow, with paragraph rulers controlling positioning.

The schema for the bodies is therefore the same TextView schema, with extra `kind`s for the embeddable views. Common ones:

```yaml
- view:
    kind: Controls.PushButton
    body:
      label: "License…"
      cmd:   "StdCmds.OpenAuxDialog('System/Rsrc/About', 'License')"
      width: 60pt
      height: 14pt

- view:
    kind: Controls.Caption
    body:
      text:  "Version:"
      width: 40pt

- view:
    kind: TextControllers.StdCtrl   # an inline editable field
    body:
      link:  "Forms.GetText"
      width: 60pt

- view:
    kind: HostBitmaps.StdView
    body:
      format: bmp                   # bmp|emf|png — original BlackBox uses bmp/emf
      data:   !!binary |
        Qk0u...                     # base64 payload

- view:
    kind: StdHeaders.View
    body:
      cells:
        - { width: 80pt, label: "Name" }
        - { width: 60pt, label: "Type" }
        - { width: 40pt, label: "Size" }
```

The control `body` is exactly the set of fields the corresponding BlackBox view writes to its store, named directly. No further envelope is needed because all of these are leaf views — they don't contain a piece flow of their own.

Where a form contains a multi-line label, the label is itself a nested `TextViews.StdView` view envelope (full recursive schema), not a single `text:` string. This keeps font and ruler control available everywhere it can apply in the original.

## Source modules — `Mod/*.odc`

`Mod/*.odc` files are TextViews whose content happens to be Component Pascal source. They typically use only:

- one paragraph ruler (`code` with the Courier font and tab stops),
- two or three character attributes (plain, comment-italic, keyword-bold), and
- no embedded views except the occasional `Sym` link.

They could be flattened to `.cp` source files for normal editing, but the YAML form preserves the exact rendering BlackBox shows in its IDE — italic comments, bold keywords, hyperlinks in module headers — without committing to a colour theme. Tooling can re-derive a plain `.cp` file from the YAML by concatenating all `t:` text with newlines on each `p:` boundary.

## Unknown views

A converter that meets a view `kind` it has no schema for must not lose data. The fallback encoding is:

```yaml
- view:
    kind: SomeOldSubsystem.WeirdView
    version: 7
    body:
      raw: !!binary |
        Q0RPbwAAAAAAAA...     # original sub-stream bytes verbatim
```

This guarantees that any future tool can recognise and re-emit the original binary even when the YAML schema doesn't describe its shape. A document is allowed to mix `body:` shapes (structured for known kinds, `raw:` for unknown ones) freely.

## Round-trip and canonicalisation

The reference implementation achieves **byte-identical** `.odc → AST → .odc` round-trip across the full BlackBox 1.7 tree (Stage A and B; see [Implementation status](#implementation-status)). For YAML round-trip (`.odc → YAML → .odc`, Stage C), the same byte-identical guarantee is the goal — which means the YAML format must carry every byte the binary holds, including the parts that aren't human-edit-relevant.

The schema accommodates this in two layers:

1. **Lifted, human-editable form** for the parts that should round-trip via *meaning* (text content, link targets, fold labels, paragraph metrics, control fields, etc.) — these survive normal hand-edits and naturally re-encode.

2. **Round-trip metadata** for the bytes that *must* survive but shouldn't clutter the editable surface: the per-store `wire_path` encoding choice, captured `comment`/`raw_next`/`raw_down` envelope fields, super-class version bytes, the legacy `Controls.Control` v0–v2 auxiliary fields, view-piece placeholder bytes, run-list padding, declared-but-unused tab slots, and any subclass `Internalize2` trailing bytes for view kinds we don't fully decode. The convention: a single `_meta:` map per store containing whatever is needed for byte-identical writeback, ignored by readers that only want the editable content.

A canonical YAML writer should:

- emit `defs` pool entries in name-sorted order;
- coalesce adjacent text runs that share a `c:` value into one;
- collapse runs of identical paragraphs (don't repeat `p: body` if it's already active);
- prefer the shorthand `t: "..."` over `t: { c: <paragraph default>, text: "..." }`;
- emit anchors as inline `- a: name` pieces and rebuild the offset map on save.

These are output policy, not schema rules — a manually edited YAML file that doesn't follow them is still valid input.

A YAML editor can drop `_meta:` blocks before saving; the writer then loses byte-identity but preserves *semantic* identity (same View tree, same text, same attributes). This is the same trade-off as VCS auto-rebases or auto-formatters: the lifted content always survives, the wire-byte fidelity is best-effort. The CLI can offer both modes:

- `odc-yaml --check`: read-then-write, byte-identical comparison (already implemented)
- `odc-yaml --check-yaml`: read .odc → write YAML → parse YAML → write .odc, byte-identical comparison (Stage C target)
- `odc-yaml --check-semantic`: read .odc → write YAML → parse YAML → write .odc, semantic comparison only (compares decoded ASTs, not bytes — useful for files that have been hand-edited)

## What this format intentionally leaves out

- **Selection state, scroll position, view geometry on screen** — these are session state, not document state, and the binary format already keeps them out of the saved file.
- **Compiled symbol or object content** (`.osf`, `.ocf`) — those are separate binary formats with their own concerns; not all of it is text.
- **OLE compound storage** — `.odc` files can in principle embed COM objects via OLE; the YAML format treats them as opaque `!!binary` blobs and does not try to surface their internal structure.
- **Live computation in fields** — BlackBox `Form` controls bind to live data through `link:` strings; the YAML stores the link string only, not the runtime value.

## Open questions

1. **Inline vs. external `defs`.** A repo-wide `defs` palette referenced by `!include` would let every Mod file share one Courier/code definition. Useful, but adds a multi-file dependency that complicates tooling. Default: inline per document, accept a future `!include` extension.
2. **Whether to surface BlackBox's "lookup" attribute** (the runtime-defined character attribute that picks up its style from the host context). For now: model as `c: lookup` with no fields, and let the renderer resolve it at display time.
3. **Embedded `.cp` source representation.** Two reasonable choices for `Mod/*.odc`:
   - keep the full TextView faithfully (current proposal), or
   - special-case `Mod/*.odc` as `kind: SourceModule` whose `body` is a single literal block scalar of `.cp` source plus an optional `attrs:` overlay describing where keywords/comments/links lie.
   The second is much more readable for the common edit case, the first is more uniform. A pragmatic answer is to *accept both* — a converter writes whichever the source warrants and reads either.
4. **Versioning.** `format:` versions the YAML schema; per-view `version:` versions the binary view's own state. Keeping them separate means the schema can evolve without rewriting every file.

## What's done and what's left

Stages A and B are complete (see [Implementation status](#implementation-status)) — the binary side of the round-trip is byte-identical across the BlackBox 1.7 corpus, and every decoded view kind has a structured encoder.

Stage C (YAML round-trip) is the remaining work. It splits into:

1. **YAML parser**: parse a YAML document conforming to this schema back into the existing `Document` / `StoreNode` AST. Must handle both the lifted form (links/targets/folds/rulers as named blocks) and the structural form (raw `view: { kind: ... }` plus the `_meta:` round-trip metadata for currently-unknown kinds).
2. **`_meta` schema finalisation**: lock down which fields go in `_meta:` per store / view kind. The implementation already captures everything the writer needs — the schema just needs to surface them in YAML in a stable shape.
3. **`bin → YAML → bin` verification**: extend `odc-yaml --check` with `--check-yaml` mode that completes the loop. Sweep the full 675-file corpus the same way Stage A and B were validated.
4. **Lift menu resources**: turn `Rsrc/Menus.odc` from "TextView whose content happens to be menu source" into a structured `menu:` block with items / separators / submenus. Hand-authoring menus is the highest-leverage edit case.
5. **Lift bitmap resources**: surface `HostBitmaps.StdView` as a `!!binary` payload with a hint about the format, plus optional sidecar extraction (`.png`) for diff-ability.

A nice-to-have output mode, available once Stage C lands:

- `odc-yaml --prose <file>`: render just the text content of a Docu, with collapsed folds shown as `[label]` placeholders the user can click in HTML, links inlined as `[text](target)`, anchors retained. This is what makes the legacy BlackBox documentation actually browsable outside the IDE — and the YAML schema makes it a one-screen change once the parser is in place.

## Reference implementation

- Crate: [`NewCP/src/newcp-odc/`](../src/newcp-odc/)
- Library API: `read_document`, `write_document`, `check_roundtrip`, `document_to_yaml`, plus per-view `decode_*` / `encode_*` functions
- CLI: `cargo run -p newcp-odc --bin odc-yaml -- <args>` (or build and run `target/debug/odc-yaml`)
- Verification: `find <BlackBox>/ -name "*.odc" -exec odc-yaml --check {} \;` reports `ok` for all 675 files
