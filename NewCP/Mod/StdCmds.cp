MODULE StdCmds;
(*
   First slice of the BlackBox `StdCmds` port.

   BB's StdCmds (~1900 lines) is the top-level command surface
   — every menu / shortcut entry point eventually calls one of
   these.  Most are wrappers around StdApi that the menu system
   wires up through Meta reflection.

   This slice ships the OpenToolDialog / OpenAuxDialog /
   OpenDoc / OpenAux / OpenBrowser / OpenCopyOf / CloseDialog
   wrappers — all delegate to StdApi.  Every other command
   (Undo / Redo / Cut / Paste / font ops / layout ops / …) is
   omitted; once the welcome page is on screen and a user-
   action chain works, the rest land alongside.

   Deferred: New, PasteView, Undo, Redo, CopyProp, PasteProp,
   Clear, SelectAll, DeselectAll, font and style commands,
   layout and mode commands, the entire guard / property
   constellation.
*)

    IMPORT Views, StdApi;


    (** Open `file` as a tool dialog with `title`.  Welcome
        page enters here: `StdCmds.OpenToolDialog("System/Rsrc/About",
        "About BlackBox")`. *)
    PROCEDURE OpenToolDialog* (IN file, title: ARRAY OF CHAR);
        VAR v: Views.View;
    BEGIN
        StdApi.OpenToolDialog(file, title, v)
    END OpenToolDialog;

    PROCEDURE OpenAuxDialog* (IN file, title: ARRAY OF CHAR);
        VAR v: Views.View;
    BEGIN
        StdApi.OpenAuxDialog(file, title, v)
    END OpenAuxDialog;

    PROCEDURE OpenDoc* (IN file: ARRAY OF CHAR);
        VAR v: Views.View;
    BEGIN
        StdApi.OpenDoc(file, v)
    END OpenDoc;

    PROCEDURE OpenAux* (IN file, title: ARRAY OF CHAR);
        VAR v: Views.View;
    BEGIN
        StdApi.OpenAux(file, title, v)
    END OpenAux;

    PROCEDURE OpenBrowser* (IN file, title: ARRAY OF CHAR);
        VAR v: Views.View;
    BEGIN
        StdApi.OpenBrowser(file, title, v)
    END OpenBrowser;

    PROCEDURE OpenCopyOf* (IN file: ARRAY OF CHAR);
        VAR v: Views.View;
    BEGIN
        StdApi.OpenCopyOf(file, v)
    END OpenCopyOf;

    PROCEDURE CloseDialog*;
        VAR v: Views.View;
    BEGIN
        StdApi.CloseDialog(v)
    END CloseDialog;

END StdCmds.
