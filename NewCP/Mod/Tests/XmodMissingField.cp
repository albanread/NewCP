MODULE XmodMissingField;
(* Negative repro for the sema field-lookup gap on cross-module
   records.  Stores.cp DEFINITION declares a `Reader` integer
   alias but no field named `notARealField` on it (Reader is a
   plain INTEGER alias).  Sema must reject the field access.

   Used as a sema-only fixture by an integration test; this file
   intentionally does NOT compile cleanly. *)

IMPORT HostStores;

PROCEDURE Run* (): INTEGER;
    VAR p: HostStores.Reader;
BEGIN
    NEW(p);
    RETURN p.thisFieldDoesNotExist
END Run;

END XmodMissingField.
