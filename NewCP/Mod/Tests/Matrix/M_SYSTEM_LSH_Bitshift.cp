MODULE M_SYSTEM_LSH_Bitshift;
    IMPORT SYSTEM;

    PROCEDURE Run* (): INTEGER;
        VAR a: INTEGER;
    BEGIN
        a := SYSTEM.LSH(1, 8);     (* 1 << 8 = 256 *)
        RETURN a
    END Run;
END M_SYSTEM_LSH_Bitshift.
