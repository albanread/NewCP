MODULE M_Expr_LogicalOr_ShortCircuit;
    VAR sideEffect: INTEGER;

    PROCEDURE Touch (): BOOLEAN;
    BEGIN INC(sideEffect); RETURN TRUE END Touch;

    PROCEDURE Run* (): INTEGER;
    BEGIN
        sideEffect := 0;
        (* TRUE OR Touch() — Touch must NOT be called *)
        IF TRUE OR Touch() THEN END;
        IF sideEffect # 0 THEN RETURN -1 END;
        (* FALSE OR Touch() — Touch IS called *)
        IF FALSE OR Touch() THEN END;
        RETURN sideEffect      (* 1 *)
    END Run;
END M_Expr_LogicalOr_ShortCircuit.
