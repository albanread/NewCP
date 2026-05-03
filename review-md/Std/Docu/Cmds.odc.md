**StdCmds**

DEFINITION StdCmds;

    IMPORT Dialog, Ports, Fonts;

    VAR

        allocator: Dialog.String;

        layout: RECORD

            wType, hType: INTEGER;

            width, height: REAL

        END;

        size: RECORD

            size: INTEGER

        END;

    PROCEDURE Bold;

    PROCEDURE BoldGuard (VAR par: Dialog.Par);

    PROCEDURE CaretGuard (VAR par: Dialog.Par);

    PROCEDURE Clear;

    PROCEDURE CloseDialog;

    PROCEDURE Color (color: Ports.Color);

    PROCEDURE ColorGuard (color: INTEGER; VAR par: Dialog.Par);

    PROCEDURE ContainerGuard (VAR par: Dialog.Par);

    PROCEDURE CopyProp;

    PROCEDURE DefaultFont;

    PROCEDURE DefaultFontGuard (VAR par: Dialog.Par);

    PROCEDURE DefaultOnDoubleClick (op, from, to: INTEGER);

    PROCEDURE DeselectAll;

    PROCEDURE Font (typeface: Fonts.Typeface);

    PROCEDURE HeightGuard (VAR par: Dialog.Par);

    PROCEDURE InitLayoutDialog;

    PROCEDURE InitSizeDialog;

    PROCEDURE Italic;

    PROCEDURE ItalicGuard (VAR par: Dialog.Par);

    PROCEDURE ModelViewGuard (VAR par: Dialog.Par);

    PROCEDURE New;

    PROCEDURE NewWindow;

    PROCEDURE Open;

    PROCEDURE OpenAsAuxDialog;

    PROCEDURE OpenAsToolDialog;

    PROCEDURE OpenAux (file, title: ARRAY OF CHAR);

    PROCEDURE OpenAuxDialog (file, title: ARRAY OF CHAR);

    PROCEDURE OpenBrowser (file, title: ARRAY OF CHAR);

    PROCEDURE OpenDoc (file: ARRAY OF CHAR);

    PROCEDURE OpenToolDialog (file, title: ARRAY OF CHAR);

    PROCEDURE PasteCharGuard (VAR par: Dialog.Par);

    PROCEDURE PasteLCharGuard (VAR par: Dialog.Par);

    PROCEDURE PasteProp;

    PROCEDURE PasteView;

    PROCEDURE PasteViewGuard (VAR par: Dialog.Par);

    PROCEDURE Plain;

    PROCEDURE PlainGuard (VAR par: Dialog.Par);

    PROCEDURE ReadOnlyGuard (VAR par: Dialog.Par);

    PROCEDURE RecalcAllSizes;

    PROCEDURE RecalcFocusSize;

    PROCEDURE Redo;

    PROCEDURE RedoGuard (VAR par: Dialog.Par);

    PROCEDURE RestoreAll;

    PROCEDURE SelectAll;

    PROCEDURE SelectAllGuard (VAR par: Dialog.Par);

    PROCEDURE SelectDocument;

    PROCEDURE SelectNextView;

    PROCEDURE SelectionGuard (VAR par: Dialog.Par);

    PROCEDURE SetBrowserMode;

    PROCEDURE SetBrowserModeGuard (VAR par: Dialog.Par);

    PROCEDURE SetEditMode;

    PROCEDURE SetEditModeGuard (VAR par: Dialog.Par);

    PROCEDURE SetLayout;

    PROCEDURE SetLayoutMode;

    PROCEDURE SetLayoutModeGuard (VAR par: Dialog.Par);

    PROCEDURE SetMaskMode;

    PROCEDURE SetMaskModeGuard (VAR par: Dialog.Par);

    PROCEDURE SetSize;

    PROCEDURE ShowProp;

    PROCEDURE ShowPropGuard (VAR par: Dialog.Par);

    PROCEDURE SingletonGuard (VAR par: Dialog.Par);

    PROCEDURE Size (size: INTEGER);

    PROCEDURE SizeGuard (size: INTEGER; VAR par: Dialog.Par);

    PROCEDURE Strikeout;

    PROCEDURE StrikeoutGuard (VAR par: Dialog.Par);

    PROCEDURE ToggleNoFocus;

    PROCEDURE ToggleNoFocusGuard (VAR par: Dialog.Par);

    PROCEDURE TypeNotifier (op, from, to: INTEGER);

    PROCEDURE TypefaceGuard (VAR par: Dialog.Par);

    PROCEDURE Underline;

    PROCEDURE UnderlineGuard (VAR par: Dialog.Par);

    PROCEDURE Undo;

    PROCEDURE UndoGuard (VAR par: Dialog.Par);

    PROCEDURE UpdateAll;

    PROCEDURE WidthGuard (VAR par: Dialog.Par);

    PROCEDURE WindowGuard (VAR par: Dialog.Par);

END StdCmds.

*StdCmds* is a command package which contains many commands and guards which are typically used in menu items (->StdMenuTool) or in hyperlinks (->StdLinks).

The module exports more commands which are not described here; these are the commands and guards for menu items of the *Windows* standard menus. See the [<u>Menus</u>](../../System/Rsrc/Menus.odc.md) text for how these commands are used (only on Windows; under Mac OS, the standard commands do not appear explicitly in the menu specifications).

VAR **allocator**: Dialog.String

This string contains the command sequence that is executed when the user invokes the *Files->New* command. By default, it is set to "TextViews.Deposit; StdCmds.Open". This string may be changed in the *Config* module.

VAR **layout**: RECORD

Interactor for root view layout (*Tools->Document Size...*). The size of the outermost view of a document (root view) may be determined in several different ways. Either it is equal to the window size, to the paper size minus the margins defined in the *Page Setup* dialog, or to a particular fixed size. The vertical and horizontal directions can be defined independently.

**wType, hType**: INTEGER    wType IN {0..2}  &  hType IN {0..2}

Determines whether the root view size (horizontical/vertical) has a fixed size (0), the size defined by the *Page Setup* dialog (1), or by the window (2).

**width, height**: REAL    valid iff wType/hType = 1

The root view width/height in centimeters.

VAR **size**: RECORD

Interactor for setting the font size.

**size**: INTEGER    size >= 6

Font size in points (-> Fonts).

PROCEDURE **Bold**

PROCEDURE **BoldGuard** (VAR par: Dialog.Par)

PROCEDURE **CaretGuard** (VAR par: Dialog.Par)

Disables menu item if there is no current caret, i.e., no selection of length 0.

PROCEDURE **Clear**

PROCEDURE **CloseDialog**

This command can be used from within a control to close the window in which the control is embedded. It can only be used as reaction to interactive manipulation of the control, i.e., when the  mouse is clicked in the control or when it receives keyboard input. If the window is a document window and its contents dirty, an error message will be displayed.

Pre

must be called in interaction with control    20

PROCEDURE **Color** (color: Ports.Color)

Set selection to the given color value (-> Ports.Color).

PROCEDURE **ColorGuard** (color: INTEGER; VAR par: Dialog.Par)

Guard for the *Color* command.

PROCEDURE **CopyProp**

Collect properties of current selection, so that they can be used later on for pasting them again.

PROCEDURE **ContainerGuard** (VAR par: Dialog.Par)

Guard for making sure that the focus is a container view (-> Containers.View).

PROCEDURE **DefaultFont**

Set selection to the default typeface.

PROCEDURE **DefaultFontGuard** (VAR par: Dialog.Par)

Guard for the *DefaultFont* command.

PROCEDURE **DefaultOnDoubleClick** (op, from, to: INTEGER)

A standard notifier which "clicks" the default button when its control is double-clicked. Typically, this is used in selection boxes or combo boxes.

PROCEDURE **DeselectAll**

Remove the selection in the focus view.

PROCEDURE Font (typeface: Fonts.Typeface)

PROCEDURE **HeightGuard** (VAR par: Dialog.Par)

Guard command which sets *par.readOnly* if *layout.hType # 0*, i.e., if mode is not "fixed".

PROCEDURE **InitLayoutDialog**

Initialization command for *layout* interactor.

PROCEDURE **InitSizeDialog**

Initialization command for *size* interactor.

PROCEDURE **Italic**

Set selection to italicized text.

PROCEDURE **ItalicGuard** (VAR par: Dialog.Par)

Guard for *Italic* command.

PROCEDURE **ModelViewGuard** (VAR par: Dialog.Par)

Guard for *NewWindow* command (checks that the focus view's *ThisModel* method does not return *NIL*).

PROCEDURE **New**

Open a new untitled document, using the command sequence in variable *allocator*.

PROCEDURE **NewWindow**

Guard: ModelViewGuard

Opens another window onto the same document as the front window's.

PROCEDURE **Open**

Guard: a view was deposited

Takes a deposited view from the global view queue and opens it in a new window.

PROCEDURE **OpenAsAuxDialog**

Opens another window onto the same document as the front window's; this window works as an auxiliary dialog box.

PROCEDURE **OpenAsToolDialog**

Opens another window onto the same document as the front window's; this window works as a tool dialog box.

PROCEDURE **OpenAux** (file, title: ARRAY OF CHAR)

Takes a file specification of an BlackBox document and a window title as parameters, and opens an auxiliary window with the specified title. Parameter *file* must be the path name of a file.

Auxiliary windows are used for displaying temporary editable data.

Example:

"StdCmds.OpenAux('Form/Rsrc/Menus', 'Form Menus')"

PROCEDURE **OpenAuxDialog** (file, title: ARRAY OF CHAR)

Takes a file specification of an BlackBox document and a window title as parameters, and opens a dialog with the specified document. The dialog is opened as an auxiliary window. Parameter *file* must be the path name of a file.

Auxiliary dialogs are used for self-contained dialogs, i.e., data entry masks or parameter entry masks for commands.

In contrast to *OpenAux*, *OpenAuxDialog* turns the opened document into *mask* mode if it is a container (-> Containers), and opens it into a dialog window if the underlying platform distinguishes between document and dialog windows (e.g. as in Windows).

Example:

"StdCmds.OpenAuxDialog('Form/Rsrc/Cmds', 'New Form')"

PROCEDURE **OpenBrowser** (file, title: ARRAY OF CHAR)

Takes a file specification of an BlackBox document and a window title as parameters, and opens the specified document in an auxiliary window. The window contents is not editable, but can be selected. Parameter *file* must be the path name of a file.

Browser windows are used for displaying documentation. It is possible to select and copy out browser window contents. It is possible to apply find & replace commands, the *Info->Interface* command, etc.

Example:

"StdCmds.OpenBrowser('Form/Docu/Models', 'FormModels Docu')"

PROCEDURE **OpenDoc** (file: ARRAY OF CHAR)

Takes a file specification of an BlackBox document as parameter, and opens the document in a document window. Parameter *file* must be the path name of a file. If the same file is already open in an existing document window, this window is brought to the top, instead of opening a new window.

Document windows are used for displaying persistent editable data.

Example:

"StdCmds.OpenDoc('System/Rsrc/Menus')"

PROCEDURE **OpenToolDialog** (file, title: ARRAY OF CHAR)

Takes a file specification of an BlackBox document and a window title as parameters, and opens a dialog with the specified document. The dialog is opened as tool window. Parameter *file* must be the path name of a file.

Tool dialogs are used for dialogs which operate on document windows beneath them, e.g., a Find & Replace dialog which operates on a text under it. Otherwise it is identical to *OpenAuxDialog*.

Example:

"DevInspector.InitDialog; StdCmds.OpenToolDialog('Dev/Rsrc/Inspect', 'Inspect')"

PROCEDURE **PasteCharGuard** (VAR par: Dialog.Par)

Disables menu item if entering a character is currently not possible.

PROCEDURE **PasteLCharGuard** (VAR par: Dialog.Par)

Obsolete.

PROCEDURE **PasteProp**

Guard: a property was collected (-> *CopyProp*)

Takes the previously copied properties and applies them to the current selection.

PROCEDURE **PasteView**

Guard: a view was deposited

Takes a deposited view from the global view queue and pastes it to the focus.

PROCEDURE **PasteViewGuard** (VAR par: Dialog.Par)

Disables menu item if pasting a view is currently not possible.

PROCEDURE **Plain**

Sets the selection to plain text (not italicized, not bold, not underlined).

PROCEDURE **PlainGuard**

Guard for the *Plain* command.

PROCEDURE **ReadOnlyGuard** (VAR par: Dialog.Par)

Makes a control read-only, by setting *par.readOnly* to *TRUE*.

PROCEDURE **RecalcAllSizes**

Recalculates the sizes of all visible views. Must be called after font metrics are changed.

PROCEDURE **RecalcFocusSize**

Recalculates the size of the focus view.

PROCEDURE **Redo**

Redo the most recently undone command.

PROCEDURE **RedoGuard**

Guard for the *Redo* command.

PROCEDURE **RestoreAll**

Forces a restoration of all visible views.

PROCEDURE **SelectAll**

Selects the whole contents of the focus view.

PROCEDURE **SelectAllGuard**

Guard for the *SelectAll* command.

PROCEDURE **SelectDocument**

Selects the root view (the outermost container) of the front window.

PROCEDURE **SelectNextView**

If a singleton view is selected in a container, this command selects the next view in the container in a round-robin fashion.

PROCEDURE **SelectionGuard** (VAR par: Dialog.Par)

Disables menu item if there is no current selection.

PROCEDURE **SetBrowserMode**

Guard: SetBrowserModeGuard

Sets the selected container in browser mode. If no container is selected the outer most container is set in browser mode. This mode is common for texts used for documentation purposes.

PROCEDURE **SetBrowserModeGuard** (VAR par: Dialog.Par)

Guard for procedure *SetBrowserMode*. Sets *par.checked* if appropriate.

PROCEDURE **SetEditMode**

Guard: SetEditModeGuard

Sets the selected container in edit mode. If no container is selected the outer most container is set in edit mode. This mode is common for editable texts.

PROCEDURE **SetEditModeGuard** (VAR par: Dialog.Par)

Guard for procedure *SetEditMode*. Sets *par.checked* if appropriate.

PROCEDURE **SetLayout**

Applies the interactor values in *layout*. To set up layout to the selected view, *InitLayoutDialog* needs to be called first.

PROCEDURE **SetLayoutMode**

Guard: SetLayoutModeGuard

Sets the selected container in layout mode. If no container is selected the outer most container is set in layout mode. This mode is common for editable forms.

PROCEDURE **SetLayoutModeGuard** (VAR par: Dialog.Par)

Guard for procedure *SetLayoutMode*. Sets *par.checked* if appropriate.

PROCEDURE **SetMaskMode**

Guard: SetMaskModeGuard

Sets the selected container in mask mode. If no container is selected the outer most container is set in mask mode. This mode is common for forms used as dialogs.

PROCEDURE **SetMaskModeGuard** (VAR par: Dialog.Par)

Guard for procedure *SetMaskMode*. Sets *par.checked* if appropriate.

PROCEDURE **SetSize**

Command for setting the selection to the size given in the *size* interactor.

PROCEDURE **ShowProp**

Shows the properties of the selection in the focus view.

PROCEDURE **ShowPropGuard**

Guard for the *ShowProp* command.

PROCEDURE **SingletonGuard** (VAR par: Dialog.Par)

Disables menu item if there is no current selection, or if the selection doesn't encompass exactly one embedded view.

PROCEDURE **Size** (size: INTEGER)

Sets the selection to the given size, in points (-> Fonts).

PROCEDURE **SizeGuard** (size: INTEGER; VAR par: Dialog.Par)

Guard for the *Size* command.

PROCEDURE **Strikeout**

Sets the selection to the strikeout style.

PROCEDURE **StrikeoutGuard** (VAR par: Dialog.Par)

Guard for the *Strikeout* command.

PROCEDURE **ToggleNoFocus**

Makes a focusable container non-focusable, or vice versa, by changing its mode in an appropriate way (-> Containers).

PROCEDURE **ToggleNoFocusGuard** (VAR par: Dialog.Par)

Guard for the *ToggleNoFocus* command.

PROCEDURE **TypeNotifier** (op, from, to: INTEGER)

PROCEDURE **TypefaceGuard** (VAR par: Dialog.Par)

Guard for the *Font* command.

PROCEDURE **Underline**

Sets the selection to the underline style.

PROCEDURE **UnderlineGuard** (VAR par: Dialog.Par)

Guard for the *Underline* command.

PROCEDURE **Undo**

Undoes the most recent command.

PROCEDURE **UndoGuard** (VAR par: Dialog.Par)

Guard for the *Undo* command.

PROCEDURE **UpdateAll**

PROCEDURE **WidthGuard** (VAR par: Dialog.Par)

Guard command which sets *par.readOnly* if *layout.wType # 0*, i.e., if mode is not "fixed".

PROCEDURE **WindowGuard** (VAR par: Dialog.Par)

