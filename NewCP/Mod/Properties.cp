MODULE Properties;
(*
   NewCP port of BlackBox `System/Mod/Properties.odc`.

   The property-bag / preference-bag layer that sits between views
   and the dialog/UI runtime.

   Two layered concerns:

   1. The property / preference TYPE TREE — pure data records
      that subclassing modules (TextViews, FormViews, Containers,
      …) extend to expose their typed state.  Extending the
      `Preference` chain is how a parent view asks a child "what
      bounds / size / focus state do you want?".

   2. A bundle of module-level procedures (`Insert`, `CopyOfList`,
      `Merge`, `Intersect`, `PreferredSize`, `ThisType`, …) that
      operate on `Property` lists and `Preference` message
      round-trips.

   Divergences from BlackBox:
   - `SYSTEM.TYP(p^)` (type-descriptor address) replaced by
     `SYSTEM.VAL(INTEGER, Kernel.TypeOf(p))` for the sort key.
   - `SYSTEM.MOVE` arguments use pointer values directly (pointer
     value IS the address of the pointed-to record on our ABI).
   - `Services.TypeLevel` takes `ANYPTR` not `ANYREC`; all call
     sites in this module pass pointer values so the behaviour
     is identical.
*)

    IMPORT SYSTEM, Kernel, Math, Services, Stores, Fonts, Views,
           Controllers, Dialog;

    CONST
        (** StdProp.known / valid bitmask positions. *)
        color*    = 0;
        typeface* = 1;
        size*     = 2;
        style*    = 3;
        weight*   = 4;

        (** SizeProp.known / valid bitmask positions. *)
        width*  = 0;
        height* = 1;

        (** PollVerbsMsg limit on verbs per poll. *)
        maxVerbs* = 16;

        (** PollPickMsg.mark / PollPick mark flag. *)
        noMark* = FALSE;
        mark*   = TRUE;

        (** PollPickMsg.show / PollPick show flag. *)
        hide* = FALSE;
        show* = TRUE;


    TYPE
        (** Sorted property-list head.  Concrete property classes
            (`StdProp`, `SizeProp`, framework-specific extensions)
            extend this and override `IntersectWith`. *)
        PropertyDesc* = ABSTRACT RECORD
            next-:              Property;  (** sorted by type-descriptor address *)
            known*, readOnly*:  SET;
            valid*:             SET
        END;
        Property* = POINTER TO PropertyDesc;

        StdPropDesc* = RECORD (PropertyDesc)
            color*:    Dialog.Color;
            typeface*: Fonts.Typeface;
            size*:     INTEGER;
            style*:    RECORD val*, mask*: SET END;
            weight*:   INTEGER
        END;
        StdProp* = POINTER TO StdPropDesc;

        SizePropDesc* = RECORD (PropertyDesc)
            width*, height*: INTEGER
        END;
        SizeProp* = POINTER TO SizePropDesc;


        (** Re-export the Views property-message base. *)
        Message* = Views.PropMessage;

        PollMsg* = RECORD (Message)
            prop*: Property
        END;

        SetMsg* = RECORD (Message)
            old*, prop*: Property
        END;


        (** Abstract Preference base. *)
        Preference* = ABSTRACT RECORD (Message) END;

        ResizePref* = RECORD (Preference)
            fixed*:        BOOLEAN;
            horFitToPage*: BOOLEAN;
            verFitToPage*: BOOLEAN;
            horFitToWin*:  BOOLEAN;
            verFitToWin*:  BOOLEAN
        END;

        SizePref* = RECORD (Preference)
            w*, h*:           INTEGER;
            fixedW*, fixedH*: BOOLEAN
        END;

        BoundsPref* = RECORD (Preference)
            w*, h*: INTEGER
        END;

        FocusPref* = RECORD (Preference)
            atLocation*:          BOOLEAN;
            x*, y*:               INTEGER;
            hotFocus*, setFocus*: BOOLEAN
        END;

        ControlPref* = RECORD (Preference)
            char*:     CHAR;
            focus*:    Views.View;
            getFocus*: BOOLEAN;
            accepts*:  BOOLEAN
        END;

        TypePref* = RECORD (Preference)
            type*: Stores.TypeName;
            view*: Views.View
        END;


        PollVerbMsg* = RECORD (Message)
            verb*:                INTEGER;
            label*:               ARRAY 64 OF CHAR;
            disabled*, checked*:  BOOLEAN
        END;

        DoVerbMsg* = RECORD (Message)
            verb*:  INTEGER;
            frame*: Views.Frame
        END;


        (** Controller-side wrappers. *)
        CollectMsg* = RECORD (Controllers.Message)
            poll*: PollMsg
        END;

        EmitMsg* = RECORD (Controllers.RequestMessage)
            set*: SetMsg
        END;

        PollPickMsg* = RECORD (Controllers.TransferMessage)
            mark*: BOOLEAN;
            show*: BOOLEAN;
            dest*: Views.Frame
        END;

        PickMsg* = RECORD (Controllers.TransferMessage)
            prop*: Property
        END;


    VAR
        (** Bumped on every Property-list change. *)
        era-: INTEGER;


    (* ---- PropertyDesc methods ------------------------------------------ *)

    (** Intersect this property bag with `q`.  ABSTRACT — StdProp
        and SizeProp override. *)
    PROCEDURE (p: Property) IntersectWith* (q: Property;
                                             OUT equal: BOOLEAN),
                                            NEW, ABSTRACT;


    (* ---- IntersectSelections (defined before methods that call it) ------ *)

    (** Intersect two style-selection pairs (val + mask SETs).
        Result `c`/`cMask` is the narrowed selection. *)
    PROCEDURE IntersectSelections* (a, aMask, b, bMask: SET;
                                    OUT c, cMask: SET;
                                    OUT equal: BOOLEAN);
    BEGIN
        cMask := aMask * bMask - (a / b);
        c     := a * cMask;
        equal := (aMask = bMask) & (bMask = cMask)
    END IntersectSelections;


    (* ---- StdProp -------------------------------------------------------- *)

    PROCEDURE (p: StdProp) IntersectWith* (q: Property; OUT equal: BOOLEAN);
        VAR valid: SET; c, m: SET; eq: BOOLEAN;
    BEGIN
        WITH q: StdProp DO
            valid := p.valid * q.valid; equal := TRUE;
            IF p.color.val # q.color.val   THEN EXCL(valid, color)    END;
            IF p.typeface  # q.typeface    THEN EXCL(valid, typeface)  END;
            IF p.size      # q.size        THEN EXCL(valid, size)      END;
            IntersectSelections(p.style.val, p.style.mask,
                                q.style.val, q.style.mask, c, m, eq);
            IF m = {} THEN
                EXCL(valid, style)
            ELSIF (style IN valid) & ~eq THEN
                p.style.mask := m; equal := FALSE
            END;
            IF p.weight # q.weight THEN EXCL(valid, weight) END;
            IF p.valid # valid THEN p.valid := valid; equal := FALSE END
        END
    END IntersectWith;


    (* ---- SizeProp ------------------------------------------------------- *)

    PROCEDURE (p: SizeProp) IntersectWith* (q: Property; OUT equal: BOOLEAN);
        VAR valid: SET;
    BEGIN
        WITH q: SizeProp DO
            valid := p.valid * q.valid; equal := TRUE;
            IF p.width  # q.width  THEN EXCL(valid, width)  END;
            IF p.height # q.height THEN EXCL(valid, height) END;
            IF p.valid # valid THEN p.valid := valid; equal := FALSE END
        END
    END IntersectWith;


    (* ---- Era --------------------------------------------------------------- *)

    (** Bump the property-cache era — called whenever the focused
        view's property bag changes, so cached snapshots become
        stale. *)
    PROCEDURE IncEra*;
    BEGIN
        INC(era)
    END IncEra;


    (* ---- Collect / emit -------------------------------------------------- *)

    (** Broadcast a CollectMsg and return the assembled property
        list from the focused view. *)
    PROCEDURE CollectProp* (OUT prop: Property);
        VAR msg: CollectMsg;
    BEGIN
        msg.poll.prop := NIL;
        Controllers.Forward(msg);
        prop := msg.poll.prop
    END CollectProp;

    (** Collect properties and find the StdProp entry.
        Always returns a non-NIL StdProp (empty bag if none found). *)
    PROCEDURE CollectStdProp* (OUT prop: StdProp);
        VAR p: Property;
    BEGIN
        CollectProp(p);
        WHILE (p # NIL) & ~(p IS StdProp) DO p := p.next END;
        IF p # NIL THEN
            prop := p(StdProp); prop.next := NIL
        ELSE
            NEW(prop); prop.known := {}
        END;
        prop.valid      := prop.valid * prop.known;
        prop.style.val  := prop.style.val * prop.style.mask
    END CollectStdProp;

    (** Broadcast an EmitMsg carrying the new property bag. *)
    PROCEDURE EmitProp* (old, prop: Property);
        VAR msg: EmitMsg;
    BEGIN
        IF prop # NIL THEN
            msg.set.old := old; msg.set.prop := prop;
            Controllers.Forward(msg)
        END
    END EmitProp;


    (* ---- Pick ------------------------------------------------------------ *)

    (** Poll for a pick destination at screen coordinates
        `(x, y)`. *)
    PROCEDURE PollPick* (x, y: INTEGER;
                          source: Views.Frame;
                          sourceX, sourceY: INTEGER;
                          mark, show: BOOLEAN;
                          OUT dest: Views.Frame;
                          OUT destX, destY: INTEGER);
        VAR msg: PollPickMsg;
    BEGIN
        ASSERT(source # NIL, 20);
        msg.mark := mark; msg.show := show; msg.dest := NIL;
        Controllers.Transfer(x, y, source, sourceX, sourceY, msg);
        dest := msg.dest; destX := msg.x; destY := msg.y
    END PollPick;

    (** Pick properties from the view at `(x, y)`. *)
    PROCEDURE Pick* (x, y: INTEGER;
                     source: Views.Frame;
                     sourceX, sourceY: INTEGER;
                     OUT prop: Property);
        VAR msg: PickMsg;
    BEGIN
        ASSERT(source # NIL, 20);
        msg.prop := NIL;
        Controllers.Transfer(x, y, source, sourceX, sourceY, msg);
        prop := msg.prop
    END Pick;


    (* ---- Property-list construction helpers ----------------------------- *)

    (** Stable integer sort key for type `t` — the address of the
        type descriptor in static memory, guaranteed unique per type
        and constant across GC cycles. *)
    PROCEDURE TypeKey (p: ANYPTR): INTEGER;
        VAR t: Kernel.Type;
    BEGIN
        t := Kernel.TypeOf(p);
        IF t = NIL THEN RETURN 0 END;
        RETURN SYSTEM.VAL(INTEGER, t)
    END TypeKey;

    (** Insert `x` into the sorted property list `list`.
        The list is kept sorted by ascending type-descriptor address
        so that each concrete type appears at most once.  If an entry
        of the same type already exists it is replaced by `x`. *)
    PROCEDURE Insert* (VAR list: Property; x: Property);
        VAR p, q: Property; ta: INTEGER;
    BEGIN
        ASSERT(x # NIL, 20);
        ASSERT(x.next = NIL, 21);
        ASSERT(x # list, 22);
        ASSERT(x.valid - x.known = {}, 23);
        IF list # NIL THEN
            ASSERT(list.valid - list.known = {}, 24);
            ASSERT(Services.TypeLevel(list) = 1, 25)
        END;
        ta := TypeKey(x);
        ASSERT(Services.TypeLevel(x) = 1, 26);
        p := list; q := NIL;
        WHILE (p # NIL) & (TypeKey(p) < ta) DO
            q := p; p := p.next
        END;
        IF (p # NIL) & (TypeKey(p) = ta) THEN
            x.next := p.next
        ELSE
            x.next := p
        END;
        IF q # NIL THEN q.next := x ELSE list := x END
    END Insert;

    (** Return a fresh copy of the property list rooted at `p`.
        Each entry is allocated with the same runtime type and
        its fields copied verbatim; `next` is relinked in order. *)
    PROCEDURE CopyOfList* (p: Property): Property;
        VAR q, r, s: Property; t: Kernel.Type;
    BEGIN
        q := NIL; s := NIL;
        WHILE p # NIL DO
            ASSERT(Services.TypeLevel(p) = 1, 20);
            t := Kernel.TypeOf(p);
            Kernel.NewObj(r, t); ASSERT(r # NIL, 23);
            SYSTEM.MOVE(p, r, Kernel.SizeOf(t));
            r.next := NIL;
            IF q # NIL THEN q.next := r ELSE s := r END;
            q := r; p := p.next
        END;
        RETURN s
    END CopyOfList;

    (** Return a fresh copy of a single property `p`.
        Returns NIL if `p = NIL`. *)
    PROCEDURE CopyOf* (p: Property): Property;
        VAR r: Property; t: Kernel.Type;
    BEGIN
        r := NIL;
        IF p # NIL THEN
            ASSERT(Services.TypeLevel(p) = 1, 20);
            t := Kernel.TypeOf(p);
            Kernel.NewObj(r, t); ASSERT(r # NIL, 23);
            SYSTEM.MOVE(p, r, Kernel.SizeOf(t));
            r.next := NIL
        END;
        RETURN r
    END CopyOf;

    (** Merge `override` into `base` (both sorted).  Entries from
        `override` replace same-type entries in `base`; new types
        are spliced in at the correct sorted position.
        `override` is consumed (set to NIL). *)
    PROCEDURE Merge* (VAR base, override: Property);
        VAR p, q, r, s: Property; tp, tr: INTEGER;
    BEGIN
        ASSERT((base # override) OR (base = NIL), 20);
        p := base; q := NIL; r := override; override := NIL;
        IF p # NIL THEN
            tp := TypeKey(p);
            ASSERT(Services.TypeLevel(p) = 1, 21)
        END;
        IF r # NIL THEN
            tr := TypeKey(r);
            ASSERT(Services.TypeLevel(r) = 1, 22)
        END;
        WHILE (p # NIL) & (r # NIL) DO
            ASSERT(p # r, 23);
            WHILE (p # NIL) & (tp < tr) DO
                q := p; p := p.next;
                IF p # NIL THEN tp := TypeKey(p) END
            END;
            IF p # NIL THEN
                IF tp = tr THEN
                    s := p.next; p.next := NIL; p := s;
                    IF p # NIL THEN tp := TypeKey(p) END
                END;
                s := r.next;
                IF q # NIL THEN q.next := r ELSE base := r END;
                q := r; r.next := p; r := s;
                IF r # NIL THEN tr := TypeKey(r) END
            END
        END;
        IF r # NIL THEN
            IF q # NIL THEN q.next := r ELSE base := r END
        END
    END Merge;

    (** Intersect two sorted property lists.  Entries present in
        both `list` and `x` are kept (with their IntersectWith
        narrowing applied); entries in only one list are discarded.
        `equal` is TRUE iff the lists had exactly the same types
        and all IntersectWith calls returned equal=TRUE. *)
    PROCEDURE Intersect* (VAR list: Property; x: Property;
                           OUT equal: BOOLEAN);
        VAR l, p, q, r, s: Property; plen, rlen, ta: INTEGER;
            filtered: BOOLEAN;
    BEGIN
        ASSERT((x # list) OR (list = NIL), 20);
        IF list # NIL THEN ASSERT(Services.TypeLevel(list) = 1, 21) END;
        IF x    # NIL THEN ASSERT(Services.TypeLevel(x)    = 1, 22) END;

        p := list; s := NIL; list := NIL; l := NIL; plen := 0;
        r := x; rlen := 0; filtered := FALSE;

        WHILE (p # NIL) & (r # NIL) DO
            q := p.next; p.next := NIL; INC(plen);
            ta := TypeKey(p);
            WHILE (r # NIL) & (TypeKey(r) < ta) DO
                r := r.next; INC(rlen)
            END;
            IF (r # NIL) & (TypeKey(r) = ta) THEN
                ASSERT(r # p, 23);
                IF l # NIL THEN s.next := p ELSE l := p END;
                s := p;
                p.known := p.known + r.known;
                p.IntersectWith(r, equal);
                filtered := filtered OR ~equal OR (p.valid # r.valid);
                r := r.next; INC(rlen)
            END;
            p := q
        END;

        list  := l;
        equal := (p = NIL) & (r = NIL) & (plen = rlen) & ~filtered
    END Intersect;


    (* ---- Geometry helpers ----------------------------------------------- *)

    (** Query `v` for its preferred size, clamping to [min..max].
        Uses a SizePref round-trip; if the view doesn't respond,
        `defW` / `defH` are used.  Caller presets `w` / `h` to its
        own preference; the view may narrow them. *)
    PROCEDURE PreferredSize* (v: Views.View;
                               minW, maxW, minH, maxH, defW, defH: INTEGER;
                               VAR w, h: INTEGER);
        VAR p: SizePref;
    BEGIN
        ASSERT(Views.undefined < minW, 20); ASSERT(minW < maxW, 21);
        ASSERT(Views.undefined < minH, 23); ASSERT(minH < maxH, 24);
        ASSERT(Views.undefined <= defW, 26);
        ASSERT(Views.undefined <= defH, 28);
        IF (w < Views.undefined) OR (w > maxW) THEN w := defW END;
        IF (h < Views.undefined) OR (h > maxH) THEN h := defH END;
        p.w := w; p.h := h; p.fixedW := FALSE; p.fixedH := FALSE;
        Views.HandlePropMsg(v, p); w := p.w; h := p.h;
        IF w = Views.undefined THEN w := defW END;
        IF h = Views.undefined THEN h := defH END;
        IF w < minW THEN w := minW ELSIF w > maxW THEN w := maxW END;
        IF h < minH THEN h := minH ELSIF h > maxH THEN h := maxH END
    END PreferredSize;

    (** Snap `(w, h)` so that `w / h = scaleW / scaleH`, minimising
        the change in area.  `fixedW` / `fixedH` constrain one axis. *)
    PROCEDURE ProportionalConstraint* (scaleW, scaleH: INTEGER;
                                        fixedW, fixedH: BOOLEAN;
                                        VAR w, h: INTEGER);
        VAR area: REAL;
    BEGIN
        ASSERT(scaleW > Views.undefined, 22);
        ASSERT(scaleH > Views.undefined, 23);
        IF fixedH THEN
            ASSERT(~fixedW, 24);
            ASSERT(h > Views.undefined, 21);
            area := h; area := area * scaleW;
            w := SHORT(ENTIER(area / scaleH))
        ELSIF fixedW THEN
            ASSERT(w > Views.undefined, 20);
            area := w; area := area * scaleH;
            h := SHORT(ENTIER(area / scaleW))
        ELSE
            ASSERT(w > Views.undefined, 20); ASSERT(h > Views.undefined, 21);
            area := w; area := area * h;
            w := SHORT(ENTIER(Math.Sqrt(area * scaleW / scaleH)));
            h := SHORT(ENTIER(Math.Sqrt(area * scaleH / scaleW)))
        END
    END ProportionalConstraint;

    (** Snap point `(x, y)` to the nearest grid point. *)
    PROCEDURE GridConstraint* (gridX, gridY: INTEGER; VAR x, y: INTEGER);
        VAR dx, dy: INTEGER;
    BEGIN
        ASSERT(gridX > Views.undefined, 20);
        ASSERT(gridY > Views.undefined, 21);
        dx := x MOD gridX;
        IF dx < gridX DIV 2 THEN DEC(x, dx) ELSE INC(x, (-x) MOD gridX) END;
        dy := y MOD gridY;
        IF dy < gridY DIV 2 THEN DEC(y, dy) ELSE INC(y, (-y) MOD gridY) END
    END GridConstraint;


    (* ---- Type look-up ---------------------------------------------------- *)

    (** Ask `view` for the first embedded view of the named type.
        Sends a TypePref round-trip and returns `msg.view`. *)
    PROCEDURE ThisType* (view: Views.View;
                          type: Stores.TypeName): Views.View;
        VAR msg: TypePref;
    BEGIN
        msg.type := type; msg.view := NIL;
        Views.HandlePropMsg(view, msg);
        RETURN msg.view
    END ThisType;


END Properties.
