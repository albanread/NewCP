MODULE XmodMissingExport;
(* Negative test for the cross-module fall-through bug.

   Stores.cp DEFINITION declares OpenDocument but does NOT
   declare anything called `DefinitelyNotAnExport`. Sema must
   emit a "module X has no exported declaration named Y"
   diagnostic instead of silently letting the call through and
   tripping the codegen cast emit later.

   This file is consumed only as a sema-only fixture by the
   integration test; it intentionally does NOT compile cleanly. *)

IMPORT Stores;

PROCEDURE Run* (): INTEGER;
  VAR x: INTEGER;
BEGIN
  x := Stores.DefinitelyNotAnExport(0);
  RETURN x
END Run;

END XmodMissingExport.
