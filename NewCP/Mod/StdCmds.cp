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

    IMPORT Models, Views, Controllers, Documents, Windows, Files,
           TextModels, TextViews, Sequencers, HostDialog, iGui;

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


    (* ---- UTF-8 codec helpers -------------------------------------------- *)

    (** Decode one UTF-8 codepoint from `rd`.
        `b` IN: first byte of the current sequence (already loaded).
        `b` OUT: first byte of the next sequence (0 if eof).
        `cp` OUT: decoded codepoint 0..10FFFFh, or ORD('?') on error. *)
    PROCEDURE ReadUtf8Char (rd: Files.Reader; VAR b: BYTE; OUT cp: INTEGER);
        VAR bv, c1, c2, c3: INTEGER; cb: BYTE;
    BEGIN
        bv := b MOD 256;
        IF bv < 80H THEN
            (* Single-byte ASCII. *)
            cp := bv;
            rd.ReadByte(b)
        ELSIF bv < 0C0H THEN
            (* Spurious continuation byte — replace and advance. *)
            cp := ORD('?');
            rd.ReadByte(b)
        ELSIF bv < 0E0H THEN
            (* 2-byte: 110xxxxx 10xxxxxx *)
            cp := bv MOD 32;
            rd.ReadByte(cb); c1 := cb MOD 256;
            IF ~rd.eof & (c1 >= 80H) & (c1 < 0C0H) THEN
                cp := cp * 64 + (c1 MOD 64)
            ELSE cp := ORD('?')
            END;
            rd.ReadByte(b)
        ELSIF bv < 0F0H THEN
            (* 3-byte: 1110xxxx 10xxxxxx 10xxxxxx *)
            cp := bv MOD 16;
            rd.ReadByte(cb); c1 := cb MOD 256;
            rd.ReadByte(cb); c2 := cb MOD 256;
            IF ~rd.eof & (c1 >= 80H) & (c1 < 0C0H)
                       & (c2 >= 80H) & (c2 < 0C0H) THEN
                cp := ((cp * 64 + (c1 MOD 64)) * 64) + (c2 MOD 64)
            ELSE cp := ORD('?')
            END;
            rd.ReadByte(b)
        ELSE
            (* 4-byte: 11110xxx 10xxxxxx 10xxxxxx 10xxxxxx *)
            cp := bv MOD 8;
            rd.ReadByte(cb); c1 := cb MOD 256;
            rd.ReadByte(cb); c2 := cb MOD 256;
            rd.ReadByte(cb); c3 := cb MOD 256;
            IF ~rd.eof & (c1 >= 80H) & (c1 < 0C0H)
                       & (c2 >= 80H) & (c2 < 0C0H)
                       & (c3 >= 80H) & (c3 < 0C0H) THEN
                cp := (((cp * 64 + (c1 MOD 64)) * 64 + (c2 MOD 64)) * 64) + (c3 MOD 64)
            ELSE cp := ORD('?')
            END;
            rd.ReadByte(b)
        END
    END ReadUtf8Char;

    (** Encode codepoint `cp` as UTF-8 bytes and write to `wr`. *)
    PROCEDURE WriteUtf8Char (wr: Files.Writer; cp: INTEGER);
        VAR b: BYTE;
    BEGIN
        IF cp < 0 THEN RETURN END;
        IF cp < 80H THEN
            b := SHORT(cp); wr.WriteByte(b)
        ELSIF cp < 800H THEN
            b := SHORT(0C0H + cp DIV 64); wr.WriteByte(b);
            b := SHORT(80H + cp MOD 64);  wr.WriteByte(b)
        ELSIF cp < 10000H THEN
            b := SHORT(0E0H + cp DIV 4096);           wr.WriteByte(b);
            b := SHORT(80H + (cp DIV 64) MOD 64);     wr.WriteByte(b);
            b := SHORT(80H + cp MOD 64);               wr.WriteByte(b)
        ELSE
            b := SHORT(0F0H + cp DIV 262144);          wr.WriteByte(b);
            b := SHORT(80H + (cp DIV 4096) MOD 64);   wr.WriteByte(b);
            b := SHORT(80H + (cp DIV 64) MOD 64);     wr.WriteByte(b);
            b := SHORT(80H + cp MOD 64);               wr.WriteByte(b)
        END
    END WriteUtf8Char;


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
        (* Attach a fresh sequencer so Undo/Redo work on this document. *)
        IF Sequencers.dir # NIL THEN
            Models.SetSequencer(text, Sequencers.dir.New())
        END;
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
            cp:     INTEGER;
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

        (* Read the file as UTF-8 into a fresh text model.
           U+FEFF (BOM / ZWNBSP) and CR (0Dh) are skipped silently;
           LF (0Ah) maps to TextModels.line; tab (09h) and all
           printable codepoints pass through as-is. *)
        NEW(text);
        IF Sequencers.dir # NIL THEN
            Models.SetSequencer(text, Sequencers.dir.New())
        END;
        wr := text.NewWriter(NIL);
        rd.ReadByte(b);
        WHILE ~rd.eof DO
            ReadUtf8Char(rd, b, cp);
            IF (cp = 0DH) OR (cp = 0FEFFH) THEN
                (* CR or BOM: skip *)
            ELSIF cp = 0AH THEN
                wr.WriteChar(TextModels.line)
            ELSIF (cp >= 20H) OR (cp = 9) THEN
                wr.WriteChar(CHR(cp))
            END
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

    (** Save the focused text view to a user-chosen file.
        Falls back to the first valid window if no text view is focused.
        TextModels.line (0DX) is written as CR+LF; all other codepoints
        are encoded as UTF-8. *)
    PROCEDURE Save*;
        VAR filter:   ARRAY 4 OF SHORTCHAR;
            empty:    ARRAY 4 OF CHAR;
            path:     ARRAY maxPath OF CHAR;
            title:    ARRAY 256 OF CHAR;
            w:        Windows.Window;
            innerV:   Views.View;
            focV:     Views.View;
            m:        Models.Model;
            d:        Documents.Document;
            loc:      Files.Locator;
            f:        Files.File;
            wr:       Files.Writer;
            rd:       TextModels.Reader;
            ch:       CHAR;
            b:        BYTE;
    BEGIN
        (* Find the focused window so we can update its title after save. *)
        focV := Controllers.FocusView();
        w    := NIL;
        IF focV # NIL THEN
            w := Windows.first;
            WHILE w # NIL DO
                IF w.IsValid() THEN
                    d := w.ThisDoc();
                    IF (d # NIL) & (d.ThisView() = focV) THEN EXIT END
                END;
                w := w.next
            END
        END;

        (* Prefer the focused view's model; fall back to first valid window. *)
        IF (focV # NIL) & (focV IS TextViews.View) THEN
            m := focV(TextViews.View).ThisModel()
        ELSE
            m := NIL
        END;
        IF (m = NIL) OR ~(m IS TextModels.Model) THEN
            (* Fallback: walk window list. *)
            w := Windows.first;
            WHILE (w # NIL) & ~w.IsValid() DO w := w.next END;
            IF w = NIL THEN RETURN END;
            d := w.ThisDoc();
            IF d = NIL THEN RETURN END;
            innerV := d.ThisView();
            IF (innerV = NIL) OR ~(innerV IS TextViews.View) THEN RETURN END;
            m := innerV(TextViews.View).ThisModel()
        END;
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

        (* Write text model as UTF-8 with CR+LF line endings. *)
        rd := m(TextModels.Model).NewReader(NIL);
        IF rd = NIL THEN RETURN END;
        rd.SetPos(0);
        rd.ReadChar();
        WHILE ~rd.eot DO
            ch := rd.char;
            IF ch = TextModels.line THEN
                wr.WriteByte(0DH);
                wr.WriteByte(0AH)
            ELSE
                WriteUtf8Char(wr, ORD(ch))
            END;
            rd.ReadChar()
        END;

        (* Update the window title to the saved filename. *)
        IF w # NIL THEN
            ExtractName(path, title);
            w.SetTitle(title)
        END
    END Save;


    (* ---- CloseWin ---------------------------------------------------------- *)

    (** Close the window containing the currently-focused view.
        Falls back to the first valid window if the focused view
        cannot be matched to any open window. *)
    PROCEDURE CloseWin*;
        VAR focV: Views.View;
            w:    Windows.Window;
            d:    Documents.Document;
    BEGIN
        (* First pass: find the window that holds the focused view. *)
        focV := Controllers.FocusView();
        IF focV # NIL THEN
            w := Windows.first;
            WHILE w # NIL DO
                IF w.IsValid() THEN
                    d := w.ThisDoc();
                    IF (d # NIL) & (d.ThisView() = focV) THEN
                        w.Close();
                        RETURN
                    END
                END;
                w := w.next
            END
        END;
        (* Fallback: close the first still-open window. *)
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
