MODULE TextModelsProbe;
(* End-to-end load probe for the TextModels port. The Rust test
   harness stages a synthetic `.odc` whose root store has type
   tag "TextModels.StdModelDesc" and a body whose first 6 bytes
   are the version-chain placeholder, followed by a 4-byte LE
   run-list length we can verify against. *)

IMPORT Stores, TextModels, Kernel;

(** Type-resolution sanity: ThisMod("TextModels") and
    ThisType(_, "StdModelDesc") must both succeed once the
    loader has compiled this module's imports. *)
PROCEDURE TypeResolves* (): INTEGER;
    VAR m: Kernel.Module; t1, t2: Kernel.Type; sm: TextModels.StdModel;
BEGIN
    m := Kernel.ThisMod("TextModels");
    IF m = NIL THEN RETURN 0 END;
    t1 := Kernel.ThisType(m, "StdModelDesc");
    IF t1 = NIL THEN RETURN 0 END;
    NEW(sm);
    t2 := Kernel.TypeOf(sm);
    IF (t2 = NIL) OR (t2 # t1) THEN RETURN 0 END;
    RETURN 1
END TypeResolves;

(** End-to-end: load synthetic .odc → NewStore → Internalize →
    typed-field read.  The harness writes bytes
    [0,1,2,3,4,5, 7,0,0,0, 0xFF, 0xFF] for the body, so the
    super-version chain is identifiable and the run-list length
    is 7. *)
PROCEDURE LoadStdModel* (): INTEGER;
    VAR doc, root: INTEGER;
    s: Stores.Store;
        sm: TextModels.StdModel;
BEGIN
    doc := Stores.OpenDocument("Mod/Tests/_fixtures/TextModelsStub.odc");
    IF doc = 0 THEN RETURN 0 END;
    root := Stores.RootStore(doc);
    IF root = 0 THEN Stores.CloseDocument(doc); RETURN 0 END;

    s := Stores.NewStore(root);
    Stores.CloseDocument(doc);
    IF s = NIL THEN RETURN 0 END;

    sm := s(TextModels.StdModel);
    IF sm.result # TextModels.OkComplete THEN RETURN 0 END;
    IF sm.runListLen # 7 THEN RETURN 0 END;
    IF sm.superVersions[0] # 0 THEN RETURN 0 END;
    IF sm.superVersions[5] # 5 THEN RETURN 0 END;
    RETURN 1
END LoadStdModel;

(** End-to-end with actual text content.  The fixture body is:

      super-versions [0..5]
      run-list length = 6
      run list: (ano=1, len=5)  (text run, 5 1-byte chars)
                (ano=0xFF, terminator)
      chars: "Hello"

    The probe asserts the decoded summary matches and the captured
    text round-trips through the CHAR buffer.  ano=1 is non-conforming
    on the first piece (real BlackBox docs always start with a NEW
    attribute) but exercises the existing-attribute branch without
    requiring an inline attribute store the decoder can't yet
    materialize. *)
PROCEDURE LoadStdModelText* (): INTEGER;
    VAR doc, root: INTEGER;
    s: Stores.Store;
        sm: TextModels.StdModel;
BEGIN
    doc := Stores.OpenDocument("Mod/Tests/_fixtures/TextModelsHello.odc");
    IF doc = 0 THEN RETURN 0 END;
    root := Stores.RootStore(doc);
    IF root = 0 THEN Stores.CloseDocument(doc); RETURN 0 END;

    s := Stores.NewStore(root);
    Stores.CloseDocument(doc);
    IF s = NIL THEN RETURN 0 END;

    sm := s(TextModels.StdModel);
    IF sm.result # TextModels.OkComplete THEN RETURN 0 END;
    IF sm.runListLen # 6 THEN RETURN 0 END;
    IF sm.textPieceCount # 1 THEN RETURN 0 END;
    IF sm.viewPieceCount # 0 THEN RETURN 0 END;
    IF sm.totalChars # 5 THEN RETURN 0 END;
    IF sm.textLen # 5 THEN RETURN 0 END;
    IF sm.text # "Hello" THEN RETURN 0 END;
    RETURN 1
END LoadStdModelText;

(** Run list with a terminator-only body produces an empty model
    with `result = OkComplete` and zero pieces.  This is the
    boundary condition where the current LoadStdModel fixture sits
    (with its garbage run-list-length); make the contract explicit. *)
PROCEDURE LoadStdModelEmpty* (): INTEGER;
    VAR doc, root: INTEGER;
    s: Stores.Store;
        sm: TextModels.StdModel;
BEGIN
    doc := Stores.OpenDocument("Mod/Tests/_fixtures/TextModelsEmpty.odc");
    IF doc = 0 THEN RETURN 0 END;
    root := Stores.RootStore(doc);
    s := Stores.NewStore(root);
    Stores.CloseDocument(doc);
    IF s = NIL THEN RETURN 0 END;

    sm := s(TextModels.StdModel);
    IF sm.result # TextModels.OkComplete THEN RETURN 0 END;
    IF sm.textPieceCount # 0 THEN RETURN 0 END;
    IF sm.totalChars # 0 THEN RETURN 0 END;
    IF sm.textLen # 0 THEN RETURN 0 END;
    RETURN 1
END LoadStdModelEmpty;

(** Walk the store tree of `s` (DFS) returning the first store
    whose qualified type name is "TextModels.StdModelDesc", or 0
    if none is found. *)
PROCEDURE FindStdModelIn (s: Stores.StoreHandle): Stores.StoreHandle;
    VAR child, found: Stores.StoreHandle; name: ARRAY 64 OF CHAR;
BEGIN
    IF s = 0 THEN RETURN 0 END;
    Stores.GetTypeName(s, name);
    IF name = "TextModels.StdModelDesc" THEN RETURN s END;
    child := Stores.FirstChild(s);
    WHILE child # 0 DO
        found := FindStdModelIn(child);
        IF found # 0 THEN RETURN found END;
        child := Stores.NextSibling(child)
    END;
    RETURN 0
END FindStdModelIn;

(** Load a real BlackBox `Empty.odc`, walk to its embedded
    TextModels.StdModelDesc, and materialize it via NewStore.
    Reports back an INTEGER summary so the test can distinguish
    "decoder couldn't find the model" from "decoder found the
    model but bailed on a known-deferred wire-format feature".

    Return value:
      1   - found a model and `Internalize` ran to OkComplete
      10  - couldn't find a TextModels.StdModelDesc in the tree
      11  - found one but NewStore returned NIL
      20+ - found, materialized, but Internalize result was
            non-OkComplete (specific code is `20 + result`,
            so e.g. 25 = OkUnsupportedNewAttr) *)
PROCEDURE LoadStdModelFromEmptyOdc* (): INTEGER;
    VAR doc, modelStore: INTEGER;
    s: Stores.Store;
        sm: TextModels.StdModel;
        rc: INTEGER;
BEGIN
    doc := Stores.OpenDocument("Mod/Tests/_fixtures/Empty.odc");
    IF doc = 0 THEN RETURN 0 END;
    modelStore := FindStdModelIn(Stores.RootStore(doc));
    IF modelStore = 0 THEN Stores.CloseDocument(doc); RETURN 10 END;

    s := Stores.NewStore(modelStore);
    Stores.CloseDocument(doc);
    IF s = NIL THEN RETURN 11 END;

    sm := s(TextModels.StdModel);
    rc := sm.result;
    IF rc = TextModels.OkComplete THEN RETURN 1
    ELSE RETURN 20 + rc
    END
END LoadStdModelFromEmptyOdc;

(** Variant of LoadStdModelFromEmptyOdc that targets an arbitrary
    fixture path — pass via the path string.  Returns the same
    code shape: 1 = OkComplete, 10 = no model found, 11 = NewStore
    failed, 20+result for non-Ok decoder outcomes. *)
PROCEDURE LoadStdModelFromTourOdc* (): INTEGER;
    VAR doc, modelStore: INTEGER;
    s: Stores.Store;
        sm: TextModels.StdModel;
        rc: INTEGER;
BEGIN
    doc := Stores.OpenDocument("Mod/Tests/_fixtures/Tour.odc");
    IF doc = 0 THEN RETURN 0 END;
    modelStore := FindStdModelIn(Stores.RootStore(doc));
    IF modelStore = 0 THEN Stores.CloseDocument(doc); RETURN 10 END;

    s := Stores.NewStore(modelStore);
    Stores.CloseDocument(doc);
    IF s = NIL THEN RETURN 11 END;

    sm := s(TextModels.StdModel);
    rc := sm.result;
    IF rc = TextModels.OkComplete THEN RETURN 1
    ELSE RETURN 20 + rc
    END
END LoadStdModelFromTourOdc;

(** Same but reports decoded summary fields when OkComplete.
    Encodes the count tuple as a single integer:
        attrPoolGrowth * 1_000_000 + textPieceCount * 1_000 + viewPieceCount.
    For a typical attributed text doc we expect attrPoolGrowth >= 1
    (the default attribute) and textPieceCount >= 1. *)
PROCEDURE TourOdcModelSummary* (): INTEGER;
    VAR doc, modelStore: INTEGER;
    s: Stores.Store;
        sm: TextModels.StdModel;
BEGIN
    doc := Stores.OpenDocument("Mod/Tests/_fixtures/Tour.odc");
    IF doc = 0 THEN RETURN -1 END;
    modelStore := FindStdModelIn(Stores.RootStore(doc));
    IF modelStore = 0 THEN Stores.CloseDocument(doc); RETURN -2 END;
    s := Stores.NewStore(modelStore);
    Stores.CloseDocument(doc);
    IF s = NIL THEN RETURN -3 END;
    sm := s(TextModels.StdModel);
    IF sm.result # TextModels.OkComplete THEN RETURN -100 - sm.result END;
    RETURN sm.attrPoolGrowth * 1000000 + sm.textPieceCount * 1000 + sm.viewPieceCount
END TourOdcModelSummary;

(** Decoded text length from Tour.odc's first text model. We
    expect a non-trivial number of CHARs to land in the text
    buffer — every 1-byte text-run piece widened to CHAR. *)
PROCEDURE TourOdcTextLength* (): INTEGER;
    VAR doc, modelStore: INTEGER;
    s: Stores.Store;
        sm: TextModels.StdModel;
BEGIN
    doc := Stores.OpenDocument("Mod/Tests/_fixtures/Tour.odc");
    IF doc = 0 THEN RETURN -1 END;
    modelStore := FindStdModelIn(Stores.RootStore(doc));
    IF modelStore = 0 THEN Stores.CloseDocument(doc); RETURN -2 END;
    s := Stores.NewStore(modelStore);
    Stores.CloseDocument(doc);
    IF s = NIL THEN RETURN -3 END;
    sm := s(TextModels.StdModel);
    IF sm.result # TextModels.OkComplete THEN RETURN -100 - sm.result END;
    RETURN sm.textLen
END TourOdcTextLength;

(** Captures the first 32 chars of Tour.odc's decoded text into a
    tiny INTEGER digest the harness can spot-check (each byte's
    codepoint summed with a rolling shift).  Quick way to assert
    the decoder produced specific bytes without piping a string
    back through the test ABI. *)
PROCEDURE TourOdcTextDigest* (): INTEGER;
    VAR doc, modelStore: INTEGER;
    s: Stores.Store;
        sm: TextModels.StdModel;
        i, digest: INTEGER;
BEGIN
    doc := Stores.OpenDocument("Mod/Tests/_fixtures/Tour.odc");
    IF doc = 0 THEN RETURN -1 END;
    modelStore := FindStdModelIn(Stores.RootStore(doc));
    IF modelStore = 0 THEN Stores.CloseDocument(doc); RETURN -2 END;
    s := Stores.NewStore(modelStore);
    Stores.CloseDocument(doc);
    IF s = NIL THEN RETURN -3 END;
    sm := s(TextModels.StdModel);
    IF sm.result # TextModels.OkComplete THEN RETURN -100 - sm.result END;

    digest := 0;
    i := 0;
    WHILE (i < 32) & (i < sm.textLen) DO
        digest := digest * 31 + ORD(sm.text[i]);
        INC(i)
    END;
    RETURN digest
END TourOdcTextDigest;

END TextModelsProbe.
