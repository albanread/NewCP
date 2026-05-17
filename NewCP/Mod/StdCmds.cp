MODULE StdCmds;
(*
   Standard application commands — second slice.

   Provides the most commonly needed application-level operations:

     New*      — open a fresh empty text document in a new window
     Open*     — file-open dialog → read text file → new window
     Save*     — file-save dialog → write current window's text
     CloseWin* — close the frontmost open window
     Quit*     — exit the application

   Clipboard (Cut/Copy/Paste/SelectAll) and undo/redo (Undo/Redo)
   are deferred to later slices that require Controllers focus
   routing and Sequencers respectively.

   File reading/writing uses Latin-1 byte encoding: CHAR values
   above 0FFH are emitted as '?' on Save; bytes above 7FH on
   Open are passed through as-is since Latin-1 maps onto Unicode's
   first 256 code-points.  Full UTF-8 I/O is a follow-up.
*)

    IMPORT Models, Views, Documents, Windows, Files, TextModels, TextViews, HostDialog, iGui;

    CONST
        defWidth  = 800;   (** Default new-window width in DIPs  *)
        defHeight = 600;   (** Default new-window height in DIPs *)

        maxPath   = 1024;  (** Path buffer size in CHARs *)


    (* ---- Path helpers --------------------------------------------------- *)

    (** Extract the filename component from a full path.
        Scans backwards for '\' or '/' and copies everything after the
        last separator.  If no separator is found, copies the whole string. *)
    PROCEDURE ExtractName (IN path: ARRAY OF CHAR; OUT name: ARRAY OF CHAR);
        VAR i, sep, j: INTEGER;
    BEGIN
        sep := -1;
        i := 0;
        WHILE (i < LEN(path)) & (path[i] # 0X) DO
            IF (path[i] = '\') OR (path[i] = '/') THEN sep := i END;
            INC(i)
        END;
        j := 0;
        i := sep + 1;
        WHILE (i < LEN(path)) & (path[i] # 0X) & (j < LEN(name) - 1) DO
            name[j] := path[i]; INC(j); INC(i)
        END;
        name[j] := 0X
    END ExtractName;


    (* ---- New --------------------------------------------------------------- *)

    (** Create a fresh empty text document and open it in a new window.
        Silently returns if any required directory is absent — the
        framework guarantees the directories are set by the time the
        module body runs (Documents and TextViews self-install in their
        own BEGIN blocks). *)
    PROCEDURE New*;
        VAR text: TextModels.Doc;
            view: TextViews.View;
            doc:  Documents.Document;
            win:  Windows.Window;
    BEGIN
        NEW(text);
        IF TextViews.dir = NIL THEN RETURN END;
        view := TextViews.dir.New(text);
        IF view = NIL THEN RETURN END;
        IF Documents.dir = NIL THEN RETURN END;
        doc := Documents.dir.New(view, defWidth, defHeight);
        IF doc = NIL THEN RETURN END;
        win := Windows.Open(doc, "Untitled", defWidth, defHeight)
    END New;


    (* ---- Open -------------------------------------------------------------- *)

    (** Open a file-chooser dialog; if the user picks a file, read its
        bytes as Latin-1 text and show it in a new window.
        LF (0AH) and CR (0DH) are both mapped to TextModels.line (0DX).
        Non-printable control bytes (< 20H except HT 09H and line-ends)
        are discarded.  If Files.dir or TextViews.dir is NIL the call
        is a silent no-op. *)
    PROCEDURE Open*;
        VAR filter: ARRAY 4 OF SHORTCHAR;
            path:   ARRAY maxPath OF CHAR;
            title:  ARRAY 256 OF CHAR;
            loc:    Files.Locator;
            f:      Files.File;
            rd:     Files.Reader;
            text:   TextModels.Doc;
            wr:     TextModels.Writer;
            view:   TextViews.View;
            doc:    Documents.Document;
            win:    Windows.Window;
            b:      BYTE;
    BEGIN
        filter[0] := 0X;   (* empty → "All Files" in the Rust shim *)
        IF ~HostDialog.GetOpenFileName(filter, path) THEN RETURN END;
        IF path[0] = 0X THEN RETURN END;
        IF Files.dir = NIL THEN RETURN END;
        loc := Files.dir.This(path);
        IF loc = NIL THEN RETURN END;
        f := Files.dir.Old(loc, "", FALSE);
        IF f = NIL THEN RETURN END;
        rd := f.NewReader(NIL);
        IF rd = NIL THEN RETURN END;

        (* Read the file byte-by-byte into a fresh text model. *)
        NEW(text);
        wr := text.NewWriter(NIL);
        rd.ReadByte(b);
        WHILE ~rd.eof DO
            IF b = 0AH THEN
                wr.WriteChar(TextModels.line)
            ELSIF b = 0DH THEN
                (* swallow CR; LF (above) will insert the line break *)
            ELSIF (b >= 20H) OR (b = 9) THEN   (* printable or HT *)
                wr.WriteChar(CHR(b))
            END;
            rd.ReadByte(b)
        END;

        (* Title = last path component (filename). *)
        ExtractName(path, title);
        IF TextViews.dir = NIL THEN RETURN END;
        view := TextViews.dir.New(text);
        IF view = NIL THEN RETURN END;
        IF Documents.dir = NIL THEN RETURN END;
        doc := Documents.dir.New(view, defWidth, defHeight);
        IF doc = NIL THEN RETURN END;
        win := Windows.Open(doc, title, defWidth, defHeight)
    END Open;


    (* ---- Save -------------------------------------------------------------- *)

    (** Save the text from the first valid window to a user-chosen file.
        Characters <= 0FFH are written as a single Latin-1 byte.
        Characters > 0FFH are replaced by '?'.
        TextModels.line (0DX) is written as CR+LF (0DH + 0AH).
        If no valid window or TextViews.View is found, the call is a
        silent no-op. *)
    PROCEDURE Save*;
        VAR filter:   ARRAY 4 OF SHORTCHAR;
            empty:    ARRAY 4 OF CHAR;
            path:     ARRAY maxPath OF CHAR;
            w:        Windows.Window;
            innerV:   Views.View;
            m:        Models.Model;
            d:        Documents.Document;
            loc:      Files.Locator;
            f:        Files.File;
            wr:       Files.Writer;
            rd:       TextModels.Reader;
            ch:       CHAR;
            b:        BYTE;
    BEGIN
        (* Find first valid window. *)
        w := Windows.first;
        WHILE (w # NIL) & ~w.IsValid() DO w := w.next END;
        IF w = NIL THEN RETURN END;
        d := w.ThisDoc();
        IF d = NIL THEN RETURN END;
        innerV := d.ThisView();
        IF (innerV = NIL) OR ~(innerV IS TextViews.View) THEN RETURN END;
        m := innerV(TextViews.View).ThisModel();
        IF (m = NIL) OR ~(m IS TextModels.Model) THEN RETURN END;

        filter[0] := 0X;
        empty[0]  := 0X;
        IF ~HostDialog.GetSaveFileName(filter, empty, path) THEN RETURN END;
        IF path[0] = 0X THEN RETURN END;
        IF Files.dir = NIL THEN RETURN END;
        loc := Files.dir.This(path);
        IF loc = NIL THEN RETURN END;
        f := Files.dir.New(loc, FALSE);
        IF f = NIL THEN RETURN END;
        wr := f.NewWriter(NIL);
        IF wr = NIL THEN RETURN END;

        (* Write text model as Latin-1 with CR+LF line endings. *)
        rd := m(TextModels.Model).NewReader(NIL);
        IF rd = NIL THEN RETURN END;
        rd.SetPos(0);
        rd.ReadChar();
        WHILE ~rd.eot DO
            ch := rd.char;
            IF ch = TextModels.line THEN
                wr.WriteByte(0DH);
                wr.WriteByte(0AH)
            ELSIF ORD(ch) > 0FFH THEN
                wr.WriteByte(3FH)   (* '?' *)
            ELSE
                b := SHORT(ORD(ch));
                wr.WriteByte(b)
            END;
            rd.ReadChar()
        END
    END Save;


    (* ---- CloseWin ---------------------------------------------------------- *)

    (** Close the first valid (open) window in the window list.
        Walks past any already-closed entries (port = NIL → IsValid = FALSE)
        until it finds one that is still live. *)
    PROCEDURE CloseWin*;
        VAR w: Windows.Window;
    BEGIN
        w := Windows.first;
        WHILE (w # NIL) & ~w.IsValid() DO w := w.next END;
        IF w # NIL THEN w.Close() END
    END CloseWin;


    (* ---- Quit -------------------------------------------------------------- *)

    (** Post a quit request to the host event loop. *)
    PROCEDURE Quit*;
    BEGIN
        iGui.Quit
    END Quit;


END StdCmds.
