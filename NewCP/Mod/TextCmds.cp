MODULE TextCmds;
(*
   Text-editing commands — second slice.

   Every public procedure is a zero-parameter command that operates
   on the currently-focused text controller
   (`TextControllers.Focus()`).  Following BB's convention, all
   commands are silent no-ops when no text controller is in focus.

   This slice ships:
     SelectAll*   — extend the selection to span the whole text
     Deselect*    — collapse the selection to a zero-length range
     Cut*         — delete selection and write to clipboard
     Copy*        — copy selection to clipboard
     Paste*       — insert clipboard text at caret / over selection
     Find*, FindAgain*, Replace*  — EMPTY stubs (need full Scanner)
     Bold*, Italic*, Plain*       — EMPTY stubs (need attribute ops)
*)

    IMPORT TextModels, TextControllers, HostClipboard;


    (* ---- Helpers -------------------------------------------------------- *)

    (** Return the length of the focused text, or 0 if none. *)
    PROCEDURE FocusLen (): INTEGER;
        VAR c: TextControllers.Controller;
    BEGIN
        c := TextControllers.Focus();
        IF (c # NIL) & (c.text # NIL) THEN RETURN c.text.Length() END;
        RETURN 0
    END FocusLen;


    (* ---- Selection commands ------------------------------------------- *)

    (** Select all text in the focused view.
        pre: TextControllers.Focus() ≠ NIL (else no-op) *)
    PROCEDURE SelectAll*;
        VAR c: TextControllers.Controller; len: INTEGER;
    BEGIN
        c := TextControllers.Focus();
        IF (c # NIL) & (c.text # NIL) THEN
            len := c.text.Length();
            IF len > 0 THEN
                c.SetSelection(0, len)
            END
        END
    END SelectAll;


    (** Collapse the selection to the caret position (or position 0
        if no caret is set). *)
    PROCEDURE Deselect*;
        VAR c: TextControllers.Controller; pos: INTEGER;
    BEGIN
        c := TextControllers.Focus();
        IF c # NIL THEN
            pos := c.CaretPos();
            IF pos = TextControllers.none THEN pos := 0 END;
            c.SetSelection(pos, pos)
        END
    END Deselect;


    (* ---- Clipboard helpers -------------------------------------------- *)

    (** Copy characters in [beg, end) from `text` into a stack buffer
        and push it to the system clipboard.  Silently truncates if
        the selection exceeds ClipCapacity - 1 characters. *)
    PROCEDURE CopyRangeToClip (text: TextModels.Doc; beg, end: INTEGER);
        VAR buf: ARRAY HostClipboard.ClipCapacity OF CHAR;
            rd: TextModels.Reader;
            i, len: INTEGER;
    BEGIN
        IF beg >= end THEN RETURN END;
        len := end - beg;
        IF len >= HostClipboard.ClipCapacity THEN
            len := HostClipboard.ClipCapacity - 1
        END;
        rd := text.NewReader(NIL);
        IF rd = NIL THEN RETURN END;
        rd.SetPos(beg);
        i := 0;
        WHILE i < len DO
            rd.ReadChar();
            IF rd.eot THEN len := i; i := len  (* break *)
            ELSE buf[i] := rd.char; INC(i)
            END
        END;
        buf[i] := 0X;
        HostClipboard.SetText(buf)
    END CopyRangeToClip;

    (** Delete the selection if any; return the resulting caret position. *)
    PROCEDURE DeleteSelection (c: TextControllers.Controller;
                               doc: TextModels.Doc): INTEGER;
        VAR beg, end: INTEGER;
    BEGIN
        c.GetSelection(beg, end);
        IF beg < end THEN
            doc.DeleteRange(beg, end);
            c.SetCaret(beg)
        ELSE
            beg := c.CaretPos();
            IF beg = TextControllers.none THEN beg := 0 END
        END;
        RETURN beg
    END DeleteSelection;


    (* ---- Clipboard commands ------------------------------------------- *)

    (** Copy selection to clipboard then delete it. *)
    PROCEDURE Cut*;
        VAR c: TextControllers.Controller;
            doc: TextModels.Doc;
            beg, end: INTEGER;
    BEGIN
        c := TextControllers.Focus();
        IF (c = NIL) OR (c.text = NIL) OR ~(c.text IS TextModels.Doc) THEN RETURN END;
        doc := c.text(TextModels.Doc);
        c.GetSelection(beg, end);
        IF beg >= end THEN RETURN END;
        CopyRangeToClip(doc, beg, end);
        doc.DeleteRange(beg, end);
        c.SetCaret(beg)
    END Cut;

    (** Copy selection to clipboard (no deletion). *)
    PROCEDURE Copy*;
        VAR c: TextControllers.Controller;
            doc: TextModels.Doc;
            beg, end: INTEGER;
    BEGIN
        c := TextControllers.Focus();
        IF (c = NIL) OR (c.text = NIL) OR ~(c.text IS TextModels.Doc) THEN RETURN END;
        doc := c.text(TextModels.Doc);
        c.GetSelection(beg, end);
        IF beg >= end THEN RETURN END;
        CopyRangeToClip(doc, beg, end)
    END Copy;

    (** Insert clipboard text at caret, replacing any selection. *)
    PROCEDURE Paste*;
        VAR c: TextControllers.Controller;
            doc: TextModels.Doc;
            clip: ARRAY HostClipboard.ClipCapacity OF CHAR;
            pos, i: INTEGER;
    BEGIN
        c := TextControllers.Focus();
        IF (c = NIL) OR (c.text = NIL) OR ~(c.text IS TextModels.Doc) THEN RETURN END;
        doc := c.text(TextModels.Doc);
        IF ~HostClipboard.GetText(clip) THEN RETURN END;
        pos := DeleteSelection(c, doc);
        i := 0;
        WHILE (clip[i] # 0X) & (doc.len < TextModels.DocCapacity - 1) DO
            doc.InsertChar(pos, clip[i]);
            INC(pos); INC(i)
        END;
        c.SetCaret(pos)
    END Paste;


    (* ---- Search / replace stubs --------------------------------------- *)

    (** Open a search dialog.  EMPTY in this slice — requires the
        full TextMappers scanner and a host dialog surface. *)
    PROCEDURE Find*;
    BEGIN
    END Find;

    PROCEDURE FindAgain*;
    BEGIN
    END FindAgain;

    PROCEDURE Replace*;
    BEGIN
    END Replace;


    (* ---- Attribute-change stubs --------------------------------------- *)

    (** Set the selection's font to bold.  EMPTY in this slice —
        requires StdProperties and attribute-write operations. *)
    PROCEDURE Bold*;
    BEGIN
    END Bold;

    PROCEDURE Italic*;
    BEGIN
    END Italic;

    (** Remove bold/italic from the selection. *)
    PROCEDURE Plain*;
    BEGIN
    END Plain;


END TextCmds.
