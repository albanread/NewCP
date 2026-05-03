**DevCompiler**

DEFINITION DevCompiler;

    PROCEDURE Compile;

    PROCEDURE CompileAndUnload;

    PROCEDURE CompileModuleList;

    PROCEDURE CompileSelection;

    PROCEDURE CompileThis;

    ... plus some private commands ...

END DevCompiler.

Command package for the Component Pascal compiler. The compiler has no compiler options. Safety-critical runtime checks are always performed (type guards, array range checks, etc.), while non-critical runtime checks may not be generated (SHORT, integer overflows, testing of set membership). "Critical" means that non-local memory may be destroyed, with unknown global effects.

Typical menu:

**MENU**

    "&Compile"    ""    "DevCompiler.Compile"    "TextCmds.FocusGuard"

    "Compile and Unload"    ""    "DevCompiler.CompileAndUnload"    "TextCmds.FocusGuard"

    "Compile &Selection"    ""    "DevCompiler.CompileSelection"    "TextCmds.SelectionGuard"

    "Com&pile Module List"    ""    "DevCompiler.CompileModuleList"    "TextCmds.SelectionGuard"

**END**

PROCEDURE **Compile**

Guard: TextCmds.FocusGuard

Compile the Component Pascal module whose source is in the focus view.

PROCEDURE **CompileAndUnload**

Guard: TextCmds.FocusGuard

Compile the module whose source is in the focus view. If compilation is successful, it is attempted to unload the old version of this module. *CompileAndUnload* is convenient when developing top-level modules, i.e., modules which are not imported by any other modules, and thus can be unloaded individually.

PROCEDURE **CompileModuleList**

Guard: TextCmds.SelectionGuard

Compile the list of modules whose names are selected. When the first error is detected, the offending source is opened to show the error.

PROCEDURE **CompileSelection**

Guard: TextCmds.SelectionGuard

Compile the module, whose beginning is selected.

PROCEDURE **CompileThis**

Used in a text with a *DevCommanders.View*. This command takes the text following it and interprets it as a list of modules that should be compiled. Similar to *CompileModuleList* except that no selection is necessary.

... plus some private commands ...

