MODULE XmodMissingField;
(* Negative repro for the sema field-lookup gap on cross-module
    records.  `Stores.Reader` is a real imported record type, but it
    still does not declare the field below.  Sema must reject the
    missing selector rather than silently accepting it.

   Used as a sema-only fixture by an integration test; this file
   intentionally does NOT compile cleanly. *)

IMPORT Stores;

PROCEDURE Run* (): INTEGER;
    VAR p: Stores.Reader;
BEGIN
    RETURN p.thisFieldDoesNotExist
END Run;

END XmodMissingField.
