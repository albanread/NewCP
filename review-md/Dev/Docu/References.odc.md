**DevReferences**

DEFINITION DevReferences;

    IMPORT TextMappers, Files;

    PROCEDURE  ShowDocu;

    PROCEDURE  ShowSource;

    PROCEDURE  ShowText (module, ident: TextMappers.String; category: Files.Name);

END DevReferences.

This module provides two commands which, given the name of a module or of a qualified identifier, look up the corresponding documentation or source text.

Typical menu:

**MENU**

    "&Source"    ""    "DevReferences.ShowSource"    "TextCmds.SelectionGuard"

    "&Documentation"    ""    "DevReferences.ShowDocu"    "TextCmds.SelectionGuard"

**END**

PROCEDURE **ShowDocu**

Guard: TextCmds.SelectionGuard

Looks up the documentation text of the module whose name is selected. If a qualified identifier is selected, i.e., "module.ident", the corresponding item is searched. It must be written in boldface and in a smaller than 14 point type. The document must be located in the *Docu* directory of the module's subsystem.

PROCEDURE **ShowSource**

Guard: TextCmds.SelectionGuard

Looks up the source text of the module whose name is selected. If a qualified identifier is selected, i.e., "module.ident", the corresponding item is searched. It must be written in boldface and in a smaller than 14 point type. The document must be located in the *Mod* directory of the module's subsystem.

PROCEDURE  **ShowText** (module, ident: TextMappers.String; category: Files.Name)

Used internally.

