MODULE Documents;
(*
   BlackBox `Documents` port — Pass 1.

   Implements the concrete `StdDocument` (wraps an inner View with page
   geometry), `StdContext` (the Models.Context the inner view sees),
   and `StdDirectory` (factory).

   `ImportDocument` validates the .odc 4-byte tag and 2-byte version
   from the Files.Reader, then returns NIL — the Stores deserialisation
   path from a Files.File is not yet wired; that step requires either
   exposing the file path to `Stores.OpenDocument` or adding a
   `Stores.Reader` constructor that wraps a `Files.Reader`.

   Deferred:
   - Real ImportDocument / ExportDocument bodies (Stores-from-File path).
   - GetNewFrame returning a real RootFrame (needs RootFrame type in Views).
   - Print / PrinterContext / Pager (page-print machinery).
   - SetRectOp / SetPageOp / ReplaceViewOp (undoable operations).
*)

    IMPORT Stores, Models, Views, Ports, Containers, Files;

    CONST
        plain*    = FALSE;
        decorate* = TRUE;

        pageWidth*  = 16;
        pageHeight* = 17;
        winWidth*   = 18;
        winHeight*  = 19;

        defB*       = 8 * Ports.point;
        scrollUnit* = 16 * Ports.point;

        docTag*     = 6F4F4443H;
        docVersion* = 0;

        minVersion = 0;
        maxVersion = 0;


    TYPE
        DocumentDesc* = ABSTRACT RECORD (Containers.ViewDesc) END;
        Document*     = POINTER TO DocumentDesc;

        ContextDesc* = ABSTRACT RECORD (Models.ContextDesc) END;
        Context*     = POINTER TO ContextDesc;

        DirectoryDesc* = ABSTRACT RECORD END;
        Directory*     = POINTER TO DirectoryDesc;

        StdContextDesc = RECORD (ContextDesc)
            doc: StdDocument
        END;
        StdContext = POINTER TO StdContextDesc;

        StdDocumentDesc = EXTENSIBLE RECORD (DocumentDesc)
            view:             Views.View;
            w, h:             INTEGER;
            vl, vt, vr, vb:  INTEGER;
            pw, ph:           INTEGER;
            pl, pt, pr, pb:  INTEGER;
            decorate:         BOOLEAN;
            context:          StdContext
        END;
        StdDocument = POINTER TO StdDocumentDesc;

        StdDirectoryDesc = RECORD (DirectoryDesc) END;
        StdDirectory     = POINTER TO StdDirectoryDesc;


    VAR
        dir-:       Directory;
        stdDir-:    Directory;
        defaultDir: StdDirectory;   (* private backing; set in BEGIN *)


    (* -- Document abstract overrides --------------------------------------- *)

    (* Documents wrap a View, not a Model — no model is ever acceptable. *)
    PROCEDURE (d: Document) AcceptableModel* (m: Containers.Model): BOOLEAN;
    BEGIN RETURN FALSE END AcceptableModel;

    PROCEDURE (d: Document) GetNewFrame* (VAR frame: Views.Frame);
    BEGIN frame := NIL END GetNewFrame;

    PROCEDURE (d: Document) GetBackground* (VAR color: Ports.Color);
    BEGIN color := Ports.background END GetBackground;

    PROCEDURE (d: Document) DocCopyOf*
        (v: Views.View): Document, NEW, ABSTRACT;
    PROCEDURE (d: Document) SetView*
        (view: Views.View; w, h: INTEGER), NEW, ABSTRACT;
    PROCEDURE (d: Document) ThisView*     (): Views.View, NEW, ABSTRACT;
    PROCEDURE (d: Document) OriginalView* (): Views.View, NEW, ABSTRACT;
    PROCEDURE (d: Document) SetRect*
        (l, t, r, b: INTEGER), NEW, ABSTRACT;
    PROCEDURE (d: Document) PollRect*
        (VAR l, t, r, b: INTEGER), NEW, ABSTRACT;
    PROCEDURE (d: Document) SetPage*
        (w, h, l, t, r, b: INTEGER; decorate: BOOLEAN), NEW, ABSTRACT;
    PROCEDURE (d: Document) PollPage*
        (VAR w, h, l, t, r, b: INTEGER; VAR decorate: BOOLEAN), NEW, ABSTRACT;


    (* -- Context abstract method ------------------------------------------- *)

    PROCEDURE (c: Context) ThisDoc* (): Document, NEW, ABSTRACT;


    (* -- Directory abstract method ----------------------------------------- *)

    PROCEDURE (d: Directory) New*
        (view: Views.View; w, h: INTEGER): Document, NEW, ABSTRACT;


    (* -- StdDocument ------------------------------------------------------- *)

    PROCEDURE (d: StdDocument) SetView* (view: Views.View; w, h: INTEGER);
    BEGIN
        d.view := view;
        d.w    := w;
        d.h    := h;
        IF view # NIL THEN view.InitContext(d.context) END
    END SetView;

    PROCEDURE (d: StdDocument) ThisView* (): Views.View;
    BEGIN RETURN d.view END ThisView;

    PROCEDURE (d: StdDocument) OriginalView* (): Views.View;
    BEGIN RETURN d.view END OriginalView;

    PROCEDURE (d: StdDocument) SetRect* (l, t, r, b: INTEGER);
    BEGIN d.vl := l; d.vt := t; d.vr := r; d.vb := b END SetRect;

    PROCEDURE (d: StdDocument) PollRect* (VAR l, t, r, b: INTEGER);
    BEGIN l := d.vl; t := d.vt; r := d.vr; b := d.vb END PollRect;

    PROCEDURE (d: StdDocument) SetPage*
        (w, h, l, t, r, b: INTEGER; decorate: BOOLEAN);
    BEGIN
        d.pw := w; d.ph := h;
        d.pl := l; d.pt := t; d.pr := r; d.pb := b;
        d.decorate := decorate
    END SetPage;

    PROCEDURE (d: StdDocument) PollPage*
        (VAR w, h, l, t, r, b: INTEGER; VAR decorate: BOOLEAN);
    BEGIN
        w := d.pw; h := d.ph;
        l := d.pl; t := d.pt; r := d.pr; b := d.pb;
        decorate := d.decorate
    END PollPage;

    PROCEDURE (d: StdDocument) DocCopyOf* (v: Views.View): Document;
        VAR copy: StdDocument; ctx: StdContext;
    BEGIN
        NEW(copy); NEW(ctx);
        ctx.doc      := copy;
        copy.context := ctx;
        copy.SetView(v, d.w, d.h);
        copy.vl := d.vl; copy.vt := d.vt; copy.vr := d.vr; copy.vb := d.vb;
        copy.pw := d.pw; copy.ph := d.ph;
        copy.pl := d.pl; copy.pt := d.pt; copy.pr := d.pr; copy.pb := d.pb;
        copy.decorate := d.decorate;
        RETURN copy
    END DocCopyOf;

    PROCEDURE (d: StdDocument) Restore*
        (f: Views.Frame; l, t, r, b: INTEGER);
    BEGIN
        IF d.view # NIL THEN d.view.Restore(f, l, t, r, b) END
    END Restore;

    PROCEDURE (d: StdDocument) Externalize2*
        (VAR wr: Stores.Writer), EXTENSIBLE;
    BEGIN
        wr.WriteVersion(maxVersion);
        wr.WriteLong(d.w);  wr.WriteLong(d.h);
        wr.WriteLong(d.vl); wr.WriteLong(d.vt);
        wr.WriteLong(d.vr); wr.WriteLong(d.vb);
        wr.WriteLong(d.pw); wr.WriteLong(d.ph);
        wr.WriteLong(d.pl); wr.WriteLong(d.pt);
        wr.WriteLong(d.pr); wr.WriteLong(d.pb);
        wr.WriteBool(d.decorate);
        IF d.view # NIL THEN
            wr.WriteBool(TRUE);
            wr.WriteStore(d.view)
        ELSE
            wr.WriteBool(FALSE)
        END
    END Externalize2;

    PROCEDURE (d: StdDocument) Internalize2*
        (VAR rd: Stores.Reader), EXTENSIBLE;
        VAR ver, w, h, vl, vt, vr, vb,
            pw, ph, pl, pt, pr, pb: INTEGER;
            dec, hasView: BOOLEAN;
            handle: Stores.ReaderHandle;
            s: Stores.Store;
    BEGIN
        rd.ReadVersion(minVersion, maxVersion, ver);
        IF rd.cancelled THEN RETURN END;
        rd.ReadLong(w);  IF rd.eof THEN RETURN END;
        rd.ReadLong(h);  IF rd.eof THEN RETURN END;
        rd.ReadLong(vl); IF rd.eof THEN RETURN END;
        rd.ReadLong(vt); IF rd.eof THEN RETURN END;
        rd.ReadLong(vr); IF rd.eof THEN RETURN END;
        rd.ReadLong(vb); IF rd.eof THEN RETURN END;
        rd.ReadLong(pw); IF rd.eof THEN RETURN END;
        rd.ReadLong(ph); IF rd.eof THEN RETURN END;
        rd.ReadLong(pl); IF rd.eof THEN RETURN END;
        rd.ReadLong(pt); IF rd.eof THEN RETURN END;
        rd.ReadLong(pr); IF rd.eof THEN RETURN END;
        rd.ReadLong(pb); IF rd.eof THEN RETURN END;
        rd.ReadBool(dec); IF rd.eof THEN RETURN END;
        d.w := w; d.h := h;
        d.vl := vl; d.vt := vt; d.vr := vr; d.vb := vb;
        d.pw := pw; d.ph := ph;
        d.pl := pl; d.pt := pt; d.pr := pr; d.pb := pb;
        d.decorate := dec;
        rd.ReadBool(hasView);
        IF rd.eof THEN RETURN END;
        IF hasView THEN
            rd.ReadStore(handle);
            IF rd.cancelled THEN RETURN END;
            IF handle # 0 THEN
                s := Stores.NewStore(handle);
                IF (s # NIL) & (s IS Views.View) THEN
                    d.view := s(Views.View)
                END
            END
        END
    END Internalize2;


    (* -- StdContext -------------------------------------------------------- *)

    PROCEDURE (c: StdContext) ThisDoc* (): Document;
    BEGIN RETURN c.doc END ThisDoc;

    PROCEDURE (c: StdContext) ThisModel* (): Models.Model;
    BEGIN RETURN NIL END ThisModel;

    PROCEDURE (c: StdContext) Normalize* (): BOOLEAN;
    BEGIN RETURN TRUE END Normalize;

    PROCEDURE (c: StdContext) GetSize* (OUT w, h: INTEGER);
    BEGIN w := c.doc.w; h := c.doc.h END GetSize;


    (* -- StdDirectory ------------------------------------------------------ *)

    PROCEDURE (dir: StdDirectory) New*
        (view: Views.View; w, h: INTEGER): Document;
        VAR doc: StdDocument; ctx: StdContext;
    BEGIN
        NEW(doc); NEW(ctx);
        ctx.doc      := doc;
        doc.context  := ctx;
        doc.SetView(view, w, h);
        doc.vl := 0;     doc.vt := 0;
        doc.vr := w;     doc.vb := h;
        doc.pw := w + 2 * defB; doc.ph := h + 2 * defB;
        doc.pl := defB;  doc.pt := defB;
        doc.pr := defB;  doc.pb := defB;
        doc.decorate := FALSE;
        RETURN doc
    END New;


    (* -- Module-level ------------------------------------------------------ *)

    PROCEDURE SetDir* (d: Directory);
    BEGIN
        ASSERT(d # NIL, 20);
        dir := d;
        IF stdDir = NIL THEN stdDir := d END
    END SetDir;

    (* Validates the .odc tag + version; store deserialisation from a
       Files.File is not yet wired (needs Stores.Reader wrapping a
       Files.Reader), so s is always NIL on return. *)
    PROCEDURE ImportDocument* (f: Files.File; OUT s: Stores.Store);
        VAR r: Files.Reader;
            b: BYTE; t0, t1, t2, t3, tag, v0, v1, ver: INTEGER;
    BEGIN
        s := NIL;
        IF f = NIL THEN RETURN END;
        r := f.NewReader(NIL);
        IF r = NIL THEN RETURN END;
        r.ReadByte(b); t0 := b;
        r.ReadByte(b); t1 := b;
        r.ReadByte(b); t2 := b;
        r.ReadByte(b); t3 := b;
        IF r.eof THEN RETURN END;
        tag := t0 + t1 * 100H + t2 * 10000H + t3 * 1000000H;
        IF tag # docTag THEN RETURN END;
        r.ReadByte(b); v0 := b;
        r.ReadByte(b); v1 := b;
        IF r.eof THEN RETURN END;
        ver := v0 + v1 * 100H;
        IF ver > docVersion THEN RETURN END
    END ImportDocument;

    PROCEDURE ExportDocument* (s: Stores.Store; f: Files.File);
    BEGIN
    END ExportDocument;


BEGIN
    NEW(defaultDir);
    SetDir(defaultDir)
END Documents.
