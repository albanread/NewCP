MODULE M_Expr_LogicalAnd_ShortCircuit;
    VAR sideEffect: INTEGER;

    PROCEDURE Touch (): BOOLEAN;
    BEGIN INC(sideEffect); RETURN TRUE END Touch;

    PROCEDURE Run* (): INTEGER;
    BEGIN
        sideEffect := 0;
        (* FALSE & Touch() — Touch must NOT be called *)
        IF FALSE & Touch() THEN END;
        IF sideEffect # 0 THEN RETURN -1 END;
        (* TRUE & Touch() — Touch IS called *)
        IF TRUE & Touch() THEN END;
        RETURN sideEffect      (* 1 = Touch called once total *)
    END Run;
END M_Expr_LogicalAnd_ShortCircuit.
