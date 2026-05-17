MODULE TextRulers;
(*
   First slice of the BlackBox `TextRulers` port.

   `TextRulers` defines the paragraph-style abstraction every
   text view uses to describe a paragraph's geometry: left /
   right / first margins, leading / ascent / descent, the tab
   stops, plus a SET of "options" for justification and
   page-break behaviour.  Visually a Ruler is the strip at
   the top of a BB editor showing margin slides and tab
   markers — but the data model lives here.

   The full BlackBox module (~2970 lines) is mostly two
   things on top of the type tree below:

   - The geometry / drawing logic for the visible ruler strip
     (locating marks, dragging them, computing icon
     positions, painting the strip with tab markers).
   - The full Internalize / Externalize wire-format
     handlers, with Alien-component fall-back when a ruler's
     concrete type isn't registered at load time.

   This slice ships:

   - The complete TYPE surface (`Tab`, `TabArray`, `Attributes`,
     `Style`, `Ruler`, `Prop`, `UpdateMsg`, `Directory`,
     `SetAttrOp`, `NeutralizeMsg`).
   - All BB-faithful CONST declarations (mark kinds, icon
     codes, attribute opts bitmask positions, tab-type
     positions, geometry constants).
   - Simple data ops: `CopyTabs`, the per-axis `Set*` helpers
     (`SetFirst`, `SetLeft`, …), `Attributes.Equals`,
     `Attributes.Prop`.
   - ABSTRACT method declarations on `Directory` (`NewStyle`,
     `New`) so subclasses compile.

   Deferred (called out below):

   - The geometry / drawing surface (`Mark`, `Locate`,
     `PaintScale`, `Restore`, the layout passes).
   - `AlienAttributes`, `ReadAttr`, `WriteAttr` — depend on
     `Stores.Alien` and `Stores.alienComponent`, neither of
     which is ported yet.
   - The full Internalize / Externalize bodies; for now they
     read/write the version stamp and the body fields without
     the Alien-recovery branch.
   - `Stores.Join` calls — that runtime helper isn't ported.
*)

    IMPORT SYSTEM, Kernel, Strings, Services, Fonts, Ports, Stores,
        Models, Views, Controllers, Properties, Dialog,
        TextModels;

    CONST
        (** Attributes.valid / Prop.known / Prop.valid bit
            positions (also reused as Mark.kind values for
            the ruler's selectable marks). *)
        first* = 0;
        left*  = 1;
        right* = 2;
        lead*  = 3;
        asc*   = 4;
        dsc*   = 5;
        grid*  = 6;
        opts*  = 7;
        tabs*  = 8;

        (* Additional values for icon-kind marks. *)
        invalid = -1;

        firstIcon = 10;
        lastIcon  = 25;

        rightToggle = 10;

        gridDec = 12;
        gridVal = 13;
        gridInc = 14;

        leftFlush  = 16;
        centered   = 17;
        rightFlush = 18;
        justified  = 19;

        leadDec = 21;
        leadVal = 22;
        leadInc = 23;

        pageBrk = 25;

        modeIcons   = {leftFlush .. justified};
        validIcons  = {rightToggle,
                       gridDec .. gridInc,
                       leftFlush .. justified,
                       leadDec .. leadInc,
                       pageBrk};
        fieldIcons  = {gridVal, leadVal};

        (** Attributes.opts bit positions. *)
        leftAdjust*    = 0;
        rightAdjust*   = 1;
        noBreakInside* = 2;
        pageBreak*     = 3;
        parJoin*       = 4;
        rightFixed*    = 5;

        options = {leftAdjust .. rightFixed};   (* options mask *)
        adjMask = {leftAdjust, rightAdjust};

        (** Tab.type bit positions. *)
        maxTabs*   = 32;
        centerTab* = 0;
        rightTab*  = 1;
        barTab*    = 2;

        tabOptions = {centerTab .. barTab};   (* mask for valid options *)

        mm     = Ports.mm;
        inch16 = Ports.inch DIV 16;
        point  = Ports.point;

        tabBarHeight  = 11 * point;
        scaleHeight   = 10 * point;
        iconBarHeight = 14 * point;
        rulerHeight   = tabBarHeight + scaleHeight + iconBarHeight;

        iconHeight = 10 * point;
        iconWidth  = 12 * point;
        iconGap    = 2 * point;
        iconPin    = rulerHeight - (iconBarHeight - iconHeight) DIV 2;

        rulerChangeKey = "#Text:RulerChange";

        minVersion        = 0;
        maxAttrVersion*    = 2;
        maxStyleVersion*   = 0;
        maxStdStyleVersion = 0;
        maxRulerVersion*   = 0;
        maxStdRulerVersion = 0;


    TYPE
        (** A single tab stop — position in user units +
            type bitmask (centerTab / rightTab / barTab). *)
        Tab* = RECORD
            stop*: INTEGER;
            type*: SET
        END;

        (** Inline tab table.  BB notes "should be POINTER TO
            ARRAY OF Tab" but BlackBox keeps it inline so the
            ~256-byte structure can be copied by length-of-
            valid-prefix rather than a heap allocation per
            edit. *)
        TabArray* = RECORD
            len*: INTEGER;
            tab*: ARRAY maxTabs OF Tab
        END;

        (** Immutable paragraph-style attributes.  Once
            `init` flips to TRUE the contents are frozen —
            edits clone via `ModifiedAttr` rather than
            in-place mutation.  Extends `Stores.Store` so an
            Attributes round-trips through Stores.CopyOf. *)
        AttributesDesc* = EXTENSIBLE RECORD (Stores.StoreDesc)
            init-:                                BOOLEAN;
            first-, left-, right-, lead-,
            asc-, dsc-, grid-:                    INTEGER;
            opts-:                                SET;
            tabs-:                                TabArray
        END;
        Attributes* = POINTER TO AttributesDesc;

        (** Abstract Style — wraps an Attributes for sharing
            across multiple Rulers (BB economises by having
            many rulers point at the same Style). *)
        StyleDesc* = ABSTRACT RECORD (Models.ModelDesc)
            attr-: Attributes
        END;
        Style* = POINTER TO StyleDesc;

        (** Abstract Ruler — the View shape that paints a
            ruler strip on screen.  Carries the Style it
            renders.  Extends `Views.View`. *)
        RulerDesc* = ABSTRACT RECORD (Views.ViewDesc)
            style-: Style
        END;
        Ruler* = POINTER TO RulerDesc;

        (** Property bag for the geometry axes — what the
            UI publishes when the user selects a paragraph
            and asks "show me the ruler settings". *)
        PropDesc* = RECORD (Properties.PropertyDesc)
            first*, left*, right*, lead*,
            asc*,   dsc*,  grid*:                INTEGER;
            opts*:  RECORD val*, mask*: SET END;
            tabs*:  TabArray
        END;
        Prop* = POINTER TO PropDesc;

        (** Sent when a Style's Attributes pointer flips —
            every Ruler watching that Style repaints. *)
        UpdateMsg* = RECORD (Models.UpdateMsg)
            style*:   Style;
            oldAttr*: Attributes
        END;

        (** Abstract factory — concrete directories supply
            the leaf `Style` / `Ruler` types.  The host
            installs one as `dir` at startup. *)
        DirectoryDesc* = ABSTRACT RECORD
            attr-: Attributes
        END;
        Directory* = POINTER TO DirectoryDesc;

        (** Undo-op for a "I changed this ruler's attributes"
            edit — saves the previous attributes for replay. *)
        SetAttrOpDesc* = RECORD (Stores.OperationDesc)
            style*: Style;
            attr*:  Attributes
        END;
        SetAttrOp* = POINTER TO SetAttrOpDesc;

        (** Sent when a ruler should drop its transient
            selection / drag state. *)
        NeutralizeMsg* = RECORD (Views.Message) END;

        (** Concrete Style — no additional state beyond StyleDesc. *)
        StdStyleDesc* = RECORD (StyleDesc) END;
        StdStyle*     = POINTER TO StdStyleDesc;

        (** Concrete Ruler — records the painted width/height of the
            ruler strip in user units once the geometry slice lands. *)
        StdRulerDesc* = RECORD (RulerDesc)
            w-, h-: INTEGER
        END;
        StdRuler*     = POINTER TO StdRulerDesc;

        (** Concrete Directory — default factory used by the
            framework when no host-specific directory is installed. *)
        StdDirectoryDesc* = RECORD (DirectoryDesc) END;
        StdDirectory*     = POINTER TO StdDirectoryDesc;


    VAR
        (** Currently-installed factories — `dir` is mutable
            (the host may swap in a styled variant); `stdDir`
            is the immutable default the framework falls back
            on. *)
        dir-, stdDir-: Directory;
        std: StdDirectory;


    (* -- Tab helpers ---------------------------------------------------- *)

    (** Copy the valid prefix of `src` into `dst`.  BB notes
        this is "much faster than `:= all`" because a TabArray
        is 256 bytes but most rulers use <4 tabs. *)
    PROCEDURE CopyTabs* (IN src: TabArray; OUT dst: TabArray);
        VAR i, n: INTEGER;
    BEGIN
        n := src.len;
        dst.len := n;
        i := 0;
        WHILE i < n DO
            dst.tab[i] := src.tab[i];
            INC(i)
        END
    END CopyTabs;


    (* -- Attributes methods --------------------------------------------- *)

    (** Deep-equality on attribute records — exposed to
        consumers so a propagating style change can short-
        circuit on "the new and old attrs are equivalent". *)
    PROCEDURE (a: Attributes) Equals* (b: Attributes): BOOLEAN, NEW, EXTENSIBLE;
        VAR i: INTEGER; matches: BOOLEAN;
    BEGIN
        ASSERT(a.init, 20);
        ASSERT(b.init, 21);
        IF a = b THEN RETURN TRUE END;
        i := 0;
        matches := TRUE;
        WHILE matches & (i < a.tabs.len) DO
            IF (a.tabs.tab[i].stop # b.tabs.tab[i].stop)
            OR (a.tabs.tab[i].type # b.tabs.tab[i].type) THEN
                matches := FALSE
            END;
            INC(i)
        END;
        IF ~matches THEN RETURN FALSE END;
        RETURN
            (a.first = b.first) & (a.left = b.left) & (a.right = b.right)
            & (a.lead = b.lead) & (a.asc = b.asc) & (a.dsc = b.dsc)
            & (a.grid = b.grid) & (a.opts = b.opts)
            & (a.tabs.len = b.tabs.len)
    END Equals;

    (** Materialize a Prop carrying this Attributes' state —
        the standard "publish my settings" path. *)
    PROCEDURE (a: Attributes) Prop* (): Properties.Property, NEW, EXTENSIBLE;
        VAR p: Prop;
    BEGIN
        ASSERT(a.init, 20);
        NEW(p);
        p.known := {first .. tabs};
        p.valid := p.known;
        p.first := a.first;  p.left := a.left;  p.right := a.right;
        p.lead  := a.lead;   p.asc  := a.asc;   p.dsc   := a.dsc;
        p.grid  := a.grid;
        p.opts.val  := a.opts;
        p.opts.mask := options;
        CopyTabs(a.tabs, p.tabs);
        RETURN p
    END Prop;

    (** Initialise the attributes from a Prop — only used
        on a freshly-NEW'd Attributes (pre-`init`).  Sets
        every axis from the Prop, marks `init`. *)
    PROCEDURE (a: Attributes) InitFromProp* (p: Properties.Property), NEW, EXTENSIBLE;
    BEGIN
        ASSERT(~a.init, 20);
        a.init := TRUE
        (* Full BlackBox body walks the Prop's `known` mask
           and sets each axis on `a` to the Prop's value.
           Deferred here pending the helper plumbing — most
           callers go through ModifiedAttr/CopyFrom which use
           the BB body in a separate slice. *)
    END InitFromProp;

    (** Apply a Prop's mutations on top of this Attributes.
        The actual BB body lives in a follow-up slice — it's
        a long axis-by-axis ModifyFromProp routine.  EMPTY
        here so subclasses compile. *)
    PROCEDURE (a: Attributes) ModifyFromProp* (p: Properties.Property), NEW, EXTENSIBLE;
    BEGIN
    END ModifyFromProp;

    (** Deserialise this Attributes from `rd`.
        Wire layout (BlackBox-faithful):
          1 byte   — version stamp (0 .. maxAttrVersion)
          2 bytes  — first  (i16)
          2 bytes  — left   (i16)
          2 bytes  — right  (i16)
          2 bytes  — lead   (i16)
          2 bytes  — grid   (i16)
          4 bytes  — opts   (i32 / SET)
          2 bytes  — dsc    (i16)
          xint     — tab count (compressed)
          per tab: 2 bytes stop (i16) + 1 byte type
        No AlienAttributes fallback — Stores.Alien is not ported. *)
    PROCEDURE (a: AttributesDesc) Internalize* (VAR rd: Stores.Reader);
        VAR ver, n, i, stop: INTEGER; b: BYTE; opts: SET;
    BEGIN
        a.Internalize^(rd);
        rd.ReadVersion(minVersion, maxAttrVersion, ver);
        IF rd.cancelled THEN RETURN END;
        rd.ReadInt(a.first);
        IF rd.eof THEN RETURN END;
        rd.ReadInt(a.left);
        IF rd.eof THEN RETURN END;
        rd.ReadInt(a.right);
        IF rd.eof THEN RETURN END;
        rd.ReadInt(a.lead);
        IF rd.eof THEN RETURN END;
        rd.ReadInt(a.grid);
        IF rd.eof THEN RETURN END;
        rd.ReadSet(opts);
        IF rd.eof THEN RETURN END;
        a.opts := opts;
        rd.ReadInt(a.dsc);
        IF rd.eof THEN RETURN END;
        rd.ReadXInt(n);
        IF rd.eof THEN RETURN END;
        IF n > maxTabs THEN n := maxTabs END;
        a.tabs.len := n;
        i := 0;
        WHILE i < n DO
            rd.ReadInt(stop);
            IF rd.eof THEN a.tabs.len := i; RETURN END;
            a.tabs.tab[i].stop := stop;
            rd.ReadByte(b);
            IF rd.eof THEN a.tabs.len := i; RETURN END;
            a.tabs.tab[i].type := SYSTEM.VAL(SET, ORD(b));
            INC(i)
        END;
        a.init := TRUE
    END Internalize;

    (** Serialise this Attributes to `wr`.  Symmetric with Internalize. *)
    PROCEDURE (a: AttributesDesc) Externalize* (VAR wr: Stores.Writer);
        VAR i, n: INTEGER; b: BYTE;
    BEGIN
        a.Externalize^(wr);
        wr.WriteVersion(maxAttrVersion);
        wr.WriteInt(a.first);
        wr.WriteInt(a.left);
        wr.WriteInt(a.right);
        wr.WriteInt(a.lead);
        wr.WriteInt(a.grid);
        wr.WriteSet(a.opts);
        wr.WriteInt(a.dsc);
        n := a.tabs.len;
        wr.WriteXInt(n);
        i := 0;
        WHILE i < n DO
            wr.WriteInt(a.tabs.tab[i].stop);
            b := SHORT(SHORT(SYSTEM.VAL(INTEGER, a.tabs.tab[i].type)));
            wr.WriteByte(b);
            INC(i)
        END
    END Externalize;


    (* -- StdStyle methods ----------------------------------------------- *)

    PROCEDURE (s: StdStyleDesc) Domain* (): Stores.Domain;
    BEGIN RETURN NIL END Domain;

    PROCEDURE (s: StdStyleDesc) Internalize* (VAR rd: Stores.Reader);
        VAR ver, handle: INTEGER; store: Stores.Store;
    BEGIN
        s.Internalize^(rd);
        rd.ReadVersion(minVersion, maxStdStyleVersion, ver);
        IF rd.cancelled THEN RETURN END;
        rd.ReadStore(handle);
        IF rd.cancelled THEN RETURN END;
        IF handle # 0 THEN
            store := Stores.NewStore(handle);
            IF (store # NIL) & (store IS Attributes) THEN
                s.attr := store(Attributes)
            END
        END
    END Internalize;

    PROCEDURE (s: StdStyleDesc) Externalize* (VAR wr: Stores.Writer);
    BEGIN
        s.Externalize^(wr);
        wr.WriteVersion(maxStdStyleVersion);
        wr.WriteStore(s.attr)
    END Externalize;


    (* -- StdRuler methods ----------------------------------------------- *)

    PROCEDURE (r: StdRulerDesc) Domain* (): Stores.Domain;
    BEGIN RETURN NIL END Domain;

    PROCEDURE (v: StdRulerDesc) Restore* (f: Views.Frame; l, t, r, b: INTEGER);
    BEGIN
        (* Ruler painting deferred — geometry slice not yet ported. *)
    END Restore;

    PROCEDURE (r: StdRulerDesc) Internalize* (VAR rd: Stores.Reader);
        VAR ver, handle: INTEGER; store: Stores.Store;
    BEGIN
        r.Internalize^(rd);
        IF rd.cancelled THEN RETURN END;
        rd.ReadVersion(minVersion, maxStdRulerVersion, ver);
        IF rd.cancelled THEN RETURN END;
        rd.ReadStore(handle);
        IF rd.cancelled THEN RETURN END;
        IF handle # 0 THEN
            store := Stores.NewStore(handle);
            IF (store # NIL) & (store IS Style) THEN
                r.InitStyle(store(Style))
            END
        END
    END Internalize;

    PROCEDURE (r: StdRulerDesc) Externalize* (VAR wr: Stores.Writer);
    BEGIN
        r.Externalize^(wr);
        wr.WriteVersion(maxStdRulerVersion);
        wr.WriteStore(r.style)
    END Externalize;


    (* -- StdDirectory methods ------------------------------------------ *)

    PROCEDURE (d: StdDirectoryDesc) NewStyle* (attr: Attributes): Style;
        VAR s: StdStyle;
    BEGIN
        ASSERT(attr # NIL, 20);
        ASSERT(attr.init, 21);
        NEW(s);
        s.attr := attr;
        RETURN s
    END NewStyle;

    PROCEDURE (d: StdDirectoryDesc) New* (style: Style): Ruler;
        VAR r: StdRuler;
    BEGIN
        ASSERT(style # NIL, 20);
        NEW(r);
        r.InitStyle(style);
        RETURN r
    END New;


    (* -- SetAttrOp operation ------------------------------------------- *)

    (** Undo/redo toggle for a ruler attribute change.
        Swaps style.attr with the saved attr copy and broadcasts
        an UpdateMsg so all watchers (ruler views, text views)
        can repaint. *)
    PROCEDURE (op: SetAttrOp) Do*;
        VAR old: Attributes; msg: UpdateMsg;
    BEGIN
        old := op.style.attr;
        op.style.attr := op.attr;   (* install the new / restored attrs  *)
        op.attr := old;             (* op now holds the old for reversal  *)
        msg.style   := op.style;
        msg.oldAttr := old;
        Models.Broadcast(op.style, msg)
    END Do;


    (* -- Style methods ------------------------------------------------- *)

    (** Bind new Attributes onto this Style and broadcast a
        TextRulers.UpdateMsg through the model's sequencer.
        Idempotent: does nothing when the new attrs equal the
        current ones (avoids spurious undo entries). *)
    PROCEDURE (s: Style) SetAttr* (attr: Attributes), NEW, EXTENSIBLE;
        VAR op: SetAttrOp;
    BEGIN
        ASSERT(attr # NIL, 20);
        ASSERT(attr.init, 21);
        IF ~s.attr.Equals(attr) THEN
            NEW(op);
            op.style := s;
            op.attr  := attr;
            Models.Do(s, rulerChangeKey, op)
        END
    END SetAttr;


    (* -- Directory abstract methods ------------------------------------ *)

    (** Set the directory's default Attributes — used by host
        startup to install a customised default ruler. *)
    PROCEDURE (d: Directory) SetAttr* (attr: Attributes), NEW, EXTENSIBLE;
    BEGIN
        ASSERT(attr # NIL, 20);
        ASSERT(attr.init, 21);
        d.attr := attr
    END SetAttr;

    (** Allocate a fresh Style around the given Attributes. *)
    PROCEDURE (d: Directory) NewStyle* (attr: Attributes): Style, NEW, ABSTRACT;

    (** Allocate a fresh Ruler around the given Style. *)
    PROCEDURE (d: Directory) New* (style: Style): Ruler, NEW, ABSTRACT;


    (* -- Ruler methods (declarations only) ------------------------------ *)

    (** Bind `r.style` once; idempotent if the same style
        re-binds, asserts otherwise.  BB also wires the
        Stores.Join domain link — deferred. *)
    PROCEDURE (r: Ruler) InitStyle* (s: Style), NEW;
    BEGIN
        ASSERT((r.style = NIL) OR (r.style = s), 20);
        ASSERT(s # NIL, 21);
        ASSERT(s.attr # NIL, 22);
        r.style := s
    END InitStyle;


    (* -- Prop methods --------------------------------------------------- *)

    (** Intersect this property with `q` — narrows `valid`
        to bits both agree on.  Concrete override of the
        ABSTRACT IntersectWith on Properties.Property. *)
    PROCEDURE (p: Prop) IntersectWith* (q: Properties.Property; OUT equal: BOOLEAN);
        VAR valid: SET; c, m: SET; eq: BOOLEAN; i: INTEGER;
    BEGIN
        WITH q: Prop DO
            valid := p.valid * q.valid; equal := TRUE;
            IF (first IN valid) & (p.first # q.first) THEN EXCL(valid, first) END;
            IF (left  IN valid) & (p.left  # q.left)  THEN EXCL(valid, left)  END;
            IF (right IN valid) & (p.right # q.right) THEN EXCL(valid, right) END;
            IF (lead  IN valid) & (p.lead  # q.lead)  THEN EXCL(valid, lead)  END;
            IF (asc   IN valid) & (p.asc   # q.asc)   THEN EXCL(valid, asc)   END;
            IF (dsc   IN valid) & (p.dsc   # q.dsc)   THEN EXCL(valid, dsc)   END;
            IF (grid  IN valid) & (p.grid  # q.grid)  THEN EXCL(valid, grid)  END;
            IF opts IN valid THEN
                Properties.IntersectSelections(p.opts.val, p.opts.mask,
                                               q.opts.val, q.opts.mask, c, m, eq);
                IF m = {} THEN
                    EXCL(valid, opts)
                ELSIF ~eq THEN
                    p.opts.mask := m; equal := FALSE
                END
            END;
            IF tabs IN valid THEN
                IF p.tabs.len = q.tabs.len THEN
                    i := 0;
                    WHILE (i < p.tabs.len)
                        & (p.tabs.tab[i].stop = q.tabs.tab[i].stop)
                        & (p.tabs.tab[i].type = q.tabs.tab[i].type) DO
                        INC(i)
                    END;
                    IF i < p.tabs.len THEN
                        EXCL(valid, tabs); equal := FALSE
                    END
                ELSE
                    EXCL(valid, tabs); equal := FALSE
                END
            END;
            IF p.valid # valid THEN p.valid := valid; equal := FALSE END
        END
    END IntersectWith;


    (* -- Convenience setters ------------------------------------------- *)

    (** Allocate a fresh Attributes pre-loaded with every axis from `a`.
        The caller then mutates the single axis it cares about and passes
        the result to `r.style.SetAttr`. *)
    PROCEDURE NewAttrFrom (a: Attributes): Attributes;
        VAR b: Attributes;
    BEGIN
        ASSERT(a # NIL, 20);
        ASSERT(a.init, 21);
        NEW(b);
        b.init  := TRUE;
        b.first := a.first;
        b.left  := a.left;
        b.right := a.right;
        b.lead  := a.lead;
        b.asc   := a.asc;
        b.dsc   := a.dsc;
        b.grid  := a.grid;
        b.opts  := a.opts;
        CopyTabs(a.tabs, b.tabs);
        RETURN b
    END NewAttrFrom;


    PROCEDURE SetFirst* (r: Ruler; x: INTEGER);
        VAR a: Attributes;
    BEGIN
        ASSERT(r # NIL, 20); ASSERT(r.style # NIL, 21);
        a := NewAttrFrom(r.style.attr);
        a.first := x;
        r.style.SetAttr(a)
    END SetFirst;

    PROCEDURE SetLeft* (r: Ruler; x: INTEGER);
        VAR a: Attributes;
    BEGIN
        ASSERT(r # NIL, 20); ASSERT(r.style # NIL, 21);
        a := NewAttrFrom(r.style.attr);
        a.left := x;
        r.style.SetAttr(a)
    END SetLeft;

    (** Set the right margin without touching the rightFixed flag. *)
    PROCEDURE SetRight* (r: Ruler; x: INTEGER);
        VAR a: Attributes;
    BEGIN
        ASSERT(r # NIL, 20); ASSERT(r.style # NIL, 21);
        a := NewAttrFrom(r.style.attr);
        a.right := x;
        r.style.SetAttr(a)
    END SetRight;

    (** Set the right margin AND mark it as fixed (rightFixed flag). *)
    PROCEDURE SetFixedRight* (r: Ruler; x: INTEGER);
        VAR a: Attributes;
    BEGIN
        ASSERT(r # NIL, 20); ASSERT(r.style # NIL, 21);
        a := NewAttrFrom(r.style.attr);
        a.right := x;
        INCL(a.opts, rightFixed);
        r.style.SetAttr(a)
    END SetFixedRight;

    PROCEDURE SetLead* (r: Ruler; h: INTEGER);
        VAR a: Attributes;
    BEGIN
        ASSERT(r # NIL, 20); ASSERT(r.style # NIL, 21);
        a := NewAttrFrom(r.style.attr);
        a.lead := h;
        r.style.SetAttr(a)
    END SetLead;

    PROCEDURE SetAsc* (r: Ruler; h: INTEGER);
        VAR a: Attributes;
    BEGIN
        ASSERT(r # NIL, 20); ASSERT(r.style # NIL, 21);
        a := NewAttrFrom(r.style.attr);
        a.asc := h;
        r.style.SetAttr(a)
    END SetAsc;

    PROCEDURE SetDsc* (r: Ruler; h: INTEGER);
        VAR a: Attributes;
    BEGIN
        ASSERT(r # NIL, 20); ASSERT(r.style # NIL, 21);
        a := NewAttrFrom(r.style.attr);
        a.dsc := h;
        r.style.SetAttr(a)
    END SetDsc;

    PROCEDURE SetGrid* (r: Ruler; h: INTEGER);
        VAR a: Attributes;
    BEGIN
        ASSERT(r # NIL, 20); ASSERT(r.style # NIL, 21);
        a := NewAttrFrom(r.style.attr);
        a.grid := h;
        r.style.SetAttr(a)
    END SetGrid;

    (** Paragraph alignment: left-flush only (leftAdjust set,
        rightAdjust cleared). *)
    PROCEDURE SetLeftFlush* (r: Ruler);
        VAR a: Attributes;
    BEGIN
        ASSERT(r # NIL, 20); ASSERT(r.style # NIL, 21);
        a := NewAttrFrom(r.style.attr);
        a.opts := (a.opts - adjMask) + {leftAdjust};
        r.style.SetAttr(a)
    END SetLeftFlush;

    (** Paragraph alignment: right-flush only (rightAdjust set,
        leftAdjust cleared). *)
    PROCEDURE SetRightFlush* (r: Ruler);
        VAR a: Attributes;
    BEGIN
        ASSERT(r # NIL, 20); ASSERT(r.style # NIL, 21);
        a := NewAttrFrom(r.style.attr);
        a.opts := (a.opts - adjMask) + {rightAdjust};
        r.style.SetAttr(a)
    END SetRightFlush;

    (** Paragraph alignment: centered (both adjust bits cleared). *)
    PROCEDURE SetCentered* (r: Ruler);
        VAR a: Attributes;
    BEGIN
        ASSERT(r # NIL, 20); ASSERT(r.style # NIL, 21);
        a := NewAttrFrom(r.style.attr);
        a.opts := a.opts - adjMask;
        r.style.SetAttr(a)
    END SetCentered;

    (** Paragraph alignment: fully justified (both adjust bits set). *)
    PROCEDURE SetJustified* (r: Ruler);
        VAR a: Attributes;
    BEGIN
        ASSERT(r # NIL, 20); ASSERT(r.style # NIL, 21);
        a := NewAttrFrom(r.style.attr);
        a.opts := a.opts + adjMask;
        r.style.SetAttr(a)
    END SetJustified;

    (** Toggle the noBreakInside flag. *)
    PROCEDURE SetNoBreakInside* (r: Ruler);
        VAR a: Attributes;
    BEGIN
        ASSERT(r # NIL, 20); ASSERT(r.style # NIL, 21);
        a := NewAttrFrom(r.style.attr);
        IF noBreakInside IN a.opts THEN EXCL(a.opts, noBreakInside)
        ELSE INCL(a.opts, noBreakInside)
        END;
        r.style.SetAttr(a)
    END SetNoBreakInside;

    (** Toggle the pageBreak flag. *)
    PROCEDURE SetPageBreak* (r: Ruler);
        VAR a: Attributes;
    BEGIN
        ASSERT(r # NIL, 20); ASSERT(r.style # NIL, 21);
        a := NewAttrFrom(r.style.attr);
        IF pageBreak IN a.opts THEN EXCL(a.opts, pageBreak)
        ELSE INCL(a.opts, pageBreak)
        END;
        r.style.SetAttr(a)
    END SetPageBreak;

    (** Toggle the parJoin flag. *)
    PROCEDURE SetParJoin* (r: Ruler);
        VAR a: Attributes;
    BEGIN
        ASSERT(r # NIL, 20); ASSERT(r.style # NIL, 21);
        a := NewAttrFrom(r.style.attr);
        IF parJoin IN a.opts THEN EXCL(a.opts, parJoin)
        ELSE INCL(a.opts, parJoin)
        END;
        r.style.SetAttr(a)
    END SetParJoin;


BEGIN
    NEW(std);
    (* Build a default Attributes — left-flush, zero margins, no tabs. *)
    NEW(std.attr);
    std.attr.init   := TRUE;
    std.attr.opts   := {leftAdjust};
    std.attr.first  := 0;
    std.attr.left   := 0;
    std.attr.right  := 0;
    std.attr.lead   := 0;
    std.attr.asc    := 0;
    std.attr.dsc    := 0;
    std.attr.grid   := 0;
    std.attr.tabs.len := 0;
    stdDir := std;
    dir    := std

END TextRulers.
