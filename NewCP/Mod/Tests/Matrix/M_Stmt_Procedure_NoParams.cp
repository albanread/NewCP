MODULE M_Stmt_Procedure_NoParams;
    VAR counter: INTEGER;

    PROCEDURE Bump;
    BEGIN INC(counter, 50) END Bump;

    PROCEDURE Run* (): INTEGER;
    BEGIN
        counter := 0;
        Bump;          (* bare-name call *)
        Bump();        (* same proc, parenthesised *)
        RETURN counter (* 100 *)
    END Run;
END M_Stmt_Procedure_NoParams.
