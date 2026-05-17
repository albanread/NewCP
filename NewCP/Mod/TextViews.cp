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
    typed instance via `Reader.ReadStore` followed by
    `Stores.NewStore`, and skips past the controller / ruler /
   attributes children (which need their own typed records,
   slated for later slices).

   See newcp-odc/src/text_views.rs for the canonical wire-format
   specification.
*)

IMPORT Stores, Models, TextModels, TextRulers, TextSetters, Views, Containers, Ports, Fonts;

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

    (** Wire-format reader for `.odc`'s embedded TextViews
        StdView record.  Extends `Stores.StoreDesc` so
        `Stores.NewStore` can allocate it from the
        qualified-type name baked into the .odc.

        BB collapses this functionality into `Pane` (below) via
        the full Stores.Store → Views.View → Containers.View
        chain.  We still keep the persisted wire reader (`StdView`)
        and the live pane (`Pane`) separate, but they now share the
        `Stores.StoreDesc` root. *)
    StdViewDesc* = RECORD (Stores.StoreDesc)
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

        (** Live Doc converted from `model` by `StdModelToDoc` at
            the end of a successful Internalize.  NIL when model
            was absent or decoding failed. *)
        doc*:   TextModels.Doc;

        (** Loaded inline default ruler/attributes stores.  NIL
            when the on-disk child store is absent or its type
            doesn't match the expected pointer type. *)
        defRuler*: TextRulers.Ruler;
        defAttr*:  TextModels.Attributes;

        result*: INTEGER
    END;
    StdView* = POINTER TO StdViewDesc;

    (** Editor pane carrying the runtime state a framework-bound
        text view needs: the model it's displaying, the scroll
        origin (`org` is the top-of-view character offset; `dy`
        is the sub-line pixel offset), the default ruler and
        attributes to hand to fresh content, and the view-marks
        visibility flag.

        Named `Pane` (not `StdView`) to avoid colliding with the
        existing wire-reader `StdView` above.  BB's `StdView`
        unifies the two roles by riding the full Stores.Store →
        Views.View → Containers.View → TextViews.View chain; our
        port keeps them split until the persisted wire reader and
        live pane can merge into one hierarchy. Functionally `Pane` IS
        the BB editor pane prefix: same field meanings, same
        abstract-method overrides.

        BB's StdView also carries a cached setter / reader and a
        line cache (`trailer`, `bot`, `setter`, `setter0`,
        `cachedRd`).  Those are layout / display state — useless
        without rendering — and stay deferred.  This slice is a
        BB-faithful prefix: callers writing framework-style code
        (`v.SetOrigin`, `v.SetDefaults`, `v.DisplayMarks`) get
        the same field semantics they'd get from full BB. *)
    PaneDesc* = RECORD (ViewDesc)
        text-:      TextModels.Model;
        org-:       INTEGER;
        dy-:        INTEGER;
        defRuler-:  TextRulers.Ruler;
        defAttr-:   TextModels.Attributes;
        hideMarks-: BOOLEAN
    END;
    Pane* = POINTER TO PaneDesc;

    (** Concrete view-directory.  Builds fresh Pane instances on
        `New(text)`, inheriting the default-attribute set
        installed via `Set(defAttr)`. *)
    PaneDirectoryDesc* = RECORD (DirectoryDesc) END;
    PaneDirectory*     = POINTER TO PaneDirectoryDesc;

    ViewObserverDesc = RECORD (Models.ModelObserverDesc)
        pane: Pane
    END;
    ViewObserver = POINTER TO ViewObserverDesc;

VAR
    (** Container-side directory the framework hands to fresh
        StdView instances when they need a default controller. *)
    ctrlDir-: Containers.Directory;

    (** Active and default view directories.  `SetDir` overrides
        `dir`; `stdDir` is the framework-installed default and
        never gets replaced. *)
    dir-, stdDir-: Directory;

    (** Module-private storage of the PaneDirectory instance so
        the body can NEW it through its concrete type before
        publishing through the abstract `dir-` / `stdDir-` slots. *)
    std: PaneDirectory;

(* ─── StdModel → Doc bridge ────────────────────────────────────
   Converts a decoded wire-format StdModel into the concrete
   TextModels.Doc that Pane.Restore can drive through NewReader.
   Population goes through DocWriter.WriteChar because DocDesc.buf
   and DocDesc.len carry the `-` read-only export mark, making
   direct assignment illegal from this module.  WriteChar already
   caps at DocCapacity - 1 and maintains the 0X sentinel. *)

PROCEDURE StdModelToDoc* (m: TextModels.StdModel): TextModels.Doc;
    VAR d:  TextModels.Doc;
        wr: TextModels.Writer;
        i:  INTEGER;
BEGIN
    IF (m = NIL) OR (m.result # TextModels.OkComplete) THEN
        RETURN NIL
    END;
    NEW(d);
    wr := d.NewWriter(NIL);
    i := 0;
    WHILE (i < m.textLen) & (i < TextModels.DocCapacity - 1) DO
        wr.WriteChar(m.text[i]);
        INC(i)
    END;
    RETURN d
END StdModelToDoc;

(* ─── Pane factory from a decoded StdView ──────────────────────
   Builds a live Pane from a successfully-decoded StdView, binding
   the converted Doc as the text model and copying the scroll/
   display state (org, dy, hideMarks) from the wire record.
   Returns NIL when `sv` is absent, incomplete, or model-less. *)

PROCEDURE NewPane* (sv: StdView): Pane;
    VAR p: Pane;
BEGIN
    IF (sv = NIL) OR (sv.result # OkComplete) OR (sv.model = NIL) THEN
        RETURN NIL
    END;
    NEW(p);
    p.text      := StdModelToDoc(sv.model);
    p.org       := sv.org;
    p.dy        := sv.dy;
    p.hideMarks := sv.hideMarks;
    p.defRuler  := sv.defRuler;
    p.defAttr   := sv.defAttr;
    RETURN p
END NewPane;

PROCEDURE (v: StdViewDesc) Domain* (): Stores.Domain;
BEGIN
    RETURN NIL
END Domain;

PROCEDURE (v: StdViewDesc) Internalize* (VAR rd: Stores.Reader);
    VAR i: INTEGER;
        b: BYTE;
        modelStoreHandle: INTEGER;
        modelStore: Stores.Store;
        bool: BOOLEAN;
        rulerHandle, attrHandle: INTEGER;
        rulerStore, attrStore: Stores.Store;
BEGIN
    v.model    := NIL;
    v.doc      := NIL;
    v.defRuler := NIL;
    v.defAttr  := NIL;
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
    rd.ReadStore(modelStoreHandle);
    IF modelStoreHandle = 0 THEN RETURN END;

    v.result := OkModelLoadFailed;
    modelStore := Stores.NewStore(modelStoreHandle);
    IF modelStore = NIL THEN RETURN END;
    (* Type-guard down to a concrete StdModel — anything else is
       a wire-format mismatch we don't handle yet. *)
    v.model := modelStore(TextModels.StdModel);

    (* Inline Controller store — skip without materializing. *)
    v.result := OkControllerSkipFail;
    rd.SkipStore();
    IF rd.cancelled THEN RETURN END;

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

    (* Default Ruler — load if present. *)
    v.result := OkRulerSkipFail;
    rd.ReadStore(rulerHandle);
    IF rd.cancelled THEN RETURN END;
    IF rulerHandle # 0 THEN
        rulerStore := Stores.NewStore(rulerHandle);
        IF (rulerStore # NIL) & (rulerStore IS TextRulers.Ruler) THEN
            v.defRuler := rulerStore(TextRulers.Ruler)
        END
    END;

    (* Default Attributes — load if present. *)
    v.result := OkAttrsSkipFail;
    rd.ReadStore(attrHandle);
    IF rd.cancelled THEN RETURN END;
    IF attrHandle # 0 THEN
        attrStore := Stores.NewStore(attrHandle);
        IF (attrStore # NIL) & (attrStore IS TextModels.Attributes) THEN
            v.defAttr := attrStore(TextModels.Attributes)
        END
    END;

    (* org / dy trailer — 4-byte i32 each in the BB wire format. *)
    v.result := OkTrailerTrunc;
    rd.ReadLong(v.org);
    IF rd.eof THEN RETURN END;
    rd.ReadLong(v.dy);
    IF rd.eof THEN RETURN END;

    v.doc    := StdModelToDoc(v.model);
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

(* ─── Pane concrete bodies ─────────────────────────────────────
   Field-update implementations for the abstract View methods,
   plus the two Containers.View abstracts (`AcceptableModel`,
   `Restore`) that any concrete View must supply.

   No layout / rendering yet — `GetThisLocation`, `GetRange`,
   `ThisPos`, `ShowRangeIn`, `ShowRange`, `Restore` are all
    safe no-ops so callers don't trap.  The state-update methods
    (`DisplayMarks`, `SetSetter`, `SetOrigin`, `SetDefaults`)
   and their pollers are BB-faithful and ready for use.
*)

(** Container-level: a Pane accepts any TextModels.Model
    (typically a TextModels.StdModel).  TYPE-guard checks the
    runtime type before binding. *)
PROCEDURE (v: Pane) AcceptableModel* (m: Containers.Model): BOOLEAN;
BEGIN
    RETURN (m # NIL) & (m IS TextModels.Model)
END AcceptableModel;

(** Views-level: paint the rectangle.  Two phases:

      Phase 1 — scaffold: white background fill over the dirty
        rect, plus a thin black indicator bar along the top
        edge when the pane has a bound model.

      Phase 2 — text content: walk the model via its abstract
        Reader and emit one DrawString call per "line" (here
        meaning "char run up to 0X / end-of-text").  Single-line
        rendering only — no wrapping, no per-char attributes,
        no actual layout.  But it's a real text-bearing paint
        pipeline:

          v.text.NewReader → ReadChar loop → DrawString

        The line cache + setter integration that BB uses for
        wrapped, multi-line, attribute-aware layout is the next
        slice; everything else (bound-rider dispatch, coord
        translation, font passing) is already real.

    Drawing port: every call goes through `f.DrawRect` /
    `f.DrawString`, which Ports translates from user units to
    device dots before forwarding to the bound Rider.  A
    recording rider can capture the call tuples to verify what
    landed. *)
PROCEDURE (v: Pane) Restore* (f: Views.Frame; l, t, r, b: INTEGER);
    CONST
        barH     = 50;      (* indicator bar height in user units *)
        lineH    = 120;     (* approximate line height in user units *)
        maxLen   = 256;     (* max chars to render per line *)
        charW    = 8;       (* rough char width in DIPs per printable char *)
        selColor = 0FFD6ADH; (* light-blue selection: R=0xAD, G=0xD6, B=0xFF *)
    VAR rd: TextModels.Reader;
        line:      ARRAY 256 OF CHAR;
        lineAttrs: ARRAY 256 OF TextModels.Attributes;  (* per-printable-char attr *)
        runBuf:    ARRAY 256 OF CHAR;
        i, y, j: INTEGER;
        lineChars: INTEGER;  (* total model chars on line, including non-printable *)
        defFont, font: Fonts.Font;
        runFont, charFont: Fonts.Font;
        runStart, runEnd, runLen: INTEGER;
        carPos: INTEGER;
        selBeg, selEnd: INTEGER;
        linePos: INTEGER;
        lastLineStart: INTEGER;  (* model pos of first char of last rendered line *)
        caretX, caretYTop, caretYBot: INTEGER;
        caretDrawn: BOOLEAN;
        hlBeg, hlEnd: INTEGER;  (* selection overlap on current line [chars] *)
BEGIN
    (* Phase 1: scaffold. *)
    f.DrawRect(l, t, r, b, Ports.fill, Ports.white);
    IF (v.text # NIL) & (t < barH) THEN
        f.DrawRect(l, t, r, MIN(b, barH), Ports.fill, Ports.black)
    END;

    (* Get caret and selection from controller. *)
    carPos := -1; selBeg := -1; selEnd := -1;
    IF v.controller # NIL THEN
        carPos := v.controller.CaretPos();
        v.controller.GetSelection(selBeg, selEnd)
    END;
    IF selBeg > selEnd THEN  (* normalise *)
        i := selBeg; selBeg := selEnd; selEnd := i
    END;

    (* Phase 2: text content — multi-line from v.org. *)
    IF (v.text # NIL) & (b > barH) THEN
        rd := v.text.NewReader(NIL);
        IF rd # NIL THEN
            rd.SetPos(v.org);
            (* Resolve the pane's default font — used for chars whose attr is NIL. *)
            defFont := NIL;
            IF v.defAttr # NIL THEN defFont := v.defAttr.font END;
            IF (defFont = NIL) & (Fonts.dir # NIL) THEN
                defFont := Fonts.dir.Default()
            END;
            font := defFont;

            y := barH + lineH;
            linePos := v.org;
            lastLineStart := v.org;
            caretDrawn := FALSE;
            rd.ReadChar();   (* prime the reader *)
            WHILE ~rd.eot & (y <= b) DO
                (* Collect printable chars and their resolved fonts.
                   `i` = printable chars; `lineChars` = total model chars read. *)
                lastLineStart := linePos;
                i := 0; lineChars := 0;
                WHILE ~rd.eot
                    & (rd.char # TextModels.line)
                    & (rd.char # TextModels.para)
                    & (i < maxLen - 1) DO
                    IF rd.char >= ' ' THEN
                        line[i]      := rd.char;
                        lineAttrs[i] := rd.attr;  (* may be NIL → use defFont *)
                        INC(i)
                    END;
                    INC(lineChars);
                    rd.ReadChar()
                END;
                line[i] := 0X;

                (* Draw selection highlight if this line overlaps [selBeg, selEnd). *)
                IF (selBeg >= 0) & (selBeg < selEnd)
                 & (linePos + lineChars > selBeg) & (linePos < selEnd) THEN
                    hlBeg := selBeg - linePos; IF hlBeg < 0 THEN hlBeg := 0 END;
                    hlEnd := selEnd - linePos; IF hlEnd > lineChars THEN hlEnd := lineChars END;
                    IF hlEnd > i THEN hlEnd := i END;  (* cap to printable range *)
                    f.DrawRect(l + hlBeg * charW, y - lineH + 10,
                               l + hlEnd * charW, y + 10,
                               Ports.fill, selColor)
                END;

                (* Render the line in consecutive same-font runs. *)
                IF i > 0 THEN
                    runStart := 0;
                    WHILE runStart < i DO
                        (* Determine this run's font. *)
                        runFont := defFont;
                        IF (lineAttrs[runStart] # NIL) & (lineAttrs[runStart].font # NIL) THEN
                            runFont := lineAttrs[runStart].font
                        END;
                        (* Extend run while font is the same. *)
                        runEnd := runStart + 1;
                        WHILE runEnd < i DO
                            charFont := defFont;
                            IF (lineAttrs[runEnd] # NIL) & (lineAttrs[runEnd].font # NIL) THEN
                                charFont := lineAttrs[runEnd].font
                            END;
                            IF charFont # runFont THEN EXIT END;
                            INC(runEnd)
                        END;
                        (* Build run buffer and draw. *)
                        runLen := runEnd - runStart;
                        j := 0;
                        WHILE j < runLen DO
                            runBuf[j] := line[runStart + j]; INC(j)
                        END;
                        runBuf[runLen] := 0X;
                        f.DrawString(l + runStart * charW, y, Ports.black, runBuf, runFont);
                        runStart := runEnd
                    END
                END;

                (* Draw caret if it falls on this line (uses model position). *)
                IF (carPos >= linePos) & (carPos <= linePos + lineChars) THEN
                    caretX := l + (carPos - linePos) * charW;
                    caretYTop := y - lineH + 10;
                    caretYBot := y + 10;
                    f.DrawLine(caretX, caretYTop, caretX, caretYBot, 2, Ports.black);
                    caretDrawn := TRUE
                END;

                linePos := linePos + lineChars + 1;  (* +1 for line separator *)
                INC(y, lineH);
                IF ~rd.eot THEN rd.ReadChar() END
            END;
            (* Draw caret at end-of-text if not placed yet.
               Use lastLineStart to get the correct X position. *)
            IF ~caretDrawn & (carPos >= 0) THEN
                caretX := l + (carPos - lastLineStart) * charW;
                IF caretX < l THEN caretX := l END;
                caretYTop := y - lineH + 10;
                caretYBot := y + 10;
                f.DrawLine(caretX, caretYTop, caretX, caretYBot, 2, Ports.black)
            END
        END
    END
END Restore;

PROCEDURE (v: Pane)DisplayMarks* (hide: BOOLEAN);
BEGIN
    v.hideMarks := hide
END DisplayMarks;

PROCEDURE (v: Pane)HidesMarks* (): BOOLEAN;
BEGIN
    RETURN v.hideMarks
END HidesMarks;

PROCEDURE (v: Pane)SetSetter* (setter: TextSetters.Setter);
BEGIN
    (* In the full BB StdView the setter+setter0 pair caches the
       layout engine across frames and gets invalidated when the
       model changes.  Storage is deferred to the StdView-with-
       cache slice; for now the call is a no-op so callers that
       hand-install a setter don't trap. *)
END SetSetter;

PROCEDURE (v: Pane)ThisSetter* (): TextSetters.Setter;
BEGIN
    RETURN NIL
END ThisSetter;

PROCEDURE (v: Pane)SetOrigin* (org, dy: INTEGER);
BEGIN
    v.org := org;
    v.dy := dy
END SetOrigin;

PROCEDURE (v: Pane)PollOrigin* (OUT org, dy: INTEGER);
BEGIN
    org := v.org;
    dy := v.dy
END PollOrigin;

PROCEDURE (v: Pane)SetDefaults* (r: TextRulers.Ruler; a: TextModels.Attributes);
BEGIN
    ASSERT(r # NIL, 20);
    ASSERT(a # NIL, 21);
    v.defRuler := r;
    v.defAttr  := a
END SetDefaults;

PROCEDURE (v: Pane)PollDefaults* (OUT r: TextRulers.Ruler; OUT a: TextModels.Attributes);
BEGIN
    r := v.defRuler;
    a := v.defAttr
END PollDefaults;

PROCEDURE (v: Pane)GetThisLocation* (f: Views.Frame; pos: INTEGER; OUT loc: Location);
BEGIN
    (* Geometry returned by a full StdView would walk the line
       cache to find the rectangle around `pos`.  Without a line
       cache we return a zero Location — callers that consult
       `loc.x` / `loc.y` see (0, 0); type-guard-style consumers
       checking `loc.view = NIL` correctly conclude "no embedded
       child at this position". *)
    loc.start := pos; loc.pos := pos;
    loc.x := 0; loc.y := 0;
    loc.asc := 0; loc.dsc := 0;
    loc.view := NIL;
    loc.l := 0; loc.t := 0; loc.r := 0; loc.b := 0
END GetThisLocation;

PROCEDURE (v: Pane)GetRange* (f: Views.Frame; OUT beg, end: INTEGER);
BEGIN
    (* Visible range = [org, org] — empty until rendering exists. *)
    beg := v.org;
    end := v.org
END GetRange;

PROCEDURE (v: Pane)ThisPos* (f: Views.Frame; x, y: INTEGER): INTEGER;
BEGIN
    (* No hit-testing without a line cache — every screen click
       resolves to the scroll origin. *)
    RETURN v.org
END ThisPos;

PROCEDURE (v: Pane)ShowRangeIn* (f: Views.Frame; beg, end: INTEGER);
BEGIN
    (* No-op until rendering lands. *)
END ShowRangeIn;

PROCEDURE (v: Pane)ShowRange* (beg, end: INTEGER; focusOnly: BOOLEAN);
BEGIN
    (* No-op until broadcast routing through Models.Broadcast
       fires from a meaningful set of frames. *)
END ShowRange;

PROCEDURE (v: Pane) HandleModelMsg* (VAR msg: Models.Message);
BEGIN
    WITH msg: TextModels.UpdateMsg DO
        Views.Update(v, Views.keepFrames)
    ELSE
    END
END HandleModelMsg;

PROCEDURE (v: Pane) HandleViewMsg* (f: Views.Frame; VAR msg: Views.Message);
BEGIN
    WITH msg: Views.UpdateCachesMsg DO
        v.Restore(f, f.l, f.t, f.r, f.b)
    ELSE
    END
END HandleViewMsg;

PROCEDURE (obs: ViewObserver) Notify* (m: Models.Model; VAR msg: Models.Message);
BEGIN
    obs.pane.HandleModelMsg(msg)
END Notify;

PROCEDURE (v: Pane) InitModel2* (m: Containers.Model);
    VAR obs: ViewObserver;
BEGIN
    NEW(obs);
    obs.pane := v;
    Models.InstallObserver(m, obs)
END InitModel2;

PROCEDURE (v: Pane) Externalize2* (VAR wr: Stores.Writer);
BEGIN
    wr.WriteBool(v.text # NIL);
    IF v.text # NIL THEN wr.WriteStore(v.text) END;
    wr.WriteBool(v.hideMarks);
    wr.WriteLong(v.org);
    wr.WriteLong(v.dy)
END Externalize2;

PROCEDURE (v: Pane) Internalize2* (VAR rd: Stores.Reader);
    VAR hasModel: BOOLEAN; handle: INTEGER; s: Stores.Store;
BEGIN
    rd.ReadBool(hasModel);
    IF rd.eof THEN RETURN END;
    IF hasModel THEN
        rd.ReadStore(handle);
        IF rd.cancelled THEN RETURN END;
        IF handle # 0 THEN
            s := Stores.NewStore(handle);
            IF (s # NIL) & (s IS TextModels.Doc) THEN
                v.text := s(TextModels.Doc);
                v.InitModel(v.text)
            END
        END
    END;
    rd.ReadBool(v.hideMarks);
    IF rd.eof THEN RETURN END;
    rd.ReadLong(v.org);
    IF rd.eof THEN RETURN END;
    rd.ReadLong(v.dy)
END Internalize2;

(* ─── Abstract Directory surface ───────────────────────────────
   `New(text)` is the BB-faithful "build me a fresh view for this
   model" factory — supplied by the concrete StdDirectory below.
   `Set` is concrete-EXTENSIBLE here: it just stores the
   default-attributes blob the framework will hand to fresh views.
*)

PROCEDURE (d: Directory) New* (text: TextModels.Model): View, NEW, ABSTRACT;

PROCEDURE (d: Directory) Set* (defAttr: TextModels.Attributes), NEW, EXTENSIBLE;
BEGIN
    ASSERT(defAttr # NIL, 20);
    d.defAttr := defAttr
END Set;

(* ─── StdDirectory concrete factory ────────────────────────────
   `New(text)` allocates a fresh StdView bound to `text` with
   neutral display state (origin (0, 0), marks visible, no
   defaults set yet).  BB calls `Set(defAttr)` on the directory
   at boot to plant the initial attribute set; if the caller
   hasn't done so, `defAttr` stays NIL and the StdView's
   `defAttr-` field is NIL too — a downstream call to
   `v.PollDefaults` will return NIL, which is the BB contract.
*)

PROCEDURE (d: PaneDirectoryDesc) New* (text: TextModels.Model): View;
    VAR v: Pane;
BEGIN
    NEW(v);
    IF text # NIL THEN
        (* Bind through Containers' InitModel so the inherited
           `model-` field gets set and ThisModel() returns the
           bound text.  InitModel asserts via AcceptableModel
           (overridden above) and runs the EMPTY InitModel2 hook. *)
        v.InitModel(text)
    END;
    v.text     := text;
    v.org      := 0;
    v.dy       := 0;
    v.defRuler := NIL;
    v.defAttr  := d.defAttr;
    v.hideMarks := FALSE;
    (* Install a default controller so the pane can accept keyboard
       and mouse input immediately.  ctrlDir is set by TextControllers
       in its module body via SetCtrlDir(dir); it is NIL only before
       TextControllers has been imported. *)
    IF ctrlDir # NIL THEN
        v.SetController(ctrlDir.NewController({}))
    END;
    RETURN v
END New;

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

BEGIN
    (* Install StdDirectory as both the framework default and the
       currently-active directory.  BB does this from an explicit
       boot script; we install at module-init so importing
       TextViews gives a working factory immediately. *)
    NEW(std);
    stdDir := std;
    dir := std
END TextViews.
