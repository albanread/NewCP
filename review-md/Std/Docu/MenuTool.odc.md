**StdMenuTool**

DEFINITION StdMenuTool;

    IMPORT TextModels;

    PROCEDURE UpdateFromText (text: TextModels.Model);

    PROCEDURE UpdateMenus;

    PROCEDURE UpdateAllMenus;

    PROCEDURE ListAllMenus;

    PROCEDURE ThisMenu;

END StdMenuTool.

The menu tool is used during startup of BlackBox to set up the menus.

Typical menu entries:

**MENU**

    "&Menus"    ""    "StdCmds.OpenDoc('System/Rsrc/Menus')"    ""

    "&All Menus"    ""    "StdMenuTool.ListAllMenus"    ""

    "&Update Menus"    ""    "StdMenuTool.UpdateMenus"    "TextCmds.FocusGuard"

    "U&pdate All Menus"    ""    "StdMenuTool.UpdateAllMenus"    ""

**END**

PROCEDURE **UpdateFromText** (text: TextModels.Model)

Clears all menus and builds up a new menu bar according to the menu specification in *text*.

PROCEDURE **UpdateMenus**

Guard: TextCmds.FocusGuard

Clears all menus and builds up a new menu bar according to the new menu specification in the focus text.

PROCEDURE **UpdateAllMenus**

Clears all menus and builds up a new menu bar according to the menu specification in *System/Rsrc/Menus*.

PROCEDURE **ListAllMenus**

Builds up a text with hyperlinks to all available menu configuration texts, i.e., each subsystem's */Rsrc/Menus* text.

PROCEDURE **ThisMenu**

Used internally (in the hyperlinks of the INCLUDE statements).

