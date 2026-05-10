MODULE XmodPassthroughCaller;
(* Calls the wrapped open-array forwarder. Returns 1 if the
   forwarder returns a non-zero handle for an existing fixture,
   0 otherwise. *)

IMPORT XmodPassthrough, StoresSys;

PROCEDURE Run* (): INTEGER;
  VAR h: INTEGER;
BEGIN
  h := XmodPassthrough.OpenDoc("Mod/Tests/_fixtures/Empty.odc");
  IF h <= 0 THEN RETURN 0 END;
  StoresSys.CloseDocument(h);
  RETURN 1
END Run;

END XmodPassthroughCaller.
