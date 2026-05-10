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

END StoresProbe.
