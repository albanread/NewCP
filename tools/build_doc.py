"""Build NewCP/docs/yaml_module_tree.md from analysis.json + provides.json."""
from __future__ import annotations
import json
from pathlib import Path

a = json.load(open('tools/analysis.json'))
p = json.load(open('tools/provides.json'))

ROLES = {
    # System
    'Kernel': 'GC, type/module registry, traps, low-level allocator, RTTI, memory-block descriptors',
    'Math': 'Real-number math (sin, cos, sqrt, exp, ln, ...) — leaf module',
    'SMath': 'SHORTREAL math counterpart — leaf module',
    'Files': 'Locator/File/Reader/Writer abstraction; persistent file handle, length, pos',
    'Log': 'Single global log buffer routed to a TextModel; string/int/real writers',
    'Dates': 'Date/time records, formatting, day-of-week',
    'Services': 'Action queue, timers, asynchronous callbacks; background task scheduling',
    'Strings': 'String <-> int/real conversions; substr/find/replace',
    'Integers': 'Big-integer arithmetic',
    'Meta': 'Runtime reflection: walk modules, types, fields, call procedures by name',
    'Dialog': 'Notifier protocol + global "interactor" record convention; modal dialogs facade',
    'Stores': 'Persistent object base type; externalize/internalize protocol; Reader/Writer',
    'Sequencers': 'Action sequence (undo/redo) bound to a Domain',
    'Models': 'Model base type — abstract document content; notifications + storage hooks',
    'Views': 'View base type — visual representation of a Model; Frame, Domain, message dispatch',
    'Controllers': 'Controller base type — input handling for a View; focus/selection/tracker',
    'Mechanisms': 'View mechanisms — drag-drop, keyboard accelerator infra',
    'Properties': 'Property protocol (Property record, ReadProp, SetProp, msgs); used by Views/Controls',
    'Containers': 'Composite View/Model/Controller bases for views that contain views (Form, Doc, Text)',
    'Documents': 'StdDocument/Model/View — top-level container view used as the file format',
    'Windows': 'Window list, document I/O glue, MDI window plumbing (host-independent)',
    'Controls': 'Embedded control surrogates: Button/CheckBox/ComboBox/Label etc. on a Form',
    'Printing': 'Page-layout / pagination / device coordinates for printing',
    'Printers': 'Printer abstraction (paper size, resolution, init/start/end)',
    'Ports': 'Frame ports — drawing surface abstraction (Rect, Frame, drawing primitives)',
    'Fonts': 'Font/Typeface metadata, font directory abstraction',
    'Converters': 'File-format converter registry (mime -> reader proc); Open dispatcher',
    'Config': 'INI-style config file reader/writer (uses OleData)',
    'Init': 'Main loop bootstrap; calls module body initializers, hooks main menu',
    'In':  'Tutorial-style stdin reader (Open/Int/Real/Char/Name) — wraps TextMappers',
    'Out': 'Tutorial-style stdout writer (Open/Int/Real/Ln) — writes to StdLog',
    'XYplane': 'Tutorial pixel surface (Wirth Programming In Oberon)',

    # Std
    'StdLog': 'Per-process Log View; opens the Log window the first time text is written',
    'StdCmds': 'Standard menu commands: New, Open, Save, Quit, Close, Cut/Copy/Paste',
    'StdLoader': 'Module loader fronted by Files (file -> module image)',
    'StdMenuTool': 'Menu definitions parser (text-based menu sources)',
    'StdInterpreter': 'Command-string interpreter ("Mod.Proc(arg, ...)" parser/dispatcher)',
    'StdCoder': 'Identifier hash/encode helpers used by the loader',
    'StdLinks': 'Hyperlink View — clickable link embedded in a text',
    'StdFolds': 'Foldable text region View',
    'StdHeaders': 'Page header/footer Setter for printed text',
    'StdStamps': 'Date/time/page stamp inline View',
    'StdTables': 'Embedded table View',
    'StdLogos': 'Decorative logo View used by About box',
    'StdClocks': 'Animated clock View (a Models/Views demo + real component)',
    'StdScrollers': 'Scroll-bar wrapped frame around any View',
    'StdViewSizer': 'Tracker that resizes an embedded View',
    'StdCFrames': 'Custom frame helpers — clipping, scrolling primitives',
    'StdTabViews': 'Tabbed-pane View',
    'StdDialog': 'Dialog box layout/run loop — built on Form + Containers',
    'StdApi': 'Programmatic API for opening windows/documents from other modules',
    'StdDebug': 'Debug command set (heap walk, dump module)',
    'StdETHConv': 'ETH Oberon -> BlackBox text converter',

    # Text
    'TextModels': 'Text Model: run-list of attributed runs, attr pool, char ops',
    'TextViews': 'Text View: rendering, layout, line break, ruler-aware',
    'TextControllers': 'Text Controller: caret, selection, keystroke handling',
    'TextRulers': 'Ruler model (margins, tab stops, line spacing)',
    'TextMappers': 'Scanner/Formatter on a Text Reader/Writer (whitespace-sep tokens)',
    'TextSetters': 'Pluggable layout/setting strategies (column, justified, ...)',
    'TextCmds': 'Text editing commands (cut/copy/paste/clear/find)',

    # Form
    'FormModels': 'Form Model: array of placed Views with positions',
    'FormViews': 'Form View: renders FormModel; grid/snap',
    'FormControllers': 'Form Controller: drag/resize/selection of embedded Views',
    'FormCmds': 'Form editing commands (align/distribute/lock)',
    'FormGen': 'Code generator: emit a CP module from a Form (interactor proc + decls)',

    # Host
    'HostFiles': 'Files implementation backed by Win32 file API',
    'HostFonts': 'Fonts implementation backed by Win32 GDI fonts',
    'HostPorts': 'Ports implementation backed by Win32 GDI device contexts',
    'HostWindows': 'Windows implementation: HWND wrappers, MDI integration',
    'HostMenus': 'Menu bar/menus integration',
    'HostClipboard': 'Clipboard exchange via WM_CLIPBOARD',
    'HostBitmaps': 'Bitmap loading/saving via GDI',
    'HostPictures': 'Picture (DIB/metafile) wrapper View',
    'HostRegistry': 'Win32 registry reader/writer used by config',
    'HostDialog': 'Win32 common dialog (file open/save, color, font)',
    'HostMail': 'MAPI mail integration',
    'HostMechanisms': 'Win-specific drag/drop, OLE drag-drop, keyboard accel',
    'HostPackedFiles': 'Read/write packed file archive',
    'HostPrinters': 'Win32 printer dialog + GDI printer DC',
    'HostCmds': 'Host-specific commands (registry browse, PrintScreen)',
    'HostTabFrames': 'Win32-tab-control backed StdTabViews variant',
    'HostTextConv': 'Text-encoding converters (Win-1252, UTF-8, etc.)',
    'HostCFrames': 'Win-specific custom-frame helpers',

    # Win FFI
    'WinApi': 'Win32 base API (kernel32+user32) — types, consts, FFI procs (no internal deps)',
    'WinOle': 'OLE/COM core API — IUnknown, GUIDs, HRESULT bindings',
    'WinOleAut': 'OLE Automation (IDispatch, VARIANT, BSTR, SAFEARRAY)',
    'WinOleCtl': 'OLE Controls (OCX) interfaces',
    'WinOleDlg': 'OLE common dialogs (insert object, paste special)',
    'WinCtl': 'Common Controls (toolbar, tabctrl, treeview)',
    'WinDlg': 'Common Dialogs (open/save/color/font)',
    'WinNet': 'WinSock / WNet network APIs',
    'WinMM': 'Multimedia (waveform/MIDI/timeSetEvent)',
    'WinRpc': 'MS-RPC bindings',
    'WinGL': 'OpenGL 1.1 bindings (no internal deps)',
    'WinGLAux': 'OpenGL Aux helpers',
    'WinGLUtil': 'GLU utilities',
    'WinSql': 'ODBC SQL.h type/const bindings (no internal deps)',
    'WinCmc': 'CMC (Common Mail Calls) bindings (no internal deps)',

    # Ole
    'OleClient': 'OLE client container View (embed external OLE objects)',
    'OleServer': 'OLE in-process server — expose BlackBox views as OLE',
    'OleData': 'OLE data exchange (IDataObject, format negotiation)',
    'OleStorage': 'IStorage/IStream wrapper for compound files',
    'OleViews':  'View glue for OLE-embedded content',

    # Comm
    'CommStreams': 'Generic byte-stream abstraction (read/write/seek/timeout)',
    'CommTCP': 'TCP client/server streams (over WinSock)',
    'CommV24': 'Serial port streams (RS-232 wrapper)',
}

ROLES_PER_SUB = {
    'System': 'Kernel, runtime services, model/view/controller bases, persistence',
    'Std':    'Standard Views, commands, dialogs — built on top of System',
    'Text':   'Text Model/View/Controller framework',
    'Form':   'Form Model/View/Controller — places Views on a 2D surface',
    'Host':   'Win32 implementation of host-abstract interfaces (Files, Fonts, Ports, Windows...)',
    'Win':    'Pure FFI bindings to Win32 / OLE / OpenGL — no logic',
    'Ole':    'OLE client/server framework (uses Win* and Host*)',
    'Comm':   'TCP / serial / generic stream interfaces',
    'Sql':    'SQL DB framework + ODBC driver + sample tools',
    'Ctl':    'COM type-library wrappers (Excel/Word/Outlook/etc.) — generated from typelibs',
    'Com':    'COM helpers (enumerators, demo phone-book server, Koala framework)',
    'Dev':    'Compiler/loader/browser/inspector/debugger — the IDE',
    'Obx':    'Examples (Oberon-by-example): demos, tutorials, ports of Wirth book code',
    'Xhtml':  'XHTML exporter',
}

out = []
W = out.append
W('# YAML-corpus module map: imports, exports, and porting order')
W('')
W('## Purpose')
W('')
W('This document inventories the 272 Component Pascal modules in the lifted `.odc` corpus')
W('under [`YAML/`](../../YAML), groups them by subsystem, captures the import graph, and')
W('proposes a porting order to NewCP. The data was generated by parsing every')
W('`*.odc.yaml` text run and extracting `MODULE` / `IMPORT` / `PROCEDURE*` / `TYPE*`')
W('declarations. Re-generate with [`tools/extract_imports.py`](../../tools/extract_imports.py),')
W('[`tools/extract_provides.py`](../../tools/extract_provides.py), and')
W('[`tools/analyze_imports.py`](../../tools/analyze_imports.py).')
W('')
W('Module-naming convention: with the exception of `System/Mod/*` (whose modules use bare')
W('names — `Kernel`, `Files`, `Stores`, `Views`, ...) every other subsystem prefixes its')
W('modules with the subsystem name. So `Std/Mod/Cmds.odc` declares `MODULE StdCmds;`,')
W('`Text/Mod/Views.odc` declares `MODULE TextViews;`, etc. The doc uses the *declared*')
W('module names everywhere — those are the names other modules import.')
W('')

W('## Module counts by subsystem')
W('')
W('| Subsystem | Modules | Role |')
W('|---|---:|---|')
counts = a['subsystem_counts']
for sub in sorted(counts, key=lambda s: -counts[s]):
    W(f"| `{sub}` | {counts[sub]} | {ROLES_PER_SUB.get(sub, '')} |")
W('')
W('Total: **272 modules**.')
W('')

W('## Subsystem-level dependency graph')
W('')
W('Edge `A -> B` means at least one module in `A` imports at least one module in `B`.')
W('')
W('```')
for s in sorted(a['subsystem_deps']):
    W(f"  {s:8s} -> {', '.join(a['subsystem_deps'][s])}")
W('```')
W('')
W('Notes:')
W('')
W('- `Text` depends only on `System`. **Text is the first non-System subsystem to port.**')
W('- `Form` depends only on `Std` and `System`. Port after Std.')
W('- `System` shows back-edges to `Text`, `Std`, `Host`, `Ole`, `Win` — these come from a')
W('  small set of *facade* / *tutorial* modules and are not part of the core dep chain:')
W('    - `Out`, `In` (tutorial wrappers; depend on `TextMappers` / `StdLog`)')
W('    - `XYplane` (Wirth-book pixel surface; depends on `HostPorts` / `HostWindows`)')
W('    - `Init` (depends on `HostMenus`)')
W('    - `Config` (depends on `OleData`)')
W('    - `Controls` (depends on `StdCFrames`)')
W('    - `Kernel` (depends on `WinApi`, `WinOle` — Win-specific bindings)')
W('  None of these are required to port the System core; they can be deferred or')
W('  re-stubbed in NewCP.')
W('')

W('## Highest fan-in modules (port these early)')
W('')
W('Number of other modules in the corpus that import each name:')
W('')
W('| Rank | Module | Fan-in | Role |')
W('|---:|---|---:|---|')
fanin_sorted = sorted(a['modules'].items(), key=lambda kv: -kv[1]['fanin_count'])
for rank, (n, m) in enumerate(fanin_sorted[:25], 1):
    W(f"| {rank} | `{n}` | {m['fanin_count']} | {ROLES.get(n, '')} |")
W('')
W('The top of this list is essentially the runtime/UI core: `Views`, `Dialog`, `Ports`,')
W('`Stores`, `Properties`, `TextModels`, `Controllers`, `TextViews`, `Models`, `Kernel`,')
W('`Fonts`, `Files`. Until those are available, very little of the corpus will load.')
W('')

W('## Topological layers (dependency tiers)')
W('')
W('A module appears in layer *k* iff all of its internal imports are in layers `< k`.')
W('Layer 0 has no internal imports and is safe to port first. The corpus has')
W(f'**{len(a["layers"])} layers**.')
W('')
LAYER_CAP = 16
for li, layer in enumerate(a['layers']):
    if li >= LAYER_CAP:
        rest = sum(len(l) for l in a['layers'][li:])
        W(f"### Layers {li}..{len(a['layers'])-1} ({rest} modules)")
        W('')
        W('Mostly higher-level demos (`Obx*`), generated COM wrappers (`Ctl*`), and')
        W('tools (`Dev*`). Listed in the per-subsystem appendix below; not on the')
        W('critical-path porting list.')
        W('')
        break
    W(f"### Layer {li}  ({len(layer)} modules)")
    W('')
    W('| Module | Subsystem | Internal imports | Role |')
    W('|---|---|---|---|')
    for n in layer:
        m = a['modules'][n]
        deps = ', '.join(f'`{x}`' for x in m['internal_imports']) or '—'
        role = ROLES.get(n, '')
        W(f"| `{n}` | {m['subsystem']} | {deps} | {role} |")
    W('')

W('## Recommended porting tiers')
W('')
W('Aligned with the existing roadmap (Phase 5 = "Runtime compatibility shell";')
W('Phase 7 = "Framework recovery"). Each tier is a coherent chunk that can be')
W('delivered together; later tiers cannot start until earlier tiers can compile')
W('and load.')
W('')
W('### Tier 0 — Runtime substrate (Rust first, then CP)')
W('')
W('Present as Rust services per the roadmap; each can be replaced by a CP module once')
W('the compiler is self-hosting. Layer 0..3 of the graph.')
W('')
W('| Module | Why first | Notes |')
W('|---|---|---|')
W('| `Kernel` | The runtime — every other module depends on it transitively | Currently Rust-resident. Replace last among Tier 0. |')
W('| `Math`, `SMath` | Leaf, no deps | Can come from a libm shim. |')
W('| `Files` | Persistence + Stores foundation | Will sit on `HostFiles` (Win32) or a new portable backend. |')
W('| `Log` | Single global text buffer; trivial dep on Kernel | Already exists in `NewCP/Mod/Log.cp` as a placeholder. |')
W('| `Dates` | Used by Stores/Sequencers logging | Pure CP. |')
W('| `Services` | Action queue, timers — used by Views | Needs a host event loop hook. |')
W('| `Strings` | String <-> int/real utility | Pure CP. |')
W('| `Meta` | Reflection over modules/types/procs | Reads module descriptors from `Kernel`. |')
W('| `Dialog` | Notifier protocol; everyone uses it | Modal facade can stub `Beep`/`MsgBox`. |')
W('')

def tier_table(title: str, names: list[str]):
    W(f'### {title}')
    W('')
    W('| Module | Provides |')
    W('|---|---|')
    for n in names:
        if n in ROLES:
            W(f"| `{n}` | {ROLES[n]} |")
    W('')

tier_table('Tier 1 — Persistence and the Model/View base',
    ['Stores','Sequencers','Models','Converters','Views','Controllers','Mechanisms','Properties','Containers'])
W('Layer 4..11 of the graph. Once these land you can read/write `.odc` documents.')
W('')

tier_table('Tier 2 — Drawing and host integration',
    ['Fonts','Ports','Printers','Printing','HostFiles','HostFonts','HostPorts','HostRegistry','HostMechanisms','HostWindows','HostClipboard','HostMenus','HostDialog','HostBitmaps','HostPictures'])
W('In NewCP these become the iGui layer (per [`igui_design.md`](igui_design.md)) — the')
W('Rust side replaces the `Host*` modules with native GTK/whatever, but exposes the same')
W('`Ports`/`Windows` interface to CP modules above.')
W('')

tier_table('Tier 3 — Document framework',
    ['Documents','Windows','Init','Controls'])

tier_table('Tier 4 — Text framework (Text/*)',
    ['TextModels','TextRulers','TextSetters','TextViews','TextControllers','TextMappers','TextCmds'])

tier_table('Tier 5 — Standard commands and controls (Std/*)',
    ['StdLog','StdCmds','StdLoader','StdMenuTool','StdInterpreter','StdLinks','StdFolds','StdScrollers','StdCFrames','StdViewSizer','StdTabViews','StdDialog','StdApi','StdLogos','StdClocks','StdHeaders','StdStamps','StdTables','StdDebug','StdETHConv','StdCoder'])

tier_table('Tier 6 — Form framework (Form/*)',
    ['FormModels','FormViews','FormControllers','FormCmds','FormGen'])

W('### Tier 7 — Optional / per-need')
W('')
W('Everything else: `Comm`, `Ole`, `Sql`, `Ctl`, `Com`, `Win` (beyond `WinApi`/`WinOle`),')
W('`Dev`, `Obx`, `Xhtml`. None of these are on the critical path for self-hosting; port')
W('what you need:')
W('')
W('- `Win*` FFI modules — port any one as soon as a CP module needs that API. `WinApi`')
W('  and `WinOle` are leaf modules and have no deps; they are pure declaration files.')
W('- `Dev*` — IDE tools (Compiler/Linker/Browser/Debugger). These are *replaced* by')
W("  NewCP's Rust toolchain — do not port literally.")
W('- `Obx*` — examples; convert opportunistically as integration tests.')
W('- `Sql`, `Ole`, `Ctl`, `Com`, `Comm`, `Xhtml` — port only when a target app needs them.')
W('')

W('## Appendix: per-subsystem module listing')
W('')
W('Each row shows the declared module name, fan-in (how many other modules import it),')
W('exported-procedure count, exported-type count, and its internal imports. External')
W('imports (`SYSTEM`, `COM`, and DLL bindings) are omitted from this column.')
W('')
groups: dict[str, list] = {}
for n, m in a['modules'].items():
    groups.setdefault(m['subsystem'], []).append((n, m))
for sub in sorted(groups):
    W(f"### `{sub}/`")
    W('')
    W('| Module | Fan-in | Exports | Internal imports |')
    W('|---|---:|---|---|')
    for n, m in sorted(groups[sub]):
        key = f"{m['subsystem']}/{m['file']}"
        pp = p[key]
        deps = ', '.join(f'`{x}`' for x in m['internal_imports']) or '—'
        exp = f"{len(pp['exported_procedures'])} procs, {len(pp['exported_types'])} types"
        W(f"| `{n}` | {m['fanin_count']} | {exp} | {deps} |")
    W('')

W('## Methodology and caveats')
W('')
W('- The text content of each `.odc.yaml` is reassembled by concatenating every')
W('  `- text:` run in document order. This recovers the source faithfully because')
W('  the YAML reflects the BlackBox text-run model. Inline non-text Views (folds,')
W('  hyperlinks, etc.) are skipped — they are decorations, not source.')
W('- IMPORT clauses are matched as `IMPORT ... ;` non-greedily; both `Name` and')
W('  `Alias := Name` forms are recovered. `(* ... *)` comments inside the IMPORT')
W('  clause are stripped before parsing.')
W('- Exports are counted by regex (`PROCEDURE ... Name*` and `Name* = ...`). The')
W('  counts are approximate but useful for spotting which modules are weighty')
W('  (`Kernel` 89 procs / 17 types; `Views` 70 procs / 14 types).')
W('- A handful of leaf modules really do have no IMPORT clause at all (e.g.')
W('  `Fonts`, `WinGL`, `WinSql`, `WinCmc`, `ObxRandom`). They appear in Layer 0.')
W('- The corpus is BlackBox 1.7. If you target a different release, regenerate')
W('  the YAML (see [`YAML/README.md`](../../YAML/README.md)) before re-running')
W('  the analyzers.')

p_out = Path('NewCP/docs/yaml_module_tree.md')
p_out.write_text('\n'.join(out) + '\n', encoding='utf-8')
print(f"wrote {p_out} ({len(out)} lines)")
