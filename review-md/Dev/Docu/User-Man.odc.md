**Dev Subsystem**

**User Manual**

**Contents**

[<u>1 Compiling Component Pascal modules</u>](#Compiling Oberon)

[<u>2 Browsing tools</u>](#Browsing Tools)

[<u>3 Loading and unloading modules</u>](#Loading and)

[<u>4 Executing commands</u>](#Executing Commands)

[<u>5 Debugging</u>](#Debugging)

[<u>6 Deployment</u>](#Distribution)

[<u>7 Cross-Platform issues</u>](#Cross-Platform)

<a id="Compiling Oberon"></a>**1 Compiling Component Pascal modules**

Besides consuming stored text documents, the Component Pascal compiler can compile modules from anywhere in any displayed text document. If the beginning of a displayed text is also the beginning of a module, the command *Dev->Compile* is used to compile the module. If the module begins somewhere in the middle of a displayed text, its beginning can be selected, e.g., by double-clicking on the keyword *MODULE*, and then the command *Dev->Compile Selection* is used.

To compile a list of modules at once, a list of module names needs to appear in some displayed text, e.g.,

 FormModels FormViews FormControllers FormCmds

By selecting the part of the list that should be considered by the compiler, and by invoking the command *Dev->Compile Module List*, the list of modules is compiled consecutively. The process stops as soon as an erroneous module has been encountered. The compiler reports on the success or failure of compilations by writing into the system log. The log is a special text displayed in an auxiliary window. It can be opened using *Info->Open Log* and cleared using *Info->Clear Log*.

The log is a development tool, i.e., it is mainly used for debugging purposes and for development tools. End-user applications are not expected to display a log. Whether or not a log window is opened upon startup of BlackBox is determined by the configuration's *Config* module (in directory System/Mod). This module can be changed by the programmer as desired; by default its *Setup* procedure only contains a statement which opens the log.

*Dev->Open Module List* is a convenient command which opens the module sources of one or several modules: select a module name, e.g., *FormCmds*, and then execute *Dev->Open Module List*. This command can save you much time when you work with multiple subsystems (-> 7.2 Modules and Subsystems) at the same time.

When compiling a newly created or a modified module, direct compilation of the displayed program text with *Dev->Compile* is recommended. In addition to writing to the system log, the compiler then also inserts *error markers* into the source text and places the caret after the first marker, if some syntax error was encountered. Each marker represents an error flagged by the compiler in the source text. Normally, a marker is displayed as a crossed-out box. However, simply by clicking into it, a marker expands to display the corresponding error message. If the insertion point is directly behind a marker, it can be expanded via the command *Dev->Toggle Error Mark* (or more likely via its keyboard shortcut).

Windows:

At the bottom of a window there is a status bar. If the user clicks into an error marker, the error message is written into the status bar instead of expanding the error marker. The error marker can be forced to expand by a *modifier*-click or a double-click on the marker.

The command *Dev->Next Error* can be used to skip forward to the next marker. *Dev->Unmark Errors* removes all remaining markers from a text. However, this command is rarely required: The compiler automatically removes all old markers when recompiling a text; and when storing a text, contained markers are filtered out, i.e., do not appear anymore when the text is opened again.

If a compiled module contains one or several errors, the text is scrolled to the first one. If this doesn't happen, the module was successfully compiled. In addition to this feedback, the compiler writes the number of errors found to the log, if there are errors. If the module interface has changed compared to a previous version, the changes are listed in the log as well.

The successful compilation of a module yields two files: a symbol file and a code file. A symbol file contains the information about a module's interface, and is used by the compiler to check imported modules for consistency. The code file represents the generated code (Intel 386 code on Windows, Motorola 68020 code on Macintosh). The contents of a code file is linked and loaded dynamically when needed, thus there is no need for a separate linker.

Symbol files are only needed during development (used by the compiler and the interface browser), they are not needed to run modules. A symbol file is a special encoding of a module's interface, i.e. of its exported constants, variables, types, and procedures. If the interface of a module is changed and the module recompiled, a new symbol file is written to disk. When compiling a module, the compiler reads the symbol files of all imported modules. This information is used to generate code, but also to type-check the correct use of imported identifiers.

After the interface of a module has been modified, all modules importing it (i.e., all its *clients*) must be recompiled. Only those modules need to be recompiled which actually use a feature that has been changed. A mere addition of features does not invalidate the clients of a module: if you export a further procedure, for example, no clients need to be recompiled. This also holds for constants, variables, and types, but not for methods. Addition of a method is not considered an extension, but rather a modification of an interface, and thus may invalidate clients.

Code files are produced, but never read during compilation. They are necessary to run modules, i.e. the dynamic (linking) loader must be able to read them. They can be regarded as special very light-weight DLLs (dynamic link libraries).

See also modules [<u>DevCompiler</u>](Compiler.odc.md), [<u>DevCmds</u>](Cmds.odc.md), and [<u>DevMarkers</u>](Markers.odc.md).

<a id="Browsing Tools"></a>**2 Browsing tools**

A common cause of compile-time errors is the wrong use of interfaces. To quickly retrieve the actual definitions of items exported by modules, the browser may be used. The interface of a whole module is displayed when selecting the name of a module and executing *Info->Client Interface*. To display the definition of an individual item, e.g., a type or procedure, the qualified name of that item, i.e., *module.item* should be selected with the same command. The browser displays its output in a form that can directly be used as input for further browsing actions. Command *Info->Client Interface* only shows the part of a module's interface which is relevant for clients, i.e., it leaves out implement-only methods and similar items that are only relevant for implementers of object types. For implementers, the command *Info->Extension Interface* shows the full interface.

Two related commands, *Info->Source* and *Info->Documentation*, allow to look up an item's definition in a source file, or in an on-line documentation, respectively. They both work on selected module names as well as on names in the form *module.item*, just as the *Info->Interface* command.

The general text search commands *Info->Search in Sources* and *Info->Search in Docu* search a selected string in the available source files, respectively documentation files. For source files, this means in the global *Mod* directory and in each subsystems' *Mod* directory. For documentation files, this means in the global *Docu* directory and in each subsystem's *Docu* directory. As a result of these commands, a new text is opened which contains a hyperlink for each file where the string occurs, and the count of how many times it occurs there. When the user clicks on the hyperlink, the corresponding file is opened and first instance of the string is selected. With *Text->Find Again*, the next instance can be found. Note that these commands may take up to several minutes to complete.

See also modules [<u>DevBrowser</u>](Browser.odc.md), [<u>DevReferences</u>](References.odc.md), [<u>DevSearch</u>](Search.odc.md), and [<u>DevDependencies</u>](Dependencies.odc.md).

<a id="Loading and"></a>**3 Loading and unloading modules**

Once a module successfully passes the compiler it can be loaded to the system and used.

The BlackBox loader can dynamically link and load a module at run-time. A code file basically is a language-specific light-weight DLL (dynamic link library). The loader can be invoked from within a program by calling the procedure *Dialog.Call*. This procedure takes a command name as parameter. It loads the command's module, if it isn't loaded yet. Afterwards, it executes the command's procedure. For example,

    Dialog.Call("DevDebug.ShowLoadedModules", "", res)

causes the loading of module *DevDebug* (if necessary) and the execution of its *ShowLoadedModules* procedure.

A loading attempt may cause the following errors:

*Code file not found*

The code file of a module has not been found.

For example, loading of module *FormCmds* requires the code file *Form/Code/Cmds*.

*Corrupted code file*

The module's code file exists, but its internal format is not correct.

*Object not found*

The module imports some object (constant, type, variable, procedure) from another module, where this object is not exported.

*Object inconsistently imported*

The module imports some object (constant, type, variable, procedure) from another module, where this object is exported, but with a different signature.

Such inconsistencies typically occur if a module's interface was changed (e.g., a procedure got an additional parameter), but not all client modules were recompiled afterwards. Another possibility is that the module whose interface was changed has not been reloaded after compilation. In this case, everything will work fine after it has been unloaded (e.g., after a restart of BlackBox).

*Cyclic import*

Modules may not import each other cyclically. For example, it is not alllowed that module A imports module B, and module B imports module C, and module C imports module A.

*Not enough memory*

There is not enough memory left for loading the module.

Once loaded, a module remains loaded if it isn't unloaded explicitly. The list of all currently loaded modules is displayed by *Info->Loaded Modules*.

*In order to use a new version of a module that is already loaded, the module needs to be unloaded first.*

Command *Dev->Unload* takes a focused module source as input and tries to unload the corresponding module. *Dev->Unload* fails if the module is still in use, i.e., if it is imported by at least one other module that is still loaded. (In general, modules can only be released top down.) The command *Dev->Unload Module List* takes a selection as input, which must consist of a sequence of module names. You may directly use a selection in the text produced by *Info->Loaded Modules* as input.

See also module [<u>DevDebug</u>](Debug.odc.md).

<a id="Executing Commands"></a>**4 Executing commands**

There are several ways to execute commands (i.e., procedures exported by Component Pascal modules, intended for direct invocation by the user) within BlackBox. A command name can be written to a text, as a string of the form "module.procedure", selected, and executed using *Dev->Execute*. An easier way is to insert a *commander* in front of the command name, using *Tools->Insert Commander*. Clicking on a commander interprets the string following it as a sequence of commands, e.g.

  "Dialog.Beep; DevDebug.ShowLoadedModules"

If the string consists of only one parameterless command, the string delimiters may be omitted, e.g.,

  Dialog.Beep

With such simple commands, it is possible to combine the unloading of an old version with the execution of a new version of a command (resp. of its module): *modifier*-click on the commander causes the command's module to be unloaded, and then the new module version is loaded and the procedure executed. Note that this "unload shortcut" only works for top-level modules, i.e. for modules which are not imported by any other modules.

The allowed parameter lists for commands are documented with module [<u>StdInterpreter</u>](../../Std/Docu/Interpreter.odc.md).

The BlackBox Component Framework tries to call the command *Config.Setup* when it is starting up. You can change *Config* in order to customize your configuration upon startup.

See also modules [<u>DevDebug</u>](Debug.odc.md) and [<u>Config</u>](../../System/Docu/Config.odc.md).

<a id="Debugging"></a>**5 Debugging**

When a run-time error occurs, e.g., a division by zero or some assertion trap, a *trap window* is opened. Such a window contains a text which shows the stack contents at the time when the trap occurred. An extract of such a trap text is shown below:



In the first line, the exception or trap number is given, e.g., "Index out of range" or "TRAP 0". Further below, a sequence of procedure activations is given, e.g., the last active procedure (where the trap occurred) was *ObxTrap.Do*, which had been called by *StdInterpreter.CallProc*, which had been called by *StdInterpreter.Command*, etc. Each procedure is marked with a small diamond mark to the left of its name. Clicking on this diamond mark produces a new window which shows the global variables of the module in which this procedure is defined, e.g.,



The diamond mark to the right of a procedure opens the source of the module in which this procedure is defined, selects the statement which has been interrupted, and scrolls to this selection. Of course, this is only possible for modules whose source code is available.

But now let us go back to the stack display. The lines below a procedure's name show the parameters and local variables of the procedure, sorted alphabetically. The following example

    str    Dialog.String    "String"

means that a local variable *str *of type *Dialog.String* had the value "String" when the trap occurred. If the variable name is displayed in italics, this means that it is a variable parameter (i.e., representing another variable).

After a pointer variable, a diamond mark allows to follow the pointer to the record to which it points, e.g. the pointer *v*

    v    Views.View    [85D29730H]

can be followed (by clicking on the diamond mark) to the record



Such a display is opened in another window. The fields of a record are indicated by the preceding ".". On the first line, the path you have followed is indicated. If you have followed more than one dereferencing step, a diamond mark at the right end of this line lets you trace back again step by step.

Array elements and record fields are wrapped into folds (-> *StdFolds*), which can be opened or closed by clicking on the fold views. Folding of structured data types makes it easier to get an overview in complex situations.

Note that the debugger only finds source text files if they are located at their correct places and under their correct names, according to their subsystems. For example, for module "FormViews", the debugger looks for file *Form/Mod/Views*. If the file is stored somewhere else, or under another name, the debugger will look for an open window with the appropriate module source in it. It this also fails, it opens a dialog box so that the user can show the debugger where to find the correct file.

When a trap occurs in a view implementation, e.g., in the view's *Restore* procedure, the error which caused it might later lead to another trap again, e.g., when the view becomes uncovered, its *Restore* procedure is called again. In the worst case, this may lead to a never-ending sequence of traps. BlackBox prevents this by partially disabling a trapped view. Such a view is overlaid with a light grey pattern; its contents won't be restored again. However, the view's remaining behavior is still intact, i.e., it may be saved to a file. If saving leads to a trap, the view is turned into an alien; this is indicated by a cross overlaid over the view. The rest of the container document could still be saved, however.

BlackBox distinguishes three categories of view traps:

ꀢ refresh: a trap in the view's *Restore* or *RestoreMarks* procedures

ꀢ save: a trap in the view's *Internalize* or *Externalize* procedures

ꀢ other: a trap in any other of the view procedures

If a trap in one of those categories occurs, this behavior will be disabled (i.e., the system won't call the procedures in this category anymore), behavior of the other categories remains intact. In the worst case, a view can lead to three traps, one for each category.

This means that erroneous views "degrade gracefully", by freezing only those behaviors that already have led to traps. If for some reason a view should be "unfrozen" again, the command *Dev*->*Revalidate View* can be used.

An endless loop can be interrupted by pressing *ctrl-break* (Windows) / *command-option-.* (Mac OS).

The following standard traps may occur at run-time (depending on the platform, some of these errors don't occur):

    invalid WITH

    invalid CASE

    function without return

    type guard

    value out of range

    index out of range

    string too long

    stack overflow

    integer overflow

    division by zero

    infinite real result

    real underflow

    real overflow

    undefined real result

    not a number

    keyboard interrupt

    NIL dereference

    illegal instruction

    NIL dereference (read)

    illegal memory read

    NIL dereference (write)

    illegal memory write

    NIL procedure call

    illegal execution

    out of memory

    bus error

    address error

    fpu error

    exception

Besides these standard traps, there are custom traps. Each custom trap has a trap number associated with it. The following conventions are used:

Free      0 ..  19

Preconditions     20 ..  59

Postconditions     60 ..  99

Invariants    100 .. 120

Reserved    121 .. 125

Not Yet Implemented             126

Reserved             127

You can use trap codes 0..19 freely in your programs, typically as temporary breakpoints during debugging. By convention, the other trap codes are generated by various assertions, i.e., statements which check a certain condition, and terminate the command if the condition is violated. Conditions which must be fulfilled upon entry of a procedure are called *preconditions*, conditions which must be fulfilled upon exit of a procedure are called *postconditions*, and conditions which must be fulfilled in between are called *invariants*. Most procedures in the BlackBox Component Framework check some preconditions. Typically, trap numbers are kept unique inside of a procedure. Thus, if a trap occurs, consult the documentation (or source, if available) of the trapped procedure. There you should get more information about the cause of the trap.

The developer may even provide a plain-text description of the trap's cause, by providing suitable resources in the subsystem's string resource file: module name without subsystem prefix, followed by ".", followed by the procedure name, followed by ".", followed by the trap number. The following are examples:

Math.Power.23    Pre: x # 1.0  OR  ABS(y) # INF

Views.View.CopyFrom.20    Views.CopyFrom and Views.CopyFromModelView must not both

    be overwritten

If such a resource exists, the corresponding text is shown in the trap display.

It should be noted that at no time during debugging the normal BlackBox Component Builder environment is left, there is no special "debugging mode" or "debugging environment"; everything is integrated instead!

Example: [<u>ObxTrap  docu</u>](../../Obx/Docu/Trap.odc.md)

The BlackBox debugger is a cross between a "post-mortem" debugger and a "run-time" debugger. It is invoked after a command has trapped (post-mortem), but it doesn't cause a termination of the BlackBox environment (run-time). Some features, such as the *Info->View State* command, which makes it possible to follow data structures starting from a selected view, are usually associated with run-time debuggers only.

It is typical for object-oriented programs that their control flows can become extremely convoluted and hard to follow. Thus following a program statement for statement (single step), by message sends, or by procedure calls in practice turns out to be unpractical for debugging large systems. Instead, BlackBox uses a more effective debugging strategy:

*Let errors become manifest as soon as possible.*

Instead of waiting for some error to occur, and then trying to find one's way backward to the cause of the error, it was attempted to flag errors as closely to their cause as possible. This is the only way to truly save debugging time. The language implementation follows the same strategy, by checking index overflows when accessing arrays, by checking *NIL* accesses when dereferencing a pointer, etc. In addition to these built-in checks, Component Pascal provides the standard procedure *ASSERT*, which allows to test for an arbitrary condition. If the condition is violated, a trap window is opened. Procedures of the BlackBox Component Framework consequently use assertions e.g. at the beginning of a procedure to check whether its input is valid. This prevents that a procedure with illegal input may perform any damage to the rest of the system.

This defensive programming strategy has proven itself again and again during the development of BlackBox, and is strongly recommended for serious development work.

See also module [<u>DevDebug</u>](Debug.odc.md).

<a id="Distribution"></a>**6 Deployment**

If you develop add-on components (i.e., new subsystems) for BlackBox, deployment simply means to distribute copies of your subsystem directory. You may not want to distribute the source texts, in order to protect your intellectual property rights. In this case you should not put them into the distribution copies. You may not want to make one or several of your module interfaces public, or none at all if your application is not meant to be extensible. In this case, you should not put the corresponding symbol files into the distribution copies. However, for each public module you should put a documentation file into the subsystem's *Docu* directory. For example, for module *FormCmds* there is a docu file *Form/Docu/Cmds*.

It is customary to put a *Sys-Map* file in the *Docu* directory. This map text contains a list of all public modules; the elements of the list are hyperlinks.

If the subsystem implements user interface elements such as views or commands, a user manual should be made available. This document is called *User-Man* and also resides in the *Docu* directory.

If the subsystem is meant for use by programmers, and if the individual modules' documentation files are not sufficient, the *Docu* directory should contain an overview text called *Dev-Man*.

The previous paragraph has shown everything you need to do to distribute an add-on component, i.e., an extension in the form of a subystem. On the other hand, if you want to distribute a stand-alone application for some reason, you have three possibilities:

1) This approach is suited for applications where updates and extensions are frequent: you provide the unlinked code files of your application and of BlackBox, together with a minimal linked application to boot the whole software system.

To make an unpacked and unlinked stand-alone version you need to copy some files and directories from the *BlackBox* directory to a new directory. *BlackBox.exe* may be renamed to the name of your new application.

2) This approach is suitable for applications that are closed or where incremental updates and extensions are relatively rare: you pack the BlackBox directory's contents into one large executable file. In contrast to the linker (see below), the packer is not limited to code files; it can also pack resources and documentation and any other kind of files into the application. In contrast to approach 1), the end user cannot easily destroy the applications just by deleting some code or resource file. See the [<u>packer documentation</u>](Packer.odc.md) for details.

3) This approach is most relevant for native applications (i.e., Component Pascal applications which don't use the BlackBox framework) and for linked libraries (DLLs under Windows): you use the linker tool to combine several code files into one application file.

For a linked version you need to use the linker to pack your code files together with the BlackBox code files into a single application. See the [<u>linker documentation</u>](P-S-I.odc.md) for details. After linking, you need to copy the new application to a new directory. You also need to copy the *Rsrc* subdirectories of the *Form*, *Host*, *Std*, *System*, *Text*, and of your own subsystems. If the resource files are missing, the application would still operate (kind of), but lose all string mappings, all dialog box layouts, and all custom menus.

All approaches:

In addition, you need to create your own versions of the menu definition file and of the *About...* dialog box (*System/Rsrc/Menus* and *System/Rsrc/About*). If your application is meant to be extensible, you also need to copy the symbol files (*Sym/Xyz*) of all modules whose interfaces should be public. This may or may not encompass the standard BlackBox modules. Make sure the structure of your new directories matches the subdirectory structure in the original *BlackBox* directory!

Windows:

Adapt the *Help* menu command, and the corresponding help text in an appropriate way.

Mac OS:

Edit the *Help* text in the *System/Rsrc* directory in an appropriate way.

To distribute software electronically over a medium which doesn't support binary distribution, e.g., over e-mail, a [<u>standard ASCII encoder</u>](../../Std/Docu/Coder.odc.md) is available.

<a id="Cross-Platform"></a>**7 Cross-Platform issues**

If you use BlackBox both on Mac OS and on Windows, you'll regularly transfer documents between the two platforms. Code files cannot be transferred, since their formats are non-portable. To simplify the transfer of documents, you should configure the available tools (e.g., Apple's *PC Exchange* software) such that they map the Mac OS file type ("oODC") and file creator ("obnF") to the Windows file name suffix ".odc" and vice versa.

Furthermore, you may want to use fonts which are available on both platforms, either TrueType fonts or PostScript fonts if you have Adobe's Type Manager available. If you don't have equivalent fonts on both platforms, you can still read the text, with another font substituted for the correct one. This will likely result in visually unsatisfactory results.

Use the default font for cross-platform documents, if you have no preference for a specific font. Typically, the default font is used for on-line documentation. The default font automatically adapts to the reader's system configuration, so that he or she can select their preferred font.

Like any other document, a dialog box can be transferred between the two platforms. However, for best results you may want to fine-tune a dialog box layout for the platform's controls before distributing it.

In order to minimize installation problems, it is recommended to limit module names to at most eight characters (not counting the subsystem prefix, which itself may be up to eight characters), and not to use module names which only differ in their use of small or capital letters.

