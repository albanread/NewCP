**StdApi**

DEFINITION StdApi;

    IMPORT Views;

    PROCEDURE CloseDialog (OUT closedView: Views.View);

    PROCEDURE OpenAux (file, title: ARRAY OF CHAR; OUT v: Views.View);

    PROCEDURE OpenAuxDialog (file, title: ARRAY OF CHAR; OUT v: Views.View);

    PROCEDURE OpenBrowser (file, title: ARRAY OF CHAR; OUT v: Views.View);

    PROCEDURE OpenCopyOf (file: ARRAY OF CHAR; OUT v: Views.View);

    PROCEDURE OpenDoc (file: ARRAY OF CHAR; OUT v: Views.View);

    PROCEDURE OpenToolDialog (file, title: ARRAY OF CHAR; OUT v: Views.View);

END StdApi.

*StdApi *is programming interface to many of the services provided by *StdCmds*. Since *StdCmds* is intended to be used in menus, hyperlinks etc., it is not always suitable as a programming interface. *StdApi* provides the exact same functionality but with a more "programmable" interface.

PROCEDURE **CloseDialog **(OUT closedView: Views.View)

This command closes the currently focused window. If the window is a document window and its contents dirty, an error message will be displayed. The root view of the closed window is returned in *closedView*. If *CloseDialog* failed *NIL* is returned.

PROCEDURE **OpenAux** (file, title: ARRAY OF CHAR; OUT v: Views.View)

Takes a file specification of a BlackBox document and a window title as parameters, and opens an auxiliary window with the specified title. Parameter *file* must be the path name of a file. The root view of the opened window is returned in *v*. If *OpenAux* fails *NIL* is returned in *v*.

Auxiliary windows are used for displaying temporary editable data.

PROCEDURE **OpenAuxDialog** (file, title: ARRAY OF CHAR; OUT v: Views.View)

Takes a file specification of a BlackBox document and a window title as parameters, and opens a dialog with the specified document. The dialog is opened as an auxiliary window. Parameter *file* must be the path name of a file. The root view of the opened window is returned in *v*. If *OpenAuxDialog* fails *NIL* is returned in *v*.

Auxiliary dialogs are used for self-contained dialogs, i.e., data entry masks or parameter entry masks for commands.

In contrast to *OpenAux*, *OpenAuxDialog* turns the opened document into *mask* mode if it is a container (-> Containers), and opens it into a dialog window if the underlying platform distinguishes between document and dialog windows (e.g. as in Windows).

PROCEDURE **OpenBrowser** (file, title: ARRAY OF CHAR; OUT v: Views.View)

Takes a file specification of a BlackBox document and a window title as parameters, and opens the specified document in an auxiliary window. The window contents is not editable, but can be selected. Parameter *file* must be the path name of a file. The root view of the opened window is returned in *v*. If *OpenBrowser* fails *NIL* is returned in *v*.

Browser windows are used for displaying documentation. It is possible to select and copy out browser window contents. It is possible to apply find & replace commands, the *Info->Interface* command, etc.

PROCEDURE **OpenCopyOf** (file: ARRAY OF CHAR; OUT v: Views.View)

Opens a new document and uses the document specified by *file* as a template to initialize the new document. If successful the root view of the new document is returned in *v* otherwise *NIL* is returned in *v*.

PROCEDURE **OpenDoc** (file: ARRAY OF CHAR; OUT v: Views.View)

Takes a file specification of a BlackBox document as parameter, and opens the document in a document window. Parameter *file* must be the path name of a file. If the same file is already open in an existing document window, this window is brought to the top, instead of opening a new window. The root view of the opened window (or window brought to the top) is returned in *v*. If *OpenDoc* fails *NIL* is returned in *v*.

Document windows are used for displaying persistent editable data.

PROCEDURE **OpenToolDialog** (file, title: ARRAY OF CHAR; OUT v: Views.View)

Takes a file specification of an BlackBox document and a window title as parameters, and opens a dialog with the specified document. The dialog is opened as tool window. Parameter *file* must be the path name of a file. The root view of the opened window is returned in *v*. If *OpenAuxDialog* fails *NIL* is returned in *v*.

Tool dialogs are used for dialogs which operate on document windows beneath them, e.g., a Find & Replace dialog which operates on a text under it. Otherwise it is identical to *OpenAuxDialog*.

