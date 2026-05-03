**DevCmds**

DEFINITION DevCmds;

    IMPORT Dialog;

    PROCEDURE FlushResources;

    PROCEDURE OpenModuleList;

    PROCEDURE OpenFileList;

    PROCEDURE RevalidateViewGuard (VAR par: Dialog.Par);

    PROCEDURE RevalidateView;

    PROCEDURE SetCancelButton;

    PROCEDURE SetDefaultButton;

    PROCEDURE ShowControlList;

END DevCmds.

Possible menu:

**MENU**

    "&Open Module List"    ""    "DevCmds.OpenModuleList"    "TextCmds.SelectionGuard"

    "Open &File List"    ""    "DevCmds.OpenFileList"    "TextCmds.SelectionGuard"

    "Flus&h Resources"    ""    "DevCmds.FlushResources"    ""

    "Re&validate View"    ""    "DevCmds.RevalidateView"    "DevCmds.RevalidateViewGuard"

    **SEPARATOR**

    "&Control List"    ""    "DevCmds.ShowControlList"    "StdCmds.ContainerGuard"

    "Set &Default Button"    ""    "DevCmds.SetDefaultButton"    "StdCmds.ContainerGuard"

    "Set Canc&el Button"    ""    "DevCmds.SetCancelButton"    "StdCmds.ContainerGuard"

**END**

PROCEDURE **FlushResources**

Flush menu guard and string translation resources. This command is needed when a menu item is disabled because its code could not be executed, but since then the code has become executable (e.g., due to a recompilation of a module, and possibly unloading an old module version). It is also needed if the string resource texts have been edited and saved, if the changes should become effective immediately, rather than only when BlackBox is started the next time.

PROCEDURE **OpenModuleList**

Guard: TextCmds.SelectionGuard

Opens the modules whose names are selected, e.g.,

"FormModels FormViews".

PROCEDURE **OpenFileList**

Guard: TextCmds.SelectionGuard

Opens the files whose names are selected. The names must use the portable path name syntax, e.g.,

"Form/Mod/Models Form/Mod/Views".

PROCEDURE **RevalidateViewGuard** (VAR par: Dialog.Par)

Menu guard for *Revalidate* View.

PROCEDURE **RevalidateView**

Guard: RevalidateViewGuard

A view may have become invalid after a trap occurred. As a result, the view is greyed out and made passive (so that the same trap cannot occur anymore, thus preventing trap avalanches). However, sometimes it can be useful to revalidate the view and continue to work with it.

PROCEDURE **SetCancelButton**

Guard: StdCmds.ContainerGuard

Makes the selected view in the focus container a "cancel" button, i.e., in mask mode (-> Containers) it is considered pressed when the user enters the escape character.

PROCEDURE **SetDefaultButton**

Guard: StdCmds.ContainerGuard

Makes the selected view in the focus container a "default" button, i.e., in mask mode (-> Containers) it is considered pressed when the user enters a carriage return character.

PROCEDURE **ShowControlList**

Guard: StdCmds.ContainerGuard

Lists the properties of all views in the focus container which return control properties (-> Controls).

