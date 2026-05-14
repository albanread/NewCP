MODULE TextViews;
(*
   First slice of the BlackBox `TextViews` port.

   `StdView` is the on-disk representation of a visible text
   editor pane. Its body interleaves version bytes and inline
   child stores in a fixed sequence:

     1 byte    Stores.Store version
     1 byte    Views.View version
     1 byte    Containers.View version
     store     inline Model  (a TextModels.StdModel)
     store     inline Controller  (may be a nil store)
     1 byte    TextViews.View Internalize2 version
     1 byte    TextViews.StdView Internalize2 version
     1 byte    hideMarks BOOLEAN
     store     inline default Ruler  (may be nil)
     store     inline default Attributes  (may be nil)
     4 bytes   org  INTEGER  (top-of-view character offset)
     4 bytes   dy   INTEGER  (sub-line scroll offset, in pixels)

   This slice materializes the embedded `TextModels.StdModel`
   typed instance via `Reader.ReadInlineStore` followed by
   `HostStores.NewStore`, and skips past the controller / ruler /
   attributes children (which need their own typed records,
   slated for later slices).

   See newcp-odc/src/text_views.rs for the canonical wire-format
   specification.
*)

IMPORT HostStores, TextModels, TextRulers, TextSetters, Views, Containers;

CONST
    OkComplete*           = 0;
    OkSuperVersionsTrunc* = 1;
    OkModelMissing*       = 2;
    OkModelLoadFailed*    = 3;
    OkControllerSkipFail* = 4;
    OkInternalize2Trunc*  = 5;
    OkRulerSkipFail*      = 6;
    OkAttrsSkipFail*      = 7;
    OkTrailerTrunc*       = 8;

    minVersion = 0; maxVersion = 0;

TYPE
    (** Abstract text-view surface. BB-faithful descendant of
        `Containers.View`; concrete editor panes (StdView once it
        ports fully) extend it.  TextControllers's `Controller`
        carries a `view-: View` field so the controller side can
        type-route messages back to the visible pane without
        knowing the concrete pane type. *)
    ViewDesc* = ABSTRACT RECORD (Containers.ViewDesc) END;
    View*     = POINTER TO ViewDesc;

    (** Abstract text-view directory.  Carries the default
        attributes a freshly-installed view should adopt; the
        concrete factory method (`New`) is declared abstract here
        and supplied by `StdDirectory` (later slice). *)
    DirectoryDesc* = ABSTRACT RECORD
        defAttr-: TextModels.Attributes
    END;
    Directory*     = POINTER TO DirectoryDesc;

    (** Geometric location of a character position within a view.
        `start`/`pos` are character offsets; `x`/`y` are pixel
        coordinates inside the frame; `asc`/`dsc` are the line
        metrics at the location; `view, l, t, r, b` describe an
        embedded child view's rectangle if the position addresses
        one.  Used by `GetThisLocation` and `GetRect`. *)
    Location* = RECORD
        start*, pos*: INTEGER;
        x*, y*:       INTEGER;
        asc*, dsc*:   INTEGER;
        view*:        Views.View;
        l*, t*, r*, b*: INTEGER
    END;

    (** Position-change broadcast.  Sent by `ShowRange` so all
        frames displaying the affected text can re-scroll. *)
    PositionMsg* = RECORD (Views.Message)
        focusOnly*: BOOLEAN;
        beg*, end*: INTEGER
    END;

    (** Per-page change broadcast — used by Printers / paginated
        display to discover which page a position lands on. *)
    PageMsg* = RECORD (Views.Message)
        current*: INTEGER
    END;

    StdViewDesc* = RECORD (HostStores.StoreDesc)
        (** Stage-1 super-class version chain (Stores.Store +
            Views.View + Containers.View). *)
        v1*: ARRAY 3 OF BYTE;
        (** Stage-2 (`Internalize2`) super-class version bytes
            (TextViews.View + TextViews.StdView). *)
        v2*: ARRAY 2 OF BYTE;
        hideMarks*: BOOLEAN;
        (** Top-of-view character offset; sub-line dy in pixels. *)
        org*: INTEGER;
        dy*:  INTEGER;
        (** Materialized child model. NIL when the on-disk view
            doesn't carry a model (rare) or when materialization
            failed; in the latter case `result` reflects the
            specific failure. *)
        model*: TextModels.StdModel;

        result*: INTEGER
    END;
    StdView* = POINTER TO StdViewDesc;

VAR
    (** Container-side directory the framework hands to fresh
        StdView instances when they need a default controller. *)
    ctrlDir-: Containers.Directory;

    (** Active and default view directories.  `SetDir` overrides
        `dir`; `stdDir` is the framework-installed default and
        never gets replaced. *)
    dir-, stdDir-: Directory;

PROCEDURE (v: StdViewDesc) Internalize* (rd: HostStores.Reader);
    VAR i: INTEGER;
        b: BYTE;
        modelStoreHandle: INTEGER;
        modelStore: HostStores.Store;
        bool: BOOLEAN;
BEGIN
    v.model := NIL;
    v.org := 0;
    v.dy := 0;
    v.hideMarks := FALSE;
    v.result := OkSuperVersionsTrunc;

    (* Stage-1 super-class version chain. *)
    i := 0;
    WHILE i < 3 DO
        rd.ReadByte(b);
        IF rd.eof THEN RETURN END;
        v.v1[i] := b;
        INC(i)
    END;

    (* Inline Model store. *)
    v.result := OkModelMissing;
    modelStoreHandle := rd.ReadInlineStore();
    IF modelStoreHandle = 0 THEN RETURN END;

    v.result := OkModelLoadFailed;
    modelStore := HostStores.NewStore(modelStoreHandle);
    IF modelStore = NIL THEN RETURN END;
    (* Type-guard down to a concrete StdModel — anything else is
       a wire-format mismatch we don't handle yet. *)
    v.model := modelStore(TextModels.StdModel);

    (* Inline Controller store — skip without materializing. *)
    v.result := OkControllerSkipFail;
    IF ~rd.SkipInlineStore() THEN RETURN END;

    (* Stage-2 version bytes. *)
    v.result := OkInternalize2Trunc;
    i := 0;
    WHILE i < 2 DO
        rd.ReadByte(b);
        IF rd.eof THEN RETURN END;
        v.v2[i] := b;
        INC(i)
    END;

    (* hideMarks BOOLEAN (1 byte on the wire). *)
    rd.ReadBool(bool);
    IF rd.eof THEN RETURN END;
    v.hideMarks := bool;

    (* Default Ruler — skip. *)
    v.result := OkRulerSkipFail;
    IF ~rd.SkipInlineStore() THEN RETURN END;

    (* Default Attributes — skip. *)
    v.result := OkAttrsSkipFail;
    IF ~rd.SkipInlineStore() THEN RETURN END;

    (* org / dy trailer. *)
    v.result := OkTrailerTrunc;
    rd.ReadInt(v.org);
    IF rd.eof THEN RETURN END;
    rd.ReadInt(v.dy);
    IF rd.eof THEN RETURN END;

    v.result := OkComplete
END Internalize;

(* ─── Abstract View surface ────────────────────────────────────
   Method declarations TextControllers reaches through to ask the
   concrete pane about display state and to drive scrolling.  All
   bodies are deferred — concrete StdView slice will supply them.
*)

PROCEDURE (v: View) DisplayMarks* (hide: BOOLEAN), NEW, ABSTRACT;
PROCEDURE (v: View) HidesMarks*   (): BOOLEAN,      NEW, ABSTRACT;

PROCEDURE (v: View) SetSetter*  (setter: TextSetters.Setter), NEW, ABSTRACT;
PROCEDURE (v: View) ThisSetter* (): TextSetters.Setter,        NEW, ABSTRACT;

PROCEDURE (v: View) SetOrigin*  (org, dy: INTEGER),               NEW, ABSTRACT;
PROCEDURE (v: View) PollOrigin* (OUT org, dy: INTEGER),           NEW, ABSTRACT;

PROCEDURE (v: View) SetDefaults*  (r: TextRulers.Ruler; a: TextModels.Attributes),
    NEW, ABSTRACT;
PROCEDURE (v: View) PollDefaults* (OUT r: TextRulers.Ruler; OUT a: TextModels.Attributes),
    NEW, ABSTRACT;

PROCEDURE (v: View) GetThisLocation* (f: Views.Frame; pos: INTEGER; OUT loc: Location),
    NEW, ABSTRACT;
PROCEDURE (v: View) GetRange*        (f: Views.Frame; OUT beg, end: INTEGER),
    NEW, ABSTRACT;
PROCEDURE (v: View) ThisPos*         (f: Views.Frame; x, y: INTEGER): INTEGER,
    NEW, ABSTRACT;
PROCEDURE (v: View) ShowRangeIn*     (f: Views.Frame; beg, end: INTEGER),
    NEW, ABSTRACT;
PROCEDURE (v: View) ShowRange*       (beg, end: INTEGER; focusOnly: BOOLEAN),
    NEW, ABSTRACT;

(* ─── Abstract Directory surface ───────────────────────────────
   `New(text)` is the BB-faithful "build me a fresh view for this
   model" factory — supplied by the concrete StdDirectory once it
   lands.  `Set` is concrete-EXTENSIBLE here: it just stores the
   default-attributes blob the framework will hand to fresh views.
*)

PROCEDURE (d: Directory) New* (text: TextModels.Model): View, NEW, ABSTRACT;

PROCEDURE (d: Directory) Set* (defAttr: TextModels.Attributes), NEW, EXTENSIBLE;
BEGIN
    ASSERT(defAttr # NIL, 20);
    d.defAttr := defAttr
END Set;

PROCEDURE SetCtrlDir* (d: Containers.Directory);
BEGIN
    ASSERT(d # NIL, 20);
    ctrlDir := d
END SetCtrlDir;

PROCEDURE SetDir* (d: Directory);
BEGIN
    ASSERT(d # NIL, 20);
    dir := d
END SetDir;

END TextViews.
