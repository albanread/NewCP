MODULE HostStoresProbe;
(* Smoke probes for the typed Stores reader and factory surface.
   The fixture path is hard-coded; the Rust integration test stages
   `Empty.odc` before invoking. *)

IMPORT Stores, Kernel;

TYPE
    (** Concrete Stores.StoreDesc subclass used by
        InternalizeDispatches: `Internalize` reads two body bytes
        and records them.  Lives at module scope because CP requires
        all TYPE declarations to precede the procedures. *)
    BytePeekDesc* = RECORD (Stores.StoreDesc)
        first*:  INTEGER;
        second*: INTEGER;
        count*:  INTEGER
    END;
    BytePeek* = POINTER TO BytePeekDesc;

(** Open Empty.odc, allocate a typed Reader on the root store,
    read a couple of bytes, seek back to 0, re-read, close. *)
PROCEDURE BasicCursor* (): INTEGER;
    VAR doc, root: INTEGER;
        r: Stores.Reader;
        b0, b0again: BYTE;
        b1: BYTE;
BEGIN
    doc := Stores.OpenDocument("Mod/Tests/_fixtures/Empty.odc");
    IF doc = 0 THEN RETURN 0 END;
    root := Stores.RootStore(doc);
    IF root = 0 THEN Stores.CloseDocument(doc); RETURN 0 END;
    IF Stores.GetBodyLen(root) < 2 THEN Stores.CloseDocument(doc); RETURN 0 END;

    Stores.NewReader(root, r);
    IF r.handle = 0 THEN Stores.CloseDocument(doc); RETURN 0 END;
    IF r.eof THEN r.Close(); Stores.CloseDocument(doc); RETURN 0 END;
    IF r.Pos() # 0 THEN r.Close(); Stores.CloseDocument(doc); RETURN 0 END;

    r.ReadByte(b0);
    IF r.Pos() # 1 THEN r.Close(); Stores.CloseDocument(doc); RETURN 0 END;

    r.ReadByte(b1);
    IF r.Pos() # 2 THEN r.Close(); Stores.CloseDocument(doc); RETURN 0 END;

    r.SetPos(0);
    IF r.Pos() # 0 THEN r.Close(); Stores.CloseDocument(doc); RETURN 0 END;

    r.ReadByte(b0again);
    IF b0again # b0 THEN r.Close(); Stores.CloseDocument(doc); RETURN 0 END;

    (* Suppress unused-variable warnings — the read is the test. *)
    IF b1 < 0 THEN RETURN 0 END;

    r.Close();
    Stores.CloseDocument(doc);
    RETURN 1
END BasicCursor;

(** Read past the body's end through the typed surface; eof must
    transition to TRUE and subsequent reads must be no-ops. *)
PROCEDURE EofTransitions* (): INTEGER;
    VAR doc, root: INTEGER;
        r: Stores.Reader;
        b: BYTE; bodyLen: INTEGER;
BEGIN
    doc := Stores.OpenDocument("Mod/Tests/_fixtures/Empty.odc");
    IF doc = 0 THEN RETURN 0 END;
    root := Stores.RootStore(doc);
    bodyLen := Stores.GetBodyLen(root);
    IF bodyLen <= 0 THEN Stores.CloseDocument(doc); RETURN 0 END;

    Stores.NewReader(root, r);
    IF r.handle = 0 THEN Stores.CloseDocument(doc); RETURN 0 END;

    r.SetPos(bodyLen);
    IF ~r.eof THEN r.Close(); Stores.CloseDocument(doc); RETURN 0 END;

    (* Read past end is a no-op; b stays at its initial 0. *)
    b := 99;
    r.ReadByte(b);
    IF b # 0 THEN r.Close(); Stores.CloseDocument(doc); RETURN 0 END;

    r.Close();
    Stores.CloseDocument(doc);
    RETURN 1
END EofTransitions;

(** ReadBytes via the typed surface: a buffer must come back filled
    with the same bytes byte-by-byte ReadByte would yield. *)
PROCEDURE BulkReadMatchesByteByByte* (): INTEGER;
    CONST N = 8;
    VAR doc, root: INTEGER;
        r1, r2: Stores.Reader;
        i: INTEGER;
        bbuf: ARRAY N OF BYTE;
        single: ARRAY N OF BYTE;
BEGIN
    doc := Stores.OpenDocument("Mod/Tests/_fixtures/Empty.odc");
    IF doc = 0 THEN RETURN 0 END;
    root := Stores.RootStore(doc);
    IF Stores.GetBodyLen(root) < N THEN Stores.CloseDocument(doc); RETURN 0 END;

    Stores.NewReader(root, r1);
    IF r1.handle = 0 THEN Stores.CloseDocument(doc); RETURN 0 END;
    i := 0;
    WHILE i < N DO r1.ReadByte(single[i]); INC(i) END;
    r1.Close();

    Stores.NewReader(root, r2);
    IF r2.handle = 0 THEN Stores.CloseDocument(doc); RETURN 0 END;
    r2.ReadBytes(bbuf, N);
    IF r2.Pos() # N THEN r2.Close(); Stores.CloseDocument(doc); RETURN 0 END;
    i := 0;
    WHILE i < N DO
        IF bbuf[i] # single[i] THEN r2.Close(); Stores.CloseDocument(doc); RETURN 0 END;
        INC(i)
    END;
    r2.Close();
    Stores.CloseDocument(doc);
    RETURN 1
END BulkReadMatchesByteByByte;

(* --- Typed subclass + Internalize dispatch (S2 slice 2B) -------------- *)

(** Override of Stores.StoreDesc.Internalize.  Reads two bytes
    from the body and records them; tracks how many were actually
    available so the test can distinguish full-success from
    truncated reads. *)
PROCEDURE (p: BytePeekDesc) Internalize* (VAR rd: Stores.Reader);
    VAR b: BYTE;
BEGIN
    p.count := 0;
    rd.ReadByte(b);
    IF rd.eof THEN RETURN END;
    p.first := b;
    p.count := 1;
    rd.ReadByte(b);
    IF rd.eof THEN RETURN END;
    p.second := b;
    p.count := 2
END Internalize;

(** Allocate a BytePeek, dispatch through the abstract Internalize
    of Stores.StoreDesc on Empty.odc's root store, and verify
    the override ran (count = 2 means both bytes landed). *)
PROCEDURE InternalizeDispatches* (): INTEGER;
    VAR doc, root: INTEGER;
        p: BytePeek;
        eof: BOOLEAN;
        expected0, expected1: BYTE;
        rd: Stores.Reader;
BEGIN
    doc := Stores.OpenDocument("Mod/Tests/_fixtures/Empty.odc");
    IF doc = 0 THEN RETURN 0 END;
    root := Stores.RootStore(doc);
    IF root = 0 THEN Stores.CloseDocument(doc); RETURN 0 END;
    IF Stores.GetBodyLen(root) < 2 THEN Stores.CloseDocument(doc); RETURN 0 END;

    (* Read the two expected bytes through the flat surface for
       comparison — the typed dispatch must produce the same
       values when reading the same store. *)
    Stores.NewReader(root, rd);
    IF rd.handle = 0 THEN Stores.CloseDocument(doc); RETURN 0 END;
    rd.ReadByte(expected0);
    rd.ReadByte(expected1);
    rd.Close();

    NEW(p);
    eof := Stores.InternalizeFrom(root, p);
    IF p.count # 2 THEN Stores.CloseDocument(doc); RETURN 0 END;
    IF p.first # expected0 THEN Stores.CloseDocument(doc); RETURN 0 END;
    IF p.second # expected1 THEN Stores.CloseDocument(doc); RETURN 0 END;
    (* eof may be TRUE or FALSE depending on the body length. The
       contract is just that the read landed; the flag is informational. *)
    IF eof THEN END;

    Stores.CloseDocument(doc);
    RETURN 1
END InternalizeDispatches;

(** Negative path: InternalizeFrom on an invalid (NIL) source store
    must report eof = TRUE without trapping. *)
PROCEDURE InternalizeFromNilStoreSetsEof* (): INTEGER;
    VAR p: BytePeek;
BEGIN
    NEW(p);
    p.count := 99;
    IF ~Stores.InternalizeFrom(0, p) THEN RETURN 0 END;
    (* Internalize was not dispatched (NewReader returned NIL),
       so count stays at the sentinel value. *)
    IF p.count # 99 THEN RETURN 0 END;
    RETURN 1
END InternalizeFromNilStoreSetsEof;

(* --- Factory paths (S2 slice 2C) -------------------------------------- *)

(** SplitQualifiedName accepts well-formed names and rejects bad ones. *)
PROCEDURE SplitNameRoundTrips* (): INTEGER;
    VAR modName, typeName: ARRAY 64 OF CHAR;
        ok: BOOLEAN;
BEGIN
    ok := Stores.SplitQualifiedName("Foo.Bar", modName, typeName);
    IF ~ok THEN RETURN 0 END;
    IF (modName # "Foo") OR (typeName # "Bar") THEN RETURN 0 END;

    ok := Stores.SplitQualifiedName("HostStoresProbe.BytePeekDesc",
                                        modName, typeName);
    IF ~ok THEN RETURN 0 END;
    IF modName # "HostStoresProbe" THEN RETURN 0 END;
    IF typeName # "BytePeekDesc" THEN RETURN 0 END;

    ok := Stores.SplitQualifiedName("NoDotHere", modName, typeName);
    IF ok THEN RETURN 0 END;

    ok := Stores.SplitQualifiedName(".OnlyType", modName, typeName);
    IF ok THEN RETURN 0 END;

    ok := Stores.SplitQualifiedName("OnlyMod.", modName, typeName);
    IF ok THEN RETURN 0 END;

    RETURN 1
END SplitNameRoundTrips;

(** NewStoreByName allocates a real BytePeek when the qualified
    name resolves to a registered TypeDesc.  Verifies the
    Kernel.ThisMod + ThisType + NewObj chain end-to-end. *)
PROCEDURE NewStoreByNameAllocates* (): INTEGER;
    VAR s: Stores.Store; t1, t2: Kernel.Type;
        bp: BytePeek;
BEGIN
    s := Stores.NewStoreByName("HostStoresProbe.BytePeekDesc");
    IF s = NIL THEN RETURN 0 END;

    (* The runtime type tag must match a freshly NEW'd BytePeek. *)
    NEW(bp);
    t1 := Kernel.TypeOf(s);
    t2 := Kernel.TypeOf(bp);
    IF (t1 = NIL) OR (t1 # t2) THEN RETURN 0 END;
    RETURN 1
END NewStoreByNameAllocates;

(** Negative paths: malformed name, unknown module, unknown type. *)
PROCEDURE NewStoreByNameRejectsBadInput* (): INTEGER;
BEGIN
    IF Stores.NewStoreByName("NoDotHere") # NIL THEN RETURN 0 END;
    IF Stores.NewStoreByName("ModuleThatDoesNotExist.X") # NIL THEN RETURN 0 END;
    IF Stores.NewStoreByName("Kernel.NoSuchType") # NIL THEN RETURN 0 END;
    RETURN 1
END NewStoreByNameRejectsBadInput;

(** NewLikeOf clones the runtime type of a template instance.
    The fresh allocation has the same TypeOf and is a distinct
    object from the template. *)
PROCEDURE NewLikeOfClonesType* (): INTEGER;
    VAR template: BytePeek; clone: Stores.Store;
BEGIN
    NEW(template);
    template.first := 11;
    template.second := 22;
    template.count := 33;

    clone := Stores.NewLikeOf(template);
    IF clone = NIL THEN RETURN 0 END;
    IF Kernel.TypeOf(clone) # Kernel.TypeOf(template) THEN RETURN 0 END;

    (* Clone is a fresh zero-filled instance, not the template itself. *)
    IF clone = template THEN RETURN 0 END;
    RETURN 1
END NewLikeOfClonesType;

(** End-to-end NewStore: the root store of Empty.odc has a wire
    type-name (typically "Documents.StdDocument") that NewCP
    doesn't have a CP record for yet.  NewStore must therefore
    return NIL — *cleanly*, not by trapping. *)
PROCEDURE NewStoreOnUnknownTypeReturnsNil* (): INTEGER;
    VAR doc, root: INTEGER;
        s: Stores.Store;
BEGIN
    doc := Stores.OpenDocument("Mod/Tests/_fixtures/Empty.odc");
    IF doc = 0 THEN RETURN 0 END;
    root := Stores.RootStore(doc);
    IF root = 0 THEN Stores.CloseDocument(doc); RETURN 0 END;

    s := Stores.NewStore(root);
    Stores.CloseDocument(doc);
    IF s # NIL THEN RETURN 0 END;
    RETURN 1
END NewStoreOnUnknownTypeReturnsNil;

(** End-to-end typed load: open a synthetic `.odc` whose root
    store carries our own qualified type name, run NewStore which
    looks up the type, allocates a typed instance, and dispatches
    Internalize.  Verify the field values match the body bytes
    the test harness wrote. *)
PROCEDURE TypedLoadFromSyntheticOdc* (): INTEGER;
    VAR doc, root: INTEGER;
        s: Stores.Store;
        bp: BytePeek;
BEGIN
    doc := Stores.OpenDocument("Mod/Tests/_fixtures/Synthetic.odc");
    IF doc = 0 THEN RETURN 0 END;
    root := Stores.RootStore(doc);
    IF root = 0 THEN Stores.CloseDocument(doc); RETURN 0 END;

    s := Stores.NewStore(root);
    Stores.CloseDocument(doc);
    IF s = NIL THEN RETURN 0 END;

    (* Type-guard down to the concrete subclass and read the
       fields its Internalize populated. *)
    bp := s(BytePeek);
    IF bp.count # 2 THEN RETURN 0 END;
    IF bp.first # 17 THEN RETURN 0 END;     (* test harness wrote 17, 42 *)
    IF bp.second # 42 THEN RETURN 0 END;
    RETURN 1
END TypedLoadFromSyntheticOdc;

END HostStoresProbe.
