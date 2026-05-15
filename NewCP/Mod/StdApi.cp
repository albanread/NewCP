MODULE StdApi;
(*
   First slice of the BlackBox `StdApi` port.

   BB's StdApi (~470 lines) is the file-loading + dialog-opening
   workhorse behind StdCmds.  `StdCmds.OpenToolDialog` delegates
   to `StdApi.OpenToolDialog`, which:

     1. Maps the title string through Dialog.
     2. Asks `Windows.SelectByTitle` whether the dialog is
        already open (`done` → raise; nothing else to do).
     3. Otherwise loads `file` (e.g. "System/Rsrc/About") via
        the Files / Converters chain.
     4. Hands the resulting Views.View to `StdDialog.Open`.

   This slice ships `OpenToolDialog`, `OpenAuxDialog`,
   `OpenDoc`, `OpenAux`, `OpenBrowser`, `OpenCopyOf` with
   simplified bodies that route to `StdDialog.Open` when a
   view can be loaded.  The file-loading path drops to
   `Converters.Import` — which is itself a stub today, so the
   chain stops at "ConverterFailed" via Dialog.ShowMsg.  Once
   Converters' reflection dispatch lands, the welcome page
   loads end-to-end with no further changes here.

   Deferred: the symbol-spec parser (`CheckQualident`,
   `PathToSpec`, `ThisDialog`, `ThisMask` BB helpers) — current
   bodies pass a literal `file` string to Converters.Import,
   skipping the resource-path resolution.  Real path resolution
   lands once Files.dir.GetFileName / Locator parsing is wired.
*)

    IMPORT
        Files, Converters, Stores, Views,
        Windows, StdDialog;


    (** Copy a 0X-terminated ARRAY OF CHAR into a Files.Name
        slot.  CP's COPY builtin isn't on our compiler yet so we
        do it by hand. *)
    PROCEDURE CopyName (IN src: ARRAY OF CHAR; OUT dst: Files.Name);
        VAR i, cap: INTEGER;
    BEGIN
        cap := LEN(dst) - 1;
        i := 0;
        WHILE (i < cap) & (src[i] # 0X) DO
            dst[i] := src[i];
            INC(i)
        END;
        dst[i] := 0X
    END CopyName;


    (** OpenToolDialog: open `file` as a tool window with
        `title`.  Welcome-page chain enters here from
        `StdCmds.OpenToolDialog`. *)
    PROCEDURE OpenToolDialog* (IN file, title: ARRAY OF CHAR; OUT v: Views.View);
        VAR done: BOOLEAN; conv: Converters.Converter; s: Stores.Store;
            loc: Files.Locator; fname: Files.Name;
    BEGIN
        v := NIL;
        Windows.SelectByTitle(NIL, {Windows.isTool}, title, done);
        IF done THEN RETURN END;

        (* Path resolution deferred — treat `file` as a Files.Name. *)
        loc := NIL;
        CopyName(file, fname);
        conv := NIL;
        Converters.Import(loc, fname, conv, s);
        IF (s # NIL) & (s IS Views.View) THEN
            v := s(Views.View);
            StdDialog.Open(v, title, loc, fname, conv,
                           TRUE, FALSE, TRUE, FALSE, TRUE)
        END
    END OpenToolDialog;

    PROCEDURE OpenAuxDialog* (IN file, title: ARRAY OF CHAR; OUT v: Views.View);
        VAR done: BOOLEAN; conv: Converters.Converter; s: Stores.Store;
            loc: Files.Locator; fname: Files.Name;
    BEGIN
        v := NIL;
        Windows.SelectByTitle(NIL, {Windows.isAux}, title, done);
        IF done THEN RETURN END;

        loc := NIL;
        CopyName(file, fname);
        conv := NIL;
        Converters.Import(loc, fname, conv, s);
        IF (s # NIL) & (s IS Views.View) THEN
            v := s(Views.View);
            StdDialog.Open(v, title, loc, fname, conv,
                           FALSE, TRUE, TRUE, FALSE, TRUE)
        END
    END OpenAuxDialog;

    PROCEDURE OpenDoc* (IN file: ARRAY OF CHAR; OUT v: Views.View);
        VAR done: BOOLEAN; conv: Converters.Converter; s: Stores.Store;
            loc: Files.Locator; fname: Files.Name;
    BEGIN
        v := NIL;
        Windows.SelectBySpec(NIL, file, NIL, done);
        IF done THEN RETURN END;

        loc := NIL;
        CopyName(file, fname);
        conv := NIL;
        Converters.Import(loc, fname, conv, s);
        IF (s # NIL) & (s IS Views.View) THEN
            v := s(Views.View);
            StdDialog.Open(v, file, loc, fname, conv,
                           FALSE, FALSE, FALSE, FALSE, TRUE)
        END
    END OpenDoc;

    PROCEDURE OpenAux* (IN file, title: ARRAY OF CHAR; OUT v: Views.View);
    BEGIN
        OpenAuxDialog(file, title, v)
    END OpenAux;

    PROCEDURE OpenBrowser* (IN file, title: ARRAY OF CHAR; OUT v: Views.View);
    BEGIN
        OpenAuxDialog(file, title, v)
    END OpenBrowser;

    PROCEDURE OpenCopyOf* (IN file: ARRAY OF CHAR; OUT v: Views.View);
    BEGIN
        OpenDoc(file, v)
    END OpenCopyOf;

    PROCEDURE CloseDialog* (OUT closedView: Views.View);
    BEGIN
        closedView := NIL
    END CloseDialog;

END StdApi.
