# vscode-newcp

Component Pascal (`.cp`) syntax highlighting for VS Code, tuned for the NewCP project.

## What it highlights

- Nestable block comments `(* ... *)`
- String literals `"..."` and `'...'`
- Character literals `0FFX`, hex integers `0FFFFH`, real literals `1.5E-3`
- Component Pascal keywords (`MODULE`, `BEGIN`, `END`, `PROCEDURE`, `IMPORT`, `RECORD`, `POINTER`, `LOOP`, etc.)
- Built-in types (`INTEGER`, `REAL`, `CHAR`, `SHORTCHAR`, `INTSHORT`, `BOOLEAN`, `SET`, `ANYREC`, `ANYPTR`, ...)
- Built-in procs/functions (`NEW`, `LEN`, `INC`, `DEC`, `ASSERT`, `HALT`, `ORD`, `CHR`, ...)
- Language constants (`TRUE`, `FALSE`, `NIL`, `INF`)
- Module names after `MODULE`/`DEFINITION MODULE`
- Procedure names and the `*`/`-` export markers
- Operators `:=`, `..`, `#`, `<=`, `>=`, etc.

## Install (development)

From this directory:

```powershell
# 1. One-time: install vsce if you don't have it
npm install -g @vscode/vsce

# 2. Package the extension
vsce package

# 3. Install the resulting .vsix
code --install-extension vscode-newcp-0.1.0.vsix
```

Or, for fastest iteration without packaging, link this folder into the
VS Code extensions directory. PowerShell (no admin needed — uses an
NTFS junction):

```powershell
New-Item -ItemType Junction `
  -Path  (Join-Path $env:USERPROFILE ".vscode\extensions\vscode-newcp-0.1.0") `
  -Target "E:\NewCP\tools\vscode-newcp"
```

(`mklink` only works in `cmd.exe`; `New-Item -ItemType SymbolicLink`
works in PowerShell but needs admin or Windows Developer Mode. Junctions
work for directories without either.)

Then reload VS Code (`Ctrl+Shift+P` → "Developer: Reload Window").

## Files

- `package.json` — extension manifest, registers the `newcp` language for `.cp` and `.Mod`
- `language-configuration.json` — comments, brackets, auto-closing, indent rules
- `syntaxes/newcp.tmLanguage.json` — TextMate grammar (the highlighter itself)

## Tweaking colours

Most TextMate scopes used here (`keyword.control`, `support.type.primitive`,
`entity.name.function`, `storage.modifier.export`, `comment.block`,
`string.quoted`, `constant.numeric`, `constant.character`) are themed by every
mainstream VS Code theme. To recolour a specific scope, add a
`editor.tokenColorCustomizations` entry to your user settings, e.g.:

```json
"editor.tokenColorCustomizations": {
  "textMateRules": [
    {
      "scope": "storage.modifier.export.newcp",
      "settings": { "foreground": "#E5C07B", "fontStyle": "bold" }
    }
  ]
}
```
