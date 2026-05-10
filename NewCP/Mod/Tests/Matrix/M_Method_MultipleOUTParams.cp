MODULE M_Method_MultipleOUTParams;
    TYPE
        BoxDesc = EXTENSIBLE RECORD a, b: INTEGER END;
        Box     = POINTER TO BoxDesc;

    PROCEDURE (b: Box) Snapshot* (OUT x: INTEGER; OUT y: INTEGER), NEW;
    BEGIN x := b.a; y := b.b END Snapshot;

    PROCEDURE Run* (): INTEGER;
        VAR b: Box; p, q: INTEGER;
    BEGIN
        NEW(b);
        b.a := 12; b.b := 23;
        b.Snapshot(p, q);
        RETURN p + q                          (* 35 *)
    END Run;
END M_Method_MultipleOUTParams.
