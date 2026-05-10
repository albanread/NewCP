MODULE M_Stmt_CASE_Without_ELSE;
    PROCEDURE Run* (): INTEGER;
        VAR x: INTEGER;
    BEGIN
        x := 0;
        CASE 2 OF
          1: x := 1
        | 2: x := 5
        | 3: x := 9
        END;
        RETURN x
    END Run;
END M_Stmt_CASE_Without_ELSE.
