MODULE StoresProbe;
(* Smoke probe for Stores Stage S1 — read-only envelope walk.

   The integration test stages an `.odc` fixture at a known path
   before invoking these procedures. We hard-code the fixture
   path because run_function-style invocations are parameterless;
   the test scaffolding owns the path.

   What the probes verify:
   - OpenDocument returns a non-NIL handle for a valid `.odc`.
   - RootStore returns a non-NIL store handle.
   - GetTypeName + GetKind + GetBodyLen on the root all return
     plausible values (root is conventionally a `Documents.StdDocument`,
     wire kind = KindStore or KindElem, body length > 0).
   - FirstChild reaches at least one child (the wrapped TextView).
   - CloseDocument invalidates outstanding store handles.
   - Negative paths: invalid handles return 0 / empty.
*)

IMPORT Stores;

(** Open a valid `.odc`, root must be non-NIL with a sane shape,
    close cleanly. Returns 1 on full success, 0 otherwise. *)
PROCEDURE OpenAndWalkEmpty*(): INTEGER;
  VAR doc, root, child: INTEGER;
      kind, bodyLen: INTEGER;
      name: ARRAY 256 OF CHAR;
BEGIN
  doc := Stores.OpenDocument("Mod/Tests/_fixtures/Empty.odc");
  IF doc = 0 THEN RETURN 0 END;

  root := Stores.RootStore(doc);
  IF root = 0 THEN
    Stores.CloseDocument(doc);
    RETURN 0
  END;

  kind := Stores.GetKind(root);
  IF (kind # Stores.KindStore) & (kind # Stores.KindElem) THEN
    Stores.CloseDocument(doc);
    RETURN 0
  END;

  bodyLen := Stores.GetBodyLen(root);
  IF bodyLen <= 0 THEN
    Stores.CloseDocument(doc);
    RETURN 0
  END;

  Stores.GetTypeName(root, name);
  IF name[0] = 0X THEN                        (* type name must be non-empty *)
    Stores.CloseDocument(doc);
    RETURN 0
  END;

  child := Stores.FirstChild(root);
  IF child = 0 THEN
    Stores.CloseDocument(doc);
    RETURN 0
  END;

  Stores.CloseDocument(doc);

  (* After close, store handles invalidate — body length goes to 0. *)
  IF Stores.GetBodyLen(root) # 0 THEN RETURN 0 END;

  RETURN 1
END OpenAndWalkEmpty;

(** Negative path: missing file returns NIL. *)
PROCEDURE OpenMissingFails*(): INTEGER;
  VAR doc: INTEGER;
BEGIN
  doc := Stores.OpenDocument("Mod/Tests/_fixtures/DefinitelyDoesNotExist.odc");
  IF doc # 0 THEN RETURN 0 END;
  RETURN 1
END OpenMissingFails;

(** Negative path: invalid handles return 0 / empty. *)
PROCEDURE InvalidHandlesReturnZero*(): INTEGER;
  VAR name: ARRAY 32 OF CHAR;
BEGIN
  IF Stores.RootStore(0) # 0 THEN RETURN 0 END;
  IF Stores.FirstChild(0) # 0 THEN RETURN 0 END;
  IF Stores.NextSibling(0) # 0 THEN RETURN 0 END;
  IF Stores.GetBodyLen(0) # 0 THEN RETURN 0 END;
  IF Stores.GetKind(0) # Stores.KindNil THEN RETURN 0 END;
  Stores.GetTypeName(0, name);
  IF name[0] # 0X THEN RETURN 0 END;
  RETURN 1
END InvalidHandlesReturnZero;

(** Open a Reader on the root store and exercise the cursor:
    the cursor starts at 0, advances one byte per ReadByte, and
    ReaderSetPos seeks back to the start. Returns 1 on success. *)
PROCEDURE ReaderBasicCursor*(): INTEGER;
  VAR doc, root, r: INTEGER;
      bodyLen, b0, b1, posAfter, posReset: INTEGER;
BEGIN
  doc := Stores.OpenDocument("Mod/Tests/_fixtures/Empty.odc");
  IF doc = 0 THEN RETURN 0 END;
  root := Stores.RootStore(doc);
  IF root = 0 THEN Stores.CloseDocument(doc); RETURN 0 END;
  bodyLen := Stores.GetBodyLen(root);
  IF bodyLen < 2 THEN Stores.CloseDocument(doc); RETURN 0 END;

  r := Stores.OpenReader(root);
  IF r = 0 THEN Stores.CloseDocument(doc); RETURN 0 END;
  IF Stores.ReaderPos(r) # 0 THEN
    Stores.CloseReader(r); Stores.CloseDocument(doc); RETURN 0
  END;
  IF Stores.ReaderEof(r) # 0 THEN
    Stores.CloseReader(r); Stores.CloseDocument(doc); RETURN 0
  END;

  b0 := Stores.ReaderReadByte(r);
  posAfter := Stores.ReaderPos(r);
  IF posAfter # 1 THEN
    Stores.CloseReader(r); Stores.CloseDocument(doc); RETURN 0
  END;

  b1 := Stores.ReaderReadByte(r);
  IF Stores.ReaderPos(r) # 2 THEN
    Stores.CloseReader(r); Stores.CloseDocument(doc); RETURN 0
  END;

  Stores.ReaderSetPos(r, 0);
  posReset := Stores.ReaderPos(r);
  IF posReset # 0 THEN
    Stores.CloseReader(r); Stores.CloseDocument(doc); RETURN 0
  END;
  (* Re-reading the first byte must yield the same value. *)
  IF Stores.ReaderReadByte(r) # b0 THEN
    Stores.CloseReader(r); Stores.CloseDocument(doc); RETURN 0
  END;

  (* Suppress unused-variable warning. *)
  IF b1 < 0 THEN RETURN 0 END;

  Stores.CloseReader(r);
  Stores.CloseDocument(doc);
  RETURN 1
END ReaderBasicCursor;

(** Seek to body_len → Eof asserts. ReaderSetPos clamps to body
    bounds, so a deliberate over-seek lands at body_len. *)
PROCEDURE ReaderEofAtEnd*(): INTEGER;
  VAR doc, root, r: INTEGER;
      bodyLen: INTEGER;
BEGIN
  doc := Stores.OpenDocument("Mod/Tests/_fixtures/Empty.odc");
  IF doc = 0 THEN RETURN 0 END;
  root := Stores.RootStore(doc);
  IF root = 0 THEN Stores.CloseDocument(doc); RETURN 0 END;
  bodyLen := Stores.GetBodyLen(root);
  IF bodyLen <= 0 THEN Stores.CloseDocument(doc); RETURN 0 END;

  r := Stores.OpenReader(root);
  IF r = 0 THEN Stores.CloseDocument(doc); RETURN 0 END;

  Stores.ReaderSetPos(r, bodyLen);
  IF Stores.ReaderEof(r) # 1 THEN
    Stores.CloseReader(r); Stores.CloseDocument(doc); RETURN 0
  END;
  IF Stores.ReaderPos(r) # bodyLen THEN
    Stores.CloseReader(r); Stores.CloseDocument(doc); RETURN 0
  END;

  (* Clamp on over-seek. *)
  Stores.ReaderSetPos(r, bodyLen + 1000);
  IF Stores.ReaderPos(r) # bodyLen THEN
    Stores.CloseReader(r); Stores.CloseDocument(doc); RETURN 0
  END;

  Stores.CloseReader(r);
  Stores.CloseDocument(doc);
  RETURN 1
END ReaderEofAtEnd;

(** Read N bytes through ReadBytes; the slice we get back must
    match the byte-by-byte read at the same offset. *)
PROCEDURE ReaderReadBytesMatchesByteByByte*(): INTEGER;
  CONST N = 8;
  VAR doc, root, r1, r2: INTEGER;
      bodyLen, i, got: INTEGER;
      bbuf: ARRAY N OF BYTE;
      single: ARRAY N OF INTEGER;
BEGIN
  doc := Stores.OpenDocument("Mod/Tests/_fixtures/Empty.odc");
  IF doc = 0 THEN RETURN 0 END;
  root := Stores.RootStore(doc);
  IF root = 0 THEN Stores.CloseDocument(doc); RETURN 0 END;
  bodyLen := Stores.GetBodyLen(root);
  IF bodyLen < N THEN Stores.CloseDocument(doc); RETURN 0 END;

  (* Reader 1: byte-by-byte. *)
  r1 := Stores.OpenReader(root);
  IF r1 = 0 THEN Stores.CloseDocument(doc); RETURN 0 END;
  i := 0;
  WHILE i < N DO
    single[i] := Stores.ReaderReadByte(r1);
    INC(i)
  END;
  Stores.CloseReader(r1);

  (* Reader 2: bulk ReadBytes. *)
  r2 := Stores.OpenReader(root);
  IF r2 = 0 THEN Stores.CloseDocument(doc); RETURN 0 END;
  got := Stores.ReaderReadBytes(r2, bbuf, N);
  IF got # N THEN
    Stores.CloseReader(r2); Stores.CloseDocument(doc); RETURN 0
  END;
  IF Stores.ReaderPos(r2) # N THEN
    Stores.CloseReader(r2); Stores.CloseDocument(doc); RETURN 0
  END;
  i := 0;
  WHILE i < N DO
    IF single[i] # bbuf[i] THEN
      Stores.CloseReader(r2); Stores.CloseDocument(doc); RETURN 0
    END;
    INC(i)
  END;

  Stores.CloseReader(r2);
  Stores.CloseDocument(doc);
  RETURN 1
END ReaderReadBytesMatchesByteByByte;

(** OpenReader on an invalid store handle returns 0; reads on an
    invalid reader return 0 / EOF. *)
PROCEDURE InvalidReaderHandlesReturnZero*(): INTEGER;
  VAR bbuf: ARRAY 4 OF BYTE;
BEGIN
  IF Stores.OpenReader(0) # 0 THEN RETURN 0 END;
  IF Stores.ReaderPos(0) # 0 THEN RETURN 0 END;
  IF Stores.ReaderEof(0) # 1 THEN RETURN 0 END;
  IF Stores.ReaderReadByte(0) # 0 THEN RETURN 0 END;
  IF Stores.ReaderReadInt(0) # 0 THEN RETURN 0 END;
  IF Stores.ReaderReadXInt(0) # 0 THEN RETURN 0 END;
  IF Stores.ReaderReadLong(0) # 0 THEN RETURN 0 END;
  IF Stores.ReaderReadBool(0) # 0 THEN RETURN 0 END;
  IF Stores.ReaderReadBytes(0, bbuf, 4) # 0 THEN RETURN 0 END;
  RETURN 1
END InvalidReaderHandlesReturnZero;

END StoresProbe.
