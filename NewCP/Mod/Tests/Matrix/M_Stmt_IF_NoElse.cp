MODULE M_Stmt_IF_NoElse;
    PROCEDURE Run* (): INTEGER;
        VAR x: INTEGER;
    BEGIN
        x := 7;
        IF x < 0 THEN x := 999 END;        (* skipped *)
        IF x > 5 THEN x := x END;           (* no change *)
        RETURN x
    END Run;
END M_Stmt_IF_NoElse.
