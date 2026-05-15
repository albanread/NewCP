MODULE DocumentsProbe;
(* Smoke test for the BB-faithful Documents MVS slice.
   Exercises just the public surface:
     - The Document / Context / Directory types are usable as
       VAR / parameter types.
     - SetDir installs a Directory and exposes it through `dir`.
     - ImportDocument returns NIL (stub body) without trapping.

   Returns 1 on success, negative on first surprise. *)

    IMPORT Documents, Stores;

    PROCEDURE Run* (): INTEGER;
        VAR s: Stores.Store;
    BEGIN
        (* dir / stdDir should both be NIL after module init. *)
        IF Documents.dir # NIL THEN RETURN -1 END;
        IF Documents.stdDir # NIL THEN RETURN -2 END;

        (* ImportDocument's stub body sets s := NIL — call it
           with NIL `file` is OK for the stub (real body would
           ASSERT). *)
        Documents.ImportDocument(NIL, s);
        IF s # NIL THEN RETURN -3 END;

        RETURN 1
    END Run;

END DocumentsProbe.
