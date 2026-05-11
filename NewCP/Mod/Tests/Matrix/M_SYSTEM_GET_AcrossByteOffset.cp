MODULE M_SYSTEM_GET_AcrossByteOffset;
    IMPORT SYSTEM;

    PROCEDURE Run* (): INTEGER;
        VAR arr: ARRAY 16 OF BYTE; n, r: INTEGER;
    BEGIN
        n := 12345;
        SYSTEM.PUT(SYSTEM.ADR(arr[4]), n);
        SYSTEM.GET(SYSTEM.ADR(arr[4]), r);
        RETURN r                                      (* 12345 *)
    END Run;
END M_SYSTEM_GET_AcrossByteOffset.
