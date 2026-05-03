**DevBrowser**

DEFINITION DevBrowser;

    PROCEDURE ImportSymFile (f: Files.File; OUT s: Stores.Store);

    PROCEDURE ShowInterface (opts: ARRAY OF CHAR);

    PROCEDURE ImportCodeFile (f: Files.File; OUT s: Stores.Store);

    PROCEDURE ShowCodeFile;

END DevBrowser.

The browser shows the interface of a module or of an item in a module. It extract the necessary interface information out of the module's symbol file. A symbol file contains only minimal information, it doesn't contain comments nor does it retain information about the textual order in which a module's item have been defined (their display is sorted alphabetically). For records, the browser only shows newly introduced fields and procedures (or procedures redefined with covariant function results). You can follow the record type hiearchy to the base type by applying the browser command again on the base type name in the record declaration.

The browser is also able to decode a code file and to display important information about the code file. In particular, the imported items (types, procedures, etc.) are shown to indicate on which features provided by other modules a given module depends (sometimes called the "required interface" of a module, in contrast to the "provided interface", i.e., the exported functionality of the module).

Both browser functions are available also as importers (-> Converters), which can be installed with the following statements in *Config.Startup*:

Windows:

    Converters.Register("DevBrowser.ImportSymFile", "", "TextViews.View", "OSF", {});

    Converters.Register("DevBrowser.ImportCodeFile", "", "TextViews.View", "OCF", {});

Mac OS:

    Converters.Register("DevBrowser.ImportSymFile", "", "TextViews.View", "oSYM", {});

    Converters.Register("DevBrowser.ImportCodeFile", "", "TextViews.View", "oOBJ", {});

Possible menu:

**MENU**

    "&Interface"    ""    "DevBrowser.ShowInterface('')"    "TextCmds.SelectionGuard"

    "&Flat Interface"    ""    "DevBrowser.ShowInterface('!')"    "TextCmds.SelectionGuard"

    "Import Interface"    ""    "DevBrowser.ShowCodeFile"    "TextCmds.SelectionGuard"

**END**

PROCEDURE **ImportSymFile** (f: Files.File; OUT s: Stores.Store)

This procedure is installed upon startup of BlackBox as an importer for symbol files (-> Converters). The importer converts the symbol file into a textual browser output.

PROCEDURE **ShowInterface** (opts: ARRAY OF CHAR)

Guard: TextCmds.SelectionGuard

If a module name is selected, this command shows the complete definition of the module. If a qualident is selected, only the definition of the corresponding item is shown.

*opts = ""* creates an output which only shows the newly introduced extensions in the case of record types.

*opts = "*!*"* creates an output which also shows the inherited base type features of record types ("flat interface").

*opts = "+*" creates an output which shows some additional low-level information useful for a compiler developer (not further documented).

*opts = "/*" creates an output which is formatted in a special way. Inofficially known as the "Dijkstra option".

*opts* = "&" creates an output which also shows hooks in the interface.

*opts* = "@" creates an output using the settings from the interface browser dialog.

*opts* = "c" creates an output which shows only the items being usable in client modules.

*opts* = "e" creates an output which shows only the items being extensible.

Several options may be combined.

PROCEDURE **ImportCodeFile** (f: Files.File; OUT s: Stores.Store)

This procedure is installed upon startup of BlackBox as an importer for code files (-> Converters). The importer converts the code file into a textual browser output.

PROCEDURE **ShowCodeFile**

Guard: TextCmds.SelectionGuard

If a module name is selected, this command shows some information about the code file of the compiled module. If a qualident is selected, only the definition of the corresponding item is shown.

