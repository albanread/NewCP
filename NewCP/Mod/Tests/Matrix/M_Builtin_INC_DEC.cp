MODULE M_Builtin_INC_DEC;
    PROCEDURE Run* (): INTEGER;
        VAR i: INTEGER;
    BEGIN
        i := 10;
        INC(i);          (* 11 *)
        INC(i, 5);       (* 16 *)
        DEC(i);          (* 15 *)
        DEC(i, 2);       (* 13 *)
        RETURN i
    END Run;
END M_Builtin_INC_DEC.
