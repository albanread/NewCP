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

    IMPORT Models, Sequencers, TextModels, TextControllers, HostClipboard, Fonts, Ports;

    VAR
        (** Last-used search term (set by Find from the selection). *)
        findTerm: ARRAY 256 OF CHAR;
        (** Position to start the next FindAgain search from. *)
        findFrom: INTEGER;


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


    (* ---- Search ------------------------------------------------------- *)

    (** Find: if there is a selection, use it as the new search term and
        search for the next occurrence from just after the selection.
        If there is no selection, re-run FindAgain using the last term.
        The search is case-sensitive and wraps around on failure. *)
    PROCEDURE Find*;
        VAR c: TextControllers.Controller;
            doc: TextModels.Doc;
            beg, end, i: INTEGER;
    BEGIN
        c := TextControllers.Focus();
        IF (c = NIL) OR (c.text = NIL) OR ~(c.text IS TextModels.Doc) THEN RETURN END;
        doc := c.text(TextModels.Doc);
        c.GetSelection(beg, end);
        IF beg < end THEN
            (* Copy selection into findTerm. *)
            i := 0;
            WHILE (beg + i < end) & (i < LEN(findTerm) - 1) DO
                findTerm[i] := doc.buf[beg + i]; INC(i)
            END;
            findTerm[i] := 0X;
            findFrom := end   (* start searching after the selection *)
        END;
        FindAgain
    END Find;

    (** Find next occurrence of findTerm starting from findFrom.
        Selects the match and advances findFrom past it.
        Wraps around to the beginning of the document if not found forward. *)
    PROCEDURE FindAgain*;
        VAR c: TextControllers.Controller;
            doc: TextModels.Doc;
            i, termLen, pos: INTEGER;
            found: BOOLEAN;
    BEGIN
        IF findTerm[0] = 0X THEN RETURN END;
        c := TextControllers.Focus();
        IF (c = NIL) OR (c.text = NIL) OR ~(c.text IS TextModels.Doc) THEN RETURN END;
        doc := c.text(TextModels.Doc);
        (* Measure the search term. *)
        termLen := 0;
        WHILE (termLen < LEN(findTerm)) & (findTerm[termLen] # 0X) DO INC(termLen) END;
        IF termLen = 0 THEN RETURN END;
        (* Forward search from findFrom. *)
        found := FALSE;
        pos := findFrom;
        WHILE (pos + termLen <= doc.len) & ~found DO
            i := 0;
            WHILE (i < termLen) & (doc.buf[pos + i] = findTerm[i]) DO INC(i) END;
            IF i = termLen THEN
                found := TRUE
            ELSE
                INC(pos)
            END
        END;
        IF ~found THEN
            (* Wrap around: search from 0 up to findFrom. *)
            pos := 0;
            WHILE (pos + termLen <= findFrom) & ~found DO
                i := 0;
                WHILE (i < termLen) & (doc.buf[pos + i] = findTerm[i]) DO INC(i) END;
                IF i = termLen THEN found := TRUE
                ELSE INC(pos)
                END
            END
        END;
        IF found THEN
            c.SetSelection(pos, pos + termLen);
            c.SetCaret(pos + termLen);
            findFrom := pos + termLen
        END
    END FindAgain;

    PROCEDURE Replace*;
    BEGIN
    END Replace;


    (* ---- Undo / Redo ------------------------------------------------- *)

    (** Undo the last edit in the focused text model's sequencer. *)
    PROCEDURE Undo*;
        VAR c: TextControllers.Controller;
            s: ANYPTR;
    BEGIN
        c := TextControllers.Focus();
        IF (c = NIL) OR (c.text = NIL) THEN RETURN END;
        s := c.text.seq;
        WITH s: Sequencers.Sequencer DO
            IF s.CanUndo() THEN s.Undo() END
        ELSE
        END
    END Undo;

    (** Redo the last undone edit. *)
    PROCEDURE Redo*;
        VAR c: TextControllers.Controller;
            s: ANYPTR;
    BEGIN
        c := TextControllers.Focus();
        IF (c = NIL) OR (c.text = NIL) THEN RETURN END;
        s := c.text.seq;
        WITH s: Sequencers.Sequencer DO
            IF s.CanRedo() THEN s.Redo() END
        ELSE
        END
    END Redo;


    (* ---- Attribute helpers -------------------------------------------- *)

    (** Resolve the base font for the selection, falling back through:
        1. the character attribute at `beg` (read via NewReader)
        2. the text directory's default attribute
        3. the Fonts directory's Default() font
        Returns NIL if no font can be determined. *)
    PROCEDURE SelectionBaseFont (doc: TextModels.Doc; beg: INTEGER): Fonts.Font;
        VAR rd: TextModels.Reader; f: Fonts.Font;
    BEGIN
        f := NIL;
        rd := doc.NewReader(NIL);
        IF rd # NIL THEN
            rd.SetPos(beg);
            rd.ReadChar();
            IF ~rd.eot & (rd.attr # NIL) & (rd.attr.font # NIL) THEN
                f := rd.attr.font
            END
        END;
        IF (f = NIL) & (TextModels.dir # NIL) & (TextModels.dir.attr # NIL) THEN
            f := TextModels.dir.attr.font
        END;
        IF (f = NIL) & (Fonts.dir # NIL) THEN
            f := Fonts.dir.Default()
        END;
        RETURN f
    END SelectionBaseFont;

    (** Create an Attributes record derived from `baseFont` but with
        `weight` and `style` overridden.  Uses the directory default
        color and zero offset. *)
    PROCEDURE DerivedAttr (baseFont: Fonts.Font; weight: INTEGER; style: SET):
        TextModels.Attributes;
        VAR tf: Fonts.Typeface; sz: INTEGER; newFont: Fonts.Font;
    BEGIN
        IF baseFont # NIL THEN
            tf := baseFont.typeface;
            sz := baseFont.size
        ELSE
            tf := Fonts.default;
            sz := 12 * Fonts.point
        END;
        IF Fonts.dir = NIL THEN RETURN NIL END;
        newFont := Fonts.dir.This(tf, sz, style, weight);
        RETURN TextModels.NewAttributes(0, newFont, 0)
    END DerivedAttr;


    (* ---- Attribute-change commands ------------------------------------ *)

    (** Set the selection's font weight to Bold (700).
        The base typeface, size, and italic state are preserved. *)
    PROCEDURE Bold*;
        VAR c: TextControllers.Controller;
            doc: TextModels.Doc;
            beg, end: INTEGER;
            base: Fonts.Font;
            attr: TextModels.Attributes;
            style: SET;
    BEGIN
        c := TextControllers.Focus();
        IF (c = NIL) OR (c.text = NIL) OR ~(c.text IS TextModels.Doc) THEN RETURN END;
        doc := c.text(TextModels.Doc);
        c.GetSelection(beg, end);
        IF beg >= end THEN RETURN END;
        base := SelectionBaseFont(doc, beg);
        IF base # NIL THEN style := base.style ELSE style := {} END;
        attr := DerivedAttr(base, Fonts.bold, style);
        IF attr # NIL THEN doc.SetAttrRange(beg, end, attr) END
    END Bold;

    (** Set the selection's font style to Italic.
        The base typeface, size, and weight are preserved. *)
    PROCEDURE Italic*;
        VAR c: TextControllers.Controller;
            doc: TextModels.Doc;
            beg, end: INTEGER;
            base: Fonts.Font;
            attr: TextModels.Attributes;
            weight: INTEGER;
    BEGIN
        c := TextControllers.Focus();
        IF (c = NIL) OR (c.text = NIL) OR ~(c.text IS TextModels.Doc) THEN RETURN END;
        doc := c.text(TextModels.Doc);
        c.GetSelection(beg, end);
        IF beg >= end THEN RETURN END;
        base := SelectionBaseFont(doc, beg);
        IF base # NIL THEN weight := base.weight ELSE weight := Fonts.normal END;
        attr := DerivedAttr(base, weight, {Fonts.italic});
        IF attr # NIL THEN doc.SetAttrRange(beg, end, attr) END
    END Italic;

    (** Remove bold and italic from the selection — reset to plain.
        Sets attr to NIL so the view's default attribute applies. *)
    PROCEDURE Plain*;
        VAR c: TextControllers.Controller;
            doc: TextModels.Doc;
            beg, end: INTEGER;
    BEGIN
        c := TextControllers.Focus();
        IF (c = NIL) OR (c.text = NIL) OR ~(c.text IS TextModels.Doc) THEN RETURN END;
        doc := c.text(TextModels.Doc);
        c.GetSelection(beg, end);
        IF beg >= end THEN RETURN END;
        doc.SetAttrRange(beg, end, NIL)   (* NIL = use view's default attr *)
    END Plain;


BEGIN
    findTerm[0] := 0X;
    findFrom := 0

END TextCmds.
