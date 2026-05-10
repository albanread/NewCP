MODULE M_Type_Alias_Chain;
    TYPE
        A = INTEGER;
        B = A;
        C = B;

    PROCEDURE Run* (): INTEGER;
        VAR x: A; y: B; z: C; r: INTEGER;
    BEGIN
        x := 10;
        y := x;
        z := y * 10;
        r := z;
        RETURN r                              (* 100 *)
    END Run;
END M_Type_Alias_Chain.
