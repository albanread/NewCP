MODULE M_Stmt_CASE_AsValue;
    PROCEDURE Run* (): INTEGER;
        VAR n, result: INTEGER;
    BEGIN
        n := 3;
        CASE n OF
          1: result := 10
        | 2: result := 20
        | 3: result := 30
        ELSE result := 0
        END;
        RETURN result
    END Run;
END M_Stmt_CASE_AsValue.
