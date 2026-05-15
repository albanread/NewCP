MODULE XmodMissingField;
(* Negative repro for the sema field-lookup gap on cross-module
    records.  HostStores.Reader resolves cross-module to the real
    `Stores.Reader` record shape, but it still does not declare the
    field below.  Sema must reject the missing selector rather than
    silently accepting it.

   Used as a sema-only fixture by an integration test; this file
   intentionally does NOT compile cleanly. *)

IMPORT HostStores;

PROCEDURE Run* (): INTEGER;
    VAR p: HostStores.Reader;
BEGIN
    RETURN p.thisFieldDoesNotExist
END Run;

END XmodMissingField.
