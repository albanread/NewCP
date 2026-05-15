MODULE Documents;
(*
   First slice of the BlackBox `Documents` port.

   BB's `Documents` (2312 lines) is the framework's
   "store + view + page-state + window-binding" composite — what
   you open from disk and bind to a Window.  The welcome-page
   chain goes:
     Init → Converters.Register("Documents.ImportDocument", ...)
          → StdCmds.OpenToolDialog → Converters.Import
          → Documents.ImportDocument (reads .odc, returns Store)
          → Windows.dir.New(doc) → Views.Restore (welcome page on screen).

   This slice ships the SURFACE — every type, constant, and
   procedure heading that downstream modules (Windows, StdApi,
   StdCmds, Converters) reference at the type level.  Bodies are
   stubs:
     - `ImportDocument` / `ExportDocument` return / accept NIL.
     - `SetDir` stores the supplied Directory.
     - Document method stubs return safe defaults.
   That keeps the compile chain alive — `Converters.Register
   ("Documents.ImportDocument", ...)` doesn't yet do anything
   useful because Meta.LookupPath returns undef in this slice,
   but the SURFACE is present.

   Deferred (in priority order):
     - Real ImportDocument body — needs the wire-format
       `Stores.Reader` walk that decodes the .odc envelope.
     - StdDocument concrete implementation — the workhorse
       behind New / SetView / ThisView etc.  Currently abstract.
     - Print / PrinterContext / Pager / MakeVisible — page-print
       machinery, off the welcome-page critical path.
     - SetRectOp / SetPageOp / ReplaceViewOp — undoable
       operations; the welcome page is read-only so they don't
       fire.

   When Windows / StdApi / StdCmds port and start exercising
   ThisView, GetNewFrame, the bodies here grow real.  The
   surface is BB-faithful so no caller adjustments will be
   needed.
*)

    IMPORT
        Kernel, Files, Ports, Dates, Printers,
        Stores, Models, Views, Properties,
        Dialog, Printing, Containers;

    CONST
        (** Document.SetPage / PollPage decorate *)
        plain*    = FALSE;
        decorate* = TRUE;

        (** Controller.opts — these are bit numbers BB stamps
            into the `opts` set on its document controller; we
            export them so Windows / StdCmds can union them in
            without adding magic numbers. *)
        pageWidth*  = 16;
        pageHeight* = 17;
        winWidth*   = 18;
        winHeight*  = 19;

        (** Public for downstream sizing-policy callers. *)
        defB*       = 8 * Ports.point;     (** Default document border. *)
        scrollUnit* = 16 * Ports.point;

        (** Wire-format tags consumed by ImportDocument /
            ExportDocument. *)
        docTag*     = 6F4F4443H;
        docVersion* = 0;


    TYPE
        (** Abstract document — wraps a View in a Store that the
            framework can serialise / page / window-bind.  BB extends
            Containers.View so Documents inherit the model-binding +
            event-dispatch surface. *)
        DocumentDesc* = ABSTRACT RECORD (Containers.ViewDesc) END;
        Document*     = POINTER TO DocumentDesc;

        (** Abstract embedding context.  Concrete instances live
            inside the StdDocument wrapper and tell the embedded
            view which Document it lives inside. *)
        ContextDesc* = ABSTRACT RECORD (Models.ContextDesc) END;
        Context*     = POINTER TO ContextDesc;

        (** Abstract directory — Document factory. *)
        DirectoryDesc* = ABSTRACT RECORD END;
        Directory*     = POINTER TO DirectoryDesc;


    VAR
        (** Current Document factory. *)
        dir-:    Directory;

        (** Fallback default factory.  BB keeps this as the
            original `dir` so user-installed directories can be
            unwound. *)
        stdDir-: Directory;


    (* -- Document methods ------------------------------------------------- *)

    (** Read the per-Document state out of `rd`.  Body is BB-
        faithful: read the version byte; cancel on read failure. *)
    PROCEDURE (d: Document) Internalize2- (VAR rd: Stores.Reader), EXTENSIBLE;
    BEGIN
        (* deferred: real body calls rd.ReadVersion (not yet on
           our Stores.Reader).  Surface-only until the wire-
           format reader API lands. *)
    END Internalize2;

    (** Write the per-Document state into `wr`.  Mirror of above. *)
    PROCEDURE (d: Document) Externalize2- (VAR wr: Stores.Writer), EXTENSIBLE;
    BEGIN
        (* deferred *)
    END Externalize2;

    (** Allocate a fresh root frame for this Document.  Used by
        Windows when constructing the window's frame tree.  BB
        uses `NEW(Views.RootFrame)`; our Views slice doesn't
        export RootFrame yet (it's currently internal to the
        StdFrame ladder), so the default-empty body inherited
        from Views.ViewDesc.GetNewFrame is fine for now —
        Windows allocates the frame itself when it needs one. *)
    PROCEDURE (d: Document) GetNewFrame* (VAR frame: Views.Frame);
    BEGIN
        frame := NIL
    END GetNewFrame;

    (** Background color of the document's page area. *)
    PROCEDURE (d: Document) GetBackground* (VAR color: Ports.Color);
    BEGIN
        color := Ports.background
    END GetBackground;

    (** Construct a Document-of-the-same-type wrapping `v`.
        Concrete implementation lives in StdDocument. *)
    PROCEDURE (d: Document) DocCopyOf* (v: Views.View): Document, NEW, ABSTRACT;

    (** Set / get the embedded view. *)
    PROCEDURE (d: Document) SetView* (view: Views.View; w, h: INTEGER), NEW, ABSTRACT;
    PROCEDURE (d: Document) ThisView* (): Views.View,                NEW, ABSTRACT;
    PROCEDURE (d: Document) OriginalView* (): Views.View,            NEW, ABSTRACT;

    (** Visible rectangle of the embedded view. *)
    PROCEDURE (d: Document) SetRect*  (l, t, r, b: INTEGER),         NEW, ABSTRACT;
    PROCEDURE (d: Document) PollRect* (VAR l, t, r, b: INTEGER),     NEW, ABSTRACT;

    (** Page geometry (paper size + margins + decoration flag). *)
    PROCEDURE (d: Document) SetPage* (w, h, l, t, r, b: INTEGER; decorate: BOOLEAN), NEW, ABSTRACT;
    PROCEDURE (d: Document) PollPage* (VAR w, h, l, t, r, b: INTEGER; VAR decorate: BOOLEAN), NEW, ABSTRACT;


    (* -- Context methods -------------------------------------------------- *)

    (** Resolve the Document this context lives inside. *)
    PROCEDURE (c: Context) ThisDoc* (): Document, NEW, ABSTRACT;


    (* -- Directory methods ------------------------------------------------ *)

    (** Construct a new Document wrapping `view`, sized to (w, h). *)
    PROCEDURE (d: Directory) New* (view: Views.View; w, h: INTEGER): Document, NEW, ABSTRACT;


    (* -- Top-level entry points ------------------------------------------- *)

    (** Register a Directory implementation.  Called by the
        host-side document factory at startup. *)
    PROCEDURE SetDir* (d: Directory);
    BEGIN
        dir := d;
        IF stdDir = NIL THEN
            stdDir := d
        END
    END SetDir;

    (** BB-faithful `Documents.ImportDocument` — the procedure
        Converters.Register binds to the ".odc" extension.  Real
        body reads the docTag/docVersion header through Stores
        then deserialises the Document, but our Stores Reader
        doesn't yet expose `ReadVersion` / `ReadStore`, so this
        stub returns `s := NIL`.  Once those land, the body is a
        ~30-line direct port. *)
    PROCEDURE ImportDocument* (f: Files.File; OUT s: Stores.Store);
    BEGIN
        s := NIL
    END ImportDocument;

    (** Symmetric ExportDocument.  Deferred for the same reason. *)
    PROCEDURE ExportDocument* (s: Stores.Store; f: Files.File);
    BEGIN
        (* deferred *)
    END ExportDocument;


BEGIN
    dir    := NIL;
    stdDir := NIL
END Documents.
