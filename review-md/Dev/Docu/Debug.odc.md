**DevDebug**

DEFINITION DevDebug;

    IMPORT Views, Kernel;

    PROCEDURE Execute;

    PROCEDURE ShowGlobalVariables;

    PROCEDURE ShowLoadedModules;

    PROCEDURE ShowViewState;

    PROCEDURE Unload;

    PROCEDURE UnloadModuleList;

    PROCEDURE UnloadThis;

    PROCEDURE HeapRefView (adr: INTEGER; name: ARRAY OF CHAR): Views.View;

    PROCEDURE ShowHeapObject (adr: INTEGER; title: ARRAY OF CHAR);

    PROCEDURE SourcePos (mod: Kernel.Module; codePos: INTEGER): INTEGER;

    PROCEDURE UpdateGlobals (name: ARRAY OF CHAR);

    PROCEDURE UpdateModules;

END DevDebug.

When a run-time error occurs, e.g., a division by zero, a "trap text" is opened which displays the procedure call chain. It is possible to use this text to navigate through data structures. Several other commands are useful when analyzing the state of the system.

Note that debugging is done completely within BlackBox; there is no separate debugger environment. Debugging occurs "post mortem" with respect to commands, i.e., a command produces a run-time error, is aborted, and then debugged. However, the run-time error usually does not affect the loaded modules and the data structures which are anchored there, nor the open documents. In other words, debugging occurs "run-time" with respect to the application as a whole.

A possible menu:

**MENU**

    "&Loaded Modules"    ""    "DevDebug.ShowLoadedModules"    ""

    "&Global Variables"    ""    "DevDebug.ShowGlobalVariables"    "TextCmds.SelectionGuard"

    "&View State"    ""    "DevDebug.ShowViewState"    "StdCmds.SingletonGuard"

    "E&xecute"    ""    "DevDebug.Execute"    "TextCmds.SelectionGuard"

    "&Unload"    ""    "DevDebug.Unload"    "TextCmds.FocusGuard"

    "Unloa&d Module List"    ""    "DevDebug.UnloadModuleList"    "TextCmds.SelectionGuard"

**END**

PROCEDURE **Execute**

Guard: TextCmds.SelectionGuard

Execute the string (between quotation marks), which must have the form of a Component Pascal command sequence, e.g., "Dialog.Beep; Dialog.Beep". For simple commands, the string delimiters may be omitted, e.g., Dialog.Beep (-> StdInterpreter).

PROCEDURE **ShowGlobalVariables**

Guard: TextCmds.SelectionGuard

Show the global variables of the module whose name is selected.

PROCEDURE **ShowLoadedModules**

Show the list of all loaded modules. This command can be convenient to determine the modules which should be linked together when building an application.

PROCEDURE **ShowViewState**

Guard: TextCmds.FocusGuard

Show the state of the current focus view.

PROCEDURE **Unload**

Guard: TextCmds.FocusGuard

Tries to unload the module whose source is in the focus view. Unloading fails if the specified module is not loaded yet, or if it is not a top module.

PROCEDURE **UnloadModuleList**

Guard: TextCmds.SelectionGuard

Tries to unload a list of modules whose names are selected. Unloading may partially or completely fail if one of the specified modules is not loaded yet, or if it is still being importet by at least one client module. Modules must be unloaded from top to bottom.

PROCEDURE **UnloadThis**

Used in a text with a *DevCommanders.View*. This command takes the text following it and interprets it as a list of modules that should be unloaded. Similar to *UnloadModuleList* except that no selection is necessary.

PROCEDURE **HeapRefView** (adr: INTEGER; name: ARRAY OF CHAR): Views.View

PROCEDURE **ShowHeapObject** (adr: INTEGER; title: ARRAY OF CHAR)

PROCEDURE **SourcePos** (mod: Kernel.Module; codePos: INTEGER): INTEGER

PROCEDURE **UpdateGlobals** (name: ARRAY OF CHAR)

PROCEDURE **UpdateModules**

These procedures are used internally.

