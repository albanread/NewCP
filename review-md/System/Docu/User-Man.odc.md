**BlackBox Component Builder**

**User Manual**

**Contents**

[<u>1 Overview</u>](#Overview)

[<u>2 Conventions</u>](#Conventions)

[<u>3 Installation (Windows only)</u>](#Installation Windows)

[<u>4 Server installation (Windows only)</u>](#Server version)

[<u>5 Deinstallation</u>](#Deinstallation)

[<u>6 Document Windows, Tool Windows, Auxiliary Windows</u>](#Windows)

[<u>7 Document Size</u>](#Document Size)

[<u>8 Menu Configuration</u>](#Menu Configuration)

[<u>9 String Resources</u>](#String Resources)

[<u>10 Standard Commands</u>](#Standard Commands)

    [<u>10.1 File Menu</u>](#File Menu)

    [<u>10.2 Edit Menu</u>](#Edit Menu)

    [<u>10.3 Font Menu (Mac OS)</u>](#Font Menu)

    [<u>10.4 Attributes Menu</u>](#Attributes Menu)

    [<u>10.5 Window Menu (Windows) / Windows Menu (Mac OS)</u>](#Window Menu)

[<u>11 Custom Commands</u>](#CustomCommands)

**Further user manuals**

[<u>Text Subsystem</u>](../../Text/Docu/User-Man.odc.md)

[<u>Form Subsystem</u>](../../Form/Docu/User-Man.odc.md)

[<u>Dev Subsystem</u>](../../Dev/Docu/User-Man.odc.md)

<a id="Overview"></a>**1 Overview**

This document serves as a user's guide for BlackBox programmers. It is not intended as a manual for the end user of software written with the BlackBox Component Builder. Knowledge of the underlying platform's user interface guidelines is assumed.

With the BlackBox Component Builder, there are no separate environments for developing programs, for testing and debugging programs, or for the distribution of programs. Instead there is only one, truly integrated, environment for all these purposes. A distribution version of a BlackBox application can be created simply by stripping away all tools which are specific to the development process.

*Neither development tools (the Dev-subsystem) nor documentation may be distributed. However, all other parts of BlackBox may be freely distributed along with applications; there are no royalties or other fees.*

Such a customized BlackBox environment always has the basic capabilities of the BlackBox Component Framework's compound document architecture, and of the standard text and form subsystems. Furthermore, it constitutes a standard Windows or Mac OS application, where the (boot) application can be double-clicked, where documents can be dropped onto the application icon to open them, etc.

"Native" applications which are only developed under, but not based on, the BlackBox Component Builder can be created as well. For this purpose, a linker tool is provided. Normally, Component Pascal modules are linked and loaded dynamically, such that a separate linker is not strictly necessary. However, it is possible even for pure BlackBox applications to link or pack all or some of their modules together, in order to reduce the number of files to distribute. For further information about this topic, refer to the document [<u>Platform-Specific Issues</u>](../../Dev/Docu/P-S-I.odc.md).

On-line documentation:

Text stretches which are blue and underlined are hyperlinks, and can be followed by clicking on them.

When working in the BlackBox Component Builder environment for the first time, the following may be helpful to remember: almost all modifications to BlackBox documents are undoable, making it quite safe to try out a feature. In general, *multi-level undo/redo* is available, i.e., not only one, but several commands can be undone; as many as memory permits.

Mac OS:

Whenever the system is working on the completion of some command for more than a fraction of a second, BlackBox changes the cursor to a special *busy cursor* sequence, i.e., a cursor which changes its shape as long as the command is running. However, this does not guarantee that the executing program does make any progress. For example, the busy cursor will keep advancing, even if the program entered an endless loop. Use *command-option-.* to terminate such a program. Note that it may take a few seconds before you see a reaction.

Windows:

An endless loop can be terminated with *ctrl-break*.

<a id="Conventions"></a>**2 Conventions**

The contents of this documentation mostly applies to both the Windows and Mac OS versions of the BlackBox Component Builder. Where there are differences, e.g., a feature only available on one platform, this is clearly indicated, e.g., as in the previous paragraph. In order to reduce the number of such platform-specific remarks, a few notational conventions are followed:

ꀢ Mac OS folders are called (sub)directories.

ꀢ Path names contain "/" as directory separators, as in Unix and in the World-Wide Web.

ꀢ File and directory names contain both capital and small letters.

ꀢ In chapters which are not Windows-specific, document file names are given without the ".odc" suffix used in Windows. Thus the file name *Text/Rsrc/Find* under Windows corresponds to *Text\Rsrc\Find.odc*, and to *Text:Rsrc:Find* under Mac OS.

ꀢ *modifier* key: on Windows this is the *ctrl* key, on Mac OS this is the *option* key.

ꀢ menu commands: *M->I* is a shorthand notation for menu item *I* in menu *M*, e.g. *File->New*.

ꀢ Differences between Windows and Mac OS may be denoted in three different ways: large differences are handled by giving two different sections, one for each platform. In this case, the section title ends in "(Windows only)" or "(Mac OS only)". If not indicated otherwise, the next section will apply to both platforms again. Smaller differences are described as short notes, typically there is a note on Windows-specific behavior, followed by a note on Mac OS-specific behavior. There is an example at the end of the previous section. For small differences, both versions are given in the same sentence, e.g. "To drag & pick, hold down the *alt* key (Windows) / *command* key (Mac OS) while dragging."

<a id="Installation Windows"></a>**3 Installation (Windows only)**

The BlackBox Component Builder requires a PC with an i386 or better, with Windows NT, Windows 2000, Windows XP, or Windows Server 2003.

A two-button mouse is sufficient, but a three-button mouse is recommended.

The resulting *BlackBox* directory contains several files and directories:

Files:

BlackBox.exe    The BlackBox Component Framework boot application.

BlackBox.exe.manifest    Manifest file for BlackBox.exe.

Empty.odc    Empty BlackBox Component Builder document.

Tour.odc    A quick tour through the BlackBox Component Builder.

unins000.dat    Data file for BlackBox Component Builder Uninstall.

unins000.exe    BlackBox Component Builder Uninstall.

Directories:

Com    Direct-To-COM Compiler.

Comm    Communications subsystem.

Ctl    OLE Automation support.

Dev    Development subsystem.

Docu    On-line documentation not specific to a particular module or subsystem.

Form    Form subsystem with the visual designer.

Host    Private code of BlackBox.

Obx    Obx subsystem, a collection of examples.

Ole    OLE compound document support.

Sql    Sql subsystem for accessing relational databases.

Std    A number of command packages available to the user.

System    Core of the BlackBox Component Framework.

Text    Text subsystem, with the standard document/program editor.

Win    Interface modules for direct Windows API access.

Xhtml    Exporter for text to HTML conversion.

<a id="Server version"></a>**4 Server installation (Windows only)**

If several developers use BlackBox on the same machine(s), it becomes cumbersome to save one's work and clean up after the end of each session, so that the next developer gets a "clean" system again. It is even more cumbersome to start working with a copy that hasn't been cleaned up correctly by the previous user. To solve this problem, server support is available for BlackBox.

BlackBox can be installed and maintained on one central server, while developers use an arbitrary number of client workstations on a local-area network. Preferably, each developer has his or her own working directory on the server (account).

Note that with the Classic Edition of BlackBox you need seperate licenses for each developer seat using the server installation.

For the installation, follow these steps:

1) Install BlackBox in a directory on the server machine, using *Setup.exe*. The directory must be *shared* on the network but the access may be restricted to *read only*.

2) For each workstation / user, create a working directory (either on the server or on the client). The user should have *read/write* access to this directory. The directory may be empty.

3) For each workstation, create a *shortcut* with the following contents:

  Command Line (Target): *<BlackBoxDir>*\BlackBox.exe /Use *<WorkDir>*

  Working Directory (Start in): *<WorkDir>*

where <BlackBoxDir> stands for the full path name of the directory where BlackBox is installed (on the server), and <WorkDir> is the path of the working directory of the actual machine (on the client or on the server). The latter must not end with a backslash ("\"). Use double quotation marks to delimit path names that contain spaces.

Example:

  Command Line (Target): "C:\Program Files\BlackBox\BlackBox.exe" /Use C:\BlackBox

  Working Directory (Start in): C:\BlackBox

These features, originally introduced for situations where multiple users want to develop with BlackBox, can also be useful in a single-user environment. The idea is to have one directory with the original installation of BlackBox, and a separate directory for all the developer-specific files. If the user opens a file, it is first searched in his or her working directory. If it isn't found there, the corresponding file in the "server directory" is opened. When a file is saved, it is always saved in the working directory. This makes it easy to set up entirely separate projects simply by creating separate working directories for them. When an upgrade of BlackBox comes out, only the central "server directory" needs to be upgraded. If you use the server features in this way, without having multiple developers using the same installation simultaneously, then *no* special license is required.

<a id="Deinstallation"></a>**5 Deinstallation**

In the Windows Control Panel, choose *Add/Remove Programs*. In the displayed list, locate the entry called *BlackBox* and select the *Change/Remove* button. Follow the instructions on the screen.

<a id="Windows"></a>**6 Document Windows, Tool Windows, Auxiliary Windows**

There are three kinds of windows in BlackBox: document windows, tool windows, and auxiliary windows.

A document window may contain e.g. a text or a form layout, or any other kind of visual object ("view"). When the contents of a document window have been modified (made "dirty") and the user tries to close the window (or quit the application), the system asks whether it should save the document.

A tool window allows to invoke actions on some document window *underneath* it. Typically, tool windows are used for modeless dialog boxes.

Windows:

Tool windows look the same way as dialog boxes.

Mac OS:

Tool windows look the same way as modeless dialog boxes, i.e. the same way as document windows. In order to make it possible for the user to distinguish tool and document windows, titles of tool windows are put between "<<" and ">>" brackets.

Auxiliary windows are used mainly to hold temporary data for information purposes, e.g., the output of a browser. The contents of an auxiliary window may be editable, but the system does not ask whether a modified auxiliary window should be stored, i.e., it is temporary in nature. The *Log* window is an example of an auxiliary window.

Windows:

A document window is decorated with the BlackBox document icon, while an auxiliary window is decorated with the BlackBox application icon. A tool window is not decorated with an icon.

Mac OS:

Auxiliary windows look the same way as document windows. In order to make it possible for the user to distinguish auxiliary and document windows, the titles of auxiliary windows are put between "[" and "]" brackets.

<a id="Document Size"></a>**7 Document Size**

The size of a document, or more exactly of its outermost ("root") view, can be updated in several ways. Its width, or independently its height, can be bound either to a fixed size, to the paper as defined in the *Page Setup* dialog box, or to the window's current size.

For example, text views by default have a width bound to the paper page size, and a height bound to the window size. Documentation texts are often bound to the current window size in both dimensions, so that they automatically resize with the window. Such bindings can be changed with the *Tools->Document Size...* dialog box.

<a id="Menu Configuration"></a>**8 Menu Configuration**

The configurable menus can be inspected using *Info->Menus*. The displayed text can be edited, and the current menu configuration updated accordingly (*Info->Update Menus*). To make changes to the menus permanent, the menu text must be saved to disk. The file *System/Rsrc/Menus* contains the startup menu configuration, i.e., the text which is opened when *Info->Menus* is executed.

The menu text consists of a sequence of menu definitions, which themselves consist of sequences of menu items. An example is the following extract of a possible *Dev* menu definition:

 **MENU **"Dev"

    "Compile"    "K"    "DevCompiler.Compile"    "TextCmds.FocusGuard"

    "Compile Selection"    ""    "DevCompiler.CompileSelection"    "TextCmds.SelectionGuard"

    **SEPARATOR**

    "Unmark Errors"    ""    "DevMarkers.UnmarkErrors"    "TextCmds.FocusGuard"

    "Next Error"    "E"    "DevMarkers.NextError"    "TextCmds.FocusGuard"

    "Toggle Error Mark"    ""    "DevMarkers.ToggleCurrent"    "TextCmds.FocusGuard"

    **SEPARATOR**

    "Insert Commander"    ""    "DevCommanders.Deposit; StdCmds.PasteView"    "TextCmds.PasteViewGuard"

    "Execute"    ""    "DevDebug.Execute"    "TextCmds.SelectionGuard"

    "Unload"    ""    "DevDebug.Unload"    "TextCmds.SelectionGuard"

**END**

Every menu has a name, in this case it is *Dev*. Optionally, the menu name can be followed by a menu type, e.g.,

**MENU **"Text" ("TextViews.View")

A typed menu is only installed in the menu bar as long as the current focus has a matching type, i.e., it is context-sensitive. The other menus are always available.

A menu's type is usually simply the name of a view type. This is only a convention, however. It guarantees that menu types are globally unique, so that no clashes occur.

Mac OS:

Standard menus, i.e., *File*, *Edit*, *Font*, *Attributes*, and *Windows* are predefined and not part of a menu configuration text.

There are two kinds of menu items: normal items and separators. A separator optically organizes a menu into different groups of items. Normal menu items consist of four strings: a label, a keyboard shortcut, an action command, and a guard command. The label is the string presented to the user in the menu. A "&" character indicates which character of the label should be underlined (this is a Windows feature, and not available on Mac OS). If you want a "&" to appear, you should write a "&&" (this also holds for the Mac OS, i.e., the syntax is the same for both platforms).

The keyboard shortcut, which may be empty, allows to associate a keyboard key to the menu item. The action string contains the command sequence which is activated when the menu item is executed. The guard string, which may be empty, contains a command which is called to determine whether the item is currently enabled or disabled, checked or unchecked, or to set up a current item name which overrides the normal name (e.g., to toggle between *Show XYZ* and *Hide XYZ*).

Note: the menu guard is executed for example when the user clicks in the menu bar. This causes the guard's module to become loaded, even if the user never invokes the corresponding command.

Note: if the guard's module cannot be loaded, the menu item remains disabled and the guard is not executed again (for performance reasons). If the module's code becomes available later, e.g., because its module was later compiled, the menu item will remain disabled. To force a re-evaluation of the guard, use the *Dev->Flush Resources* command.

Note: the standard menu configuration uses all letters of the alphabet and the digit "0" as keyboard shortcuts. Digits "1" to "9" are not used. The assignment of keyboard shortcuts, like the whole menu configuration, can easily be adapted to specific needs by appropriately changing the menu text.

Windows:

The following keyboard shortcuts can be specified in the keyboard shortcut string (they are ignored under Mac OS):

    "A".."Z", "0".."9"    modifier + key

    "*A".."*Z", "*0".."*9"    shift + modifier + key

    "F1".."F12"    function key

    "^F1".."^F12"    modifier + function key

    "*F1".."*F12"    shift + function key

    "*^F1".."*^F12"    shift + modifier + function key

Windows:

Context menus (pop-up menus activated by the right mouse button) are specified by giving them the name "*" instead of a true name. At most one context menu may be untyped, all other context menus must be typed.

For example, just add the line

        "Open &Module"  ""      "DevCmds.OpenModuleList"        "TextCmds.SelectionGuard"

to the menu  MENU "*" ("TextViews.View")  in the file *Text/Rsrc/Menus*. This adds a (text-)context menu item for opening the sources of the module(s) whose name(s) is (are) selected.

It is possible to put all menu specifications in the menu text *System/Rsrc/Menus*. However, it is a better idea to keep the menu specifications that refer to a subsystem's commands in this subsystem's resource directory. For example, the *Text* menu could be specified in *Text/Rsrc/Menus*. In this case, *System/Rsrc/Menus* needs a so-called include statement that tells the menu configuration mechanism where to look for further menus:

    **INCLUDE** "Text"

The explicit include statements allow to define the exact order in which menus appear in a menu bar. The command

    **INCLUDE** "*"

includes all menus that have not been mentioned explicitly before (directly or via an include statement). It is a "catch all" for menus, and it is recommended to put it at the end of the *System/Rsrc/Menus* text. For example, a typical *System/Rsrc/Menus* text may look as follows:

    **<u>INCLUDE</u>** "Dev"

    **<u>INCLUDE</u>** "Form"

    **<u>INCLUDE</u>** "Sql"

    **<u>INCLUDE</u>** "Obx"

    **<u>INCLUDE</u>** "Text"

    **INCLUDE** "*"

Note: when you have edited a menu text different from *System/Rsrc/Menus*, then don't execute the command *Info->Update Menus*, because this will cause the installation of the edited menu text as root menu text, meaning that you lose all other subsystems' menus. In particular, it is inconvenient if you lose the *Info* menu in this way, because you then have to leave and restart the application before you can edit and install menus again...

So when you have edited some such other menu text, then save it and execute *Info->Update Menus*. This command will re-install all menus, starting with the *System/Rsrc/Menus* like it does when starting up the application.

See also modules [<u>StdMenuTool</u>](../../Std/Docu/MenuTool.odc.md) and [<u>StdCmds</u>](../../Std/Docu/Cmds.odc.md). The command package modules of all subsystems export commands that may be used in a menu. Consult the various modules' on-line documentation, e.g., *Text/Docu/Cmds* for module *TextCmds*.

<a id="String Resources"></a>**9 String Resources**

String resources are files which define a mapping between strings, e.g., the string "untitled" may be mapped to "sans titre". This is useful to prevent hard-wiring textual messages in the program code, in order to make later editing of these messages possible without requiring a recompilation. From a programmer's point of view, string translation is done in several procedures of module *Dialog*, e.g., *Dialog.MapString*. String resource files can be normal BlackBox text documents, which simply consist of the keyword STRINGS followed by a sequence of lines; each line contains a string (the key), a TAB, another string (to which the key is mapped), and a carriage return, e.g.,

STRINGS

untitled    sans titre

open    ouvre

close    ferme

There can be one string resource file per subsystem (->7.2 Modules and Subsystems). For example, a call of

Dialog.MapString("#Form:CntrlInstallFailed", resultString)

in a program maps the string "CntrlInstallFailed" according to the table in the *Form/Rsrc/Strings* file. In the English version, the mapping is "form controller installation failed". In a German version, "CntrlInstallFailed" might be mapped to "Der Form Controller konnte nicht installiert werden".

<a id="Standard Commands"></a>**10 Standard Commands**

In this section, the menu items of the standard menus *File*, *Edit*, *Font* (Mac OS), *Attributes*, and *Window(s)* are described. For a menu item which is not permanently enabled, the condition for enabling it is specified. Often, such a *guard* command is one of the commands exported by module *StdCmds*.

Windows:

Under Windows, the standard menu items can be configured in the same way as all other menu items, by editing the *System/Rsrc/Menus* document. Most standard menu items are calls to a command exported by module *StdCmds*.

<a id="File Menu"></a>**10.1 File Menu**

**New**

Command: StdCmds.New

Guard:

Opens a new document window containing an empty text view.

**Open...**

Command: HostCmds.Open

Guard:

Opens the standard file *Open* dialog box.

Mac OS:

Only BlackBox documents (i.e., Mac OS files with file type = "oODC"), directories, and volumes are shown. By clicking the *More files* check box, all other files for which there are converters are shown as well. In the *Format* pop-up menu, a converter can be chosen in case there are several possible importers.

**Open Stationery...**

Command: StdCmds.OpenStationery

Guard:

Opens the standard file *Open* dialog box, through which a stationery (i.e., template) file can be opened.

Mac OS:

Only BlackBox documents (i.e., Mac OS files with file type = "oODC"), directories, and volumes are shown. By clicking the *More files* check box, all other files for which there are converters are shown as well. In the *Format* pop-up menu, a converter can be chosen in case there are several possible importers.

**Close**

Command: HostCmds.Close

Guard: StdCmds.WindowGuard

Closes the front window. If the window is a primary document window and its contents has been modified ("dirty"), the user is asked whether to save the window's contents in a file.

**Save**

Command: HostCmds.Save

Guard: HostCmds.SaveGuard

Saves the front window's contents to a file. If the window's contents has not yet been saved to a file, the user is asked for a file name.

**Save As...**

Command: HostCmds.SaveAs

Guard: StdCmds.WindowGuard

Saves the front window's contents to a file. The user is always asked for a file name. After the command, you continue working with the new file.

**Save Copy As...**

Command: HostCmds.SaveCopyAs

Guard: StdCmds.WindowGuard

Saves the front window's contents to a file. The user is always asked for a file name. After the command, you continue working with the old file.

**Page Setup...**

Command: HostDialog.InitPageSetup; StdCmds.OpenToolDialog('Host/Rsrc/Cmds', 'Page Setup')

Guard: StdCmds.WindowGuard

Asks the user for the page information of the front window's document, for later printing.

In addition to the data which is specific to the current printer driver, the margins can be set (the distances between the paper's edges and the printed area), and a standard header can be switched on or off. The standard header consists of a page number and a date.

Mac OS:

For some printer drivers, you should switch on greyscale printing (not black/white), otherwise printing may produce entirely black pages.

**Print...**

Command: HostCmds.Print

Guard: HostCmds.PrintGuard

Asks the user for printing information, and then creates a print-out accordingly.

**Send Document...** (Windows)

Command: HostMail.SendDocument

Guard: HostMail.SendDocumentGuard

Sends the front window's document as an electronic mail.

**Send Note...** (Windows)

Command: HostMail.SendNote

Guard: HostMail.SendNoteGuard

Sends a note as an electronic mail. The not is initialized with the text selection, if there is one.

**Exit** (Windows)

**Quit** (Mac OS)

Command: HostCmds.Exit

Guard:

Terminates the application. If windows with modified contents are open, the user is asked whether to save them in files.

<a id="Edit Menu"></a>**10.2 Edit Menu**

**Undo [...]**

Command: StdCmds.Undo

Guard: StdCmds.UndoGuard

Reverses the effect of the most recent modifying operation. Usually, the kind of operation is given behind the word "Undo", e.g., *Undo Paste*. Undo can be activated several times, until the opening, creation, or most recent saving of the document. Under low-memory conditions, the number of undoable operations may become reduced.

**Redo [...]**

Command: StdCmds.Redo

Guard: StdCmds.RedoGuard

Restores the effect of the most recently undone operation. Usually, the kind of operation is given behind the word "Redo", e.g., *Redo Paste*.

**Cut**

Command: HostCmds.CutGuard

Guard: HostCmds.Cut

Deletes the selection and puts a copy into the clipboard.

**Copy**

Command: HostCmds.CopyGuard

Guard: HostCmds.Copy

Puts a copy of the selection into the clipboard.

**Paste**

Command: HostCmds.PasteGuard

Guard: HostCmds.Paste

Pastes a copy of the clipboard's contents at the caret position. If the focus view contains the same kind of data as the clipboard, the data is inserted directly into the focus view's data. Otherwise, and if the focus view is a container, a copy of the whole view containing the clipboard data is inserted into the focus view's data.

**Delete** (Windows)

Command: StdCmds.Clear

Guard: StdCmds.CutGuard

Deletes the selection, without putting it into the clipboard.

**Copy Properties**

Command: StdCommands.CopyProp

Guard: StdCmds.SelectionGuard

Copies the properties of the current selection. This command has no effect on the clipboard contents.

**Paste Properties**

Command: StdCommands.PasteProp

Guard: StdCmds.SelectionGuard

Pastes the properties that were copied most recently (see *CopyProperties*).

**Paste Object** (Windows)

**Paste as Part** (Mac OS)

Command: HostCmds.PasteObject

Guard: HostCmds.PasteObjectGuard

Pastes a copy of the clipboard's contents at the caret position. If the focus view is a container, a copy of the whole view containing the clipboard data is inserted into the focus view's data.

**Paste Special...** (Windows)

Command: HostCmds.PasteSpecial

Guard: HostCmds.PasteObjectGuard

Opens a dialog box, which allows to choose the data type of the view in the clipboard, if the view supports several possible types.

**Paste to Window** (Windows)

Command: HostCmds.PasteToWindowGuard

Guard: HostCmds.PasteToWindow

Opens a copy of the clipboard's contents into a new document window.

**Insert Object...** (Windows)

Command: OleClient.PasteSpecial

Guard: StdCmds.PasteViewGuard

Opens a dialog box which shows all installed OLE servers. When one of them is chosen, an object of this type is allocated and inserted into the front window's contents.

**Object Properties...** (Windows)

Command: HostMenus.ObjProperties

Guard: HostMenus.PropertiesGuard

Opens an appropriate property sheet for the selected view.

**Object** (Windows)

Command: HostMenus.ObjectMenu

Guard: HostMenus.ObjectMenuGuard

Shows a submenu with commands for the selected view. These commands, which are determined by the selected view itself, are called "verbs". Usually, the first two verbs are *Edit* and *Open*:

    **Edit**

    Makes the selected view the current focus view.

    **Open**

    Opens a new window showing a second view to the selected view.

    + other verbs defined by the selected view.

**Clear** (Mac OS)

Command: StdCmds.Clear

Guard: StdCmds.CutGuard

Deletes the selection, without putting it into the clipboard.

**Select Document**

Command: StdCmds.SelectDocument

Guard: StdCmds.WindowGuard

Selects the root view of the front window's document as a singleton. Note the difference to *Select All*, which selects the latter's contents instead (or rather the contents of whatever view is currently the focus).

**Select All**

Command: StdCmds.SelectAll

Guard: StdCmds.SelectAllGuard

Selects the whole focus view's contents.

**Select Next Object** (Windows)

Command: StdCmds.SelectNextView

Guard: StdCmds.ContainerGuard

If a view in the container is selected: select the next view.

If the last view is selected or there is no singleton selection: select the first view.

**Part Info** (Mac OS)

Command: HostCmds.PartInfo

Guard: StdCmds.SingletonGuard

Opens a modeless dialog box with some information about the selected view.

**View In Window** (Mac OS)

Command: HostCmds.ViewInWindow

Guard: StdCmds.WindowGuard

Opens a new window on the front window's focus view. The subwindow's title is put between "(" and ")" parentheses. This command is similar to "New Window" (see below), except that it opens a new window on the currently focused view, which may or may not be the root view (as in "New Window").

**Preferences...** (Windows)

Command: HostDialog.InitPrefDialog; StdCmds.OpenToolDialog('HostDialog.prefs', 'Preferences')

Guard:

Allows to define several parameters: whether TrueType metrics are used (for best printing results), whether screen updates are performed during scrolling, the font used as default for texts, the font used as default for controls, and whether the status bar is visible or not.

<a id="Font Menu"></a>**10.3 Font Menu** (Mac OS)

Available: font-carrying selection or caret in focus view in front window

**Default Font**

Sets the selection or the caret to the default font. Sets the selection or caret to the chosen font (more exactly: its typeface). The default font corresponds to one of the concrete fonts shown below, but this correspondence may be changed dynamically.

{typeface name}

Then the list of all the fonts which are currently available is given. Sets the selection or caret to the chosen font (more exactly: its typeface).

The menu item for the caret's font, or the font of the selection if it is homogeneous, is checked.

**Set Default Font**

Sets the selection or caret's font as the new default font. All visible text stretches which have the default font as their font are redrawn in the new default font.

<a id="Attributes Menu"></a>**10.4 Attributes Menu**

Available: style/size/color-carrying selection or caret in focus view in front window

The following commands work on the selection; if there is no selection, the caret's current attributes are affected instead. These attributes are used as defaults when typing in new text.

For colors, there is a system-wide color (default color) which can be modified by the user. Under Windows, the default color can be changes using an operating-system utility. Everything drawn in the default color will be updated accordingly.

**Regular** (Windows)

**Plain** (Mac OS)

Command: StdCmds.Plain

Guard: StdCmds.PlainGuard

Checked: if text to the left and to the right of the caret is plain, or if selection is homogeneously plain (i.e., non-bold, non-italicized, non-underlined, and non-striked-out).

Removes all style attributes (bold, italic, underline, strikeout) from the selection.

**Bold**

Command: StdCmds.Bold

Guard: StdCmds.BoldGuard

Checked: if text to the left and to the right of the caret is bold, or if selection is homogeneously bold.

If the selection is homogeneously bold, it is made non-bold, otherwise it is made bold.

**Italic**

Command: StdCmds.Italic

Guard: StdCmds.ItalicGuard

Checked: if text to the left and to the right of the caret is italic, or if selection is homogeneously italic.

If the selection is homogeneously italic, it is made non-italic, otherwise it is made italic.

**Underline**

Command: StdCmds.Underline

Guard: StdCmds.UnderlineGuard

Checked: if text to the left and to the right of the caret is underlined, or if selection is homogeneously underlined.

If the selection is homogeneously underlined, it is made non-underlined, otherwise it is made underlined.

**Strikeout** (Mac OS)

Command: StdCmds.Strikeout

Guard: StdCmds.StrikeoutGuard

Checked: if text to the left and to the right of the caret is striked out, or if selection is homogeneously striked out.

If the selection is homogeneously striked out, it is made non-striked-out, otherwise it is made striked out.

**(8 point (Windows),) 9 point, 10, 12, 16, 20, 24**

Command: StdCmds.Size(size)

Guard: StdCmds.SizeGuard(size)

Checked: if text to the left and to the right of the caret has the given size, or if the selection is homogeneously of the given size.

The selection is set to the given point size.

**Size...** (Windows)

**Other Size...** (Mac OS)

Command: StdCmds.InitSizeDialog; StdCmds.OpenToolDialog('Std/Rsrc/Cmds', 'Size')

Guard: StdCmds.SizeGuard(-1)

Checked: if none of the other sizes apply

A tool dialog box is opened, which allows to enter a particular font size in points, and then to set the selection to this size.

**Default Color**

Command: StdCmds.Color(1000000H)

Guard: StdCmds.ColorGuard(1000000H)

Checked: if text to the left and to the right of the caret has the default color, or if the selection is homogeneously of the default color

Sets the selection's color to the default color.

**Black**

Command: StdCmds.Color(0000000H)

Guard: StdCmds.ColorGuard(0000000H)

Checked: if text to the left and to the right of the caret is black, or if the selection is homogeneously black

Sets the selection's color to black.

**Red**

Command: StdCmds.Color(00000FFH)

Guard: StdCmds.ColorGuard(00000FFH)

Checked: if text to the left and to the right of the caret is red, or if the selection is homogeneously red

Sets the selection's color to red.

**Green**

Command: StdCmds.Color(000FF00H)

Guard: StdCmds.ColorGuard(000FF00H)

Checked: if text to the left and to the right of the caret is green, or if the selection is homogeneously green

Sets the selection's color to green.

**Blue**

Command: StdCmds.Color(0FF0000H)

Guard: StdCmds.ColorGuard(0FF0000H)

Checked: if text to the left and to the right of the caret is blue, or if the selection is homogeneously blue

Sets the selection's color to blue.

**Color...** (Windows)

**Other Color...** (Mac OS)

Command: HostDialog.ColorDialog

Guard: StdCmds.ColorGuard(-1)

Checked: if none of the other colors apply

Asks the user for a color, to which it then sets the selection.

**Set Default Color...** (Mac OS)

Guard: color-carrying selection or caret in focus view in front window

Asks the user for a color, to which it then sets the default color.

**Default Font** (Windows)

Command: StdCmds.DefaultFont

Guard: StdCmds.DefaultFontGuard

Sets the selection to the default font.

**Font...** (Windows)

Command: HostDialog.FontDialog

Guard: StdCmds.TypefaceGuard

Opens the standard font dialog box and applies the chosen font attributes to the selection.

**Typeface...** (Windows)

Command: HostDialog.TypefaceDialog

Guard: StdCmds.TypefaceGuard

Opens the standard font dialog box and applies the chosen font attributes to the selection. In contrast to the *Font...* command, only the typeface (the name of the font) is changed, but not the other attributes like size or weight (bold/normal).

<a id="Window Menu"></a>**10.5 Window Menu (Windows) / Windows Menu (Mac OS)**

**New Window**

Command: StdCmds.NewWindow

Guard: StdCmds.WindowGuard

Opens a new window on the same document as the front window. The window is of the same kind as the front window. The window's title is put between "(" and ")" parentheses.

**Cascade** (Windows)

**Stack** (Mac OS)

Command: HostMenus.Cascade

Guard: StdCmds.WindowGuard

Arrange document windows in an overlapping fashion.

**Tile Horizontal**

Command: HostMenus.TileHorizontal

Guard: StdCmds.WindowGuard

Arrange windows from left to right in a non-overlapping fashion. This command does not affect non-resizable windows, and it ignores some windows when there are too many open windows for a reasonable tiling. The front window becomes the left-most window.

**Tile Vertical**

Command: HostMenus.TileVertical

Guard: StdCmds.WindowGuard

Arrange windows from top to bottom in a non-overlapping fashion. This command does not affect non-resizable windows, and it ignores some windows when there are too many open windows for a reasonable tiling. The front window becomes the top-most window.

**Arrange Icons** (Windows)

Command: HostMenus.ArrangeIcons

Guard: StdCmds.WindowGuard

Arrange icons (minimized windows) at the bottom of the application window.

**Show Clipboard / Hide Clipboard** (Mac OS)

If the clipboard is open, it closes it and all its secondary windows. Otherwise it opens a clipboard window.

**{window}**

Command: HostMenus.WindowList

Guard:

Here the list of open windows is appended, the front window at the top (checked). Document window titles are in boldface if their contents has been modified.

<a id="CustomCommands"></a>**11 Custom Commands**

Not all commands are visible in the default configuration of the menus. The following is a list of modules from the *Std* subsystem. These modules contain many useful commands but not all of them appear in the menus. For more information about the commands in the modules, please consult the corresponding module documentation.

[<u>StdClocks</u>](../../Std/Docu/Clocks.odc.md)    analog clock views

[<u>StdCmds</u>](../../Std/Docu/Cmds.odc.md)    cmds of std menus

[<u>StdCoder</u>](../../Std/Docu/Coder.odc.md)    ASCII coder

[<u>StdDebug</u>](../../Std/Docu/Debug.odc.md)    minimal debugger

[<u>StdFolds</u>](../../Std/Docu/Folds.odc.md)    fold views

[<u>StdHeaders</u>](../../Std/Docu/Headers.odc.md)     headers / footers

[<u>StdLinks</u>](../../Std/Docu/Links.odc.md)    hyperlink views

[<u>StdLog</u>](../../Std/Docu/Log.odc.md)    standard output

[<u>StdMenuTool</u>](../../Std/Docu/MenuTool.odc.md)    menu tool

[<u>StdStamps</u>](../../Std/Docu/Stamps.odc.md)    date stamp views

[<u>StdTables</u>](../../Std/Docu/Tables.odc.md)     table controls

[<u>StdTabViews</u>](../../Std/Docu/TabViews.odc.md)     tabbed folder views

[<u>StdViewSizer</u>](../../Std/Docu/ViewSizer.odc.md)     set size of a view
