MODULE MetaProbe;
(* Smoke test for the BB-faithful Meta MVS slice — just creates an
   Item via Meta.LookupPath (which currently returns undef) and
   verifies the Item's obj field is undef. *)

    IMPORT Meta;

    PROCEDURE Run* (): INTEGER;
        VAR i: Meta.Item;
    BEGIN
        Meta.LookupPath("Documents.ImportDocument", i);
        IF i.obj = Meta.undef THEN
            RETURN 1
        ELSE
            RETURN -1
        END
    END Run;

END MetaProbe.
