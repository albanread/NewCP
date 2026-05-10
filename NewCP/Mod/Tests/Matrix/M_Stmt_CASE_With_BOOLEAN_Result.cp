MODULE M_Stmt_CASE_With_BOOLEAN_Result;
    PROCEDURE Run* (): INTEGER;
        VAR n: INTEGER; flag: BOOLEAN;
    BEGIN
        n := 3;
        flag := FALSE;
        CASE n OF
          0:        flag := FALSE
        | 1, 2, 3:  flag := TRUE
        ELSE        flag := FALSE
        END;
        IF flag THEN RETURN 1 ELSE RETURN 0 END
    END Run;
END M_Stmt_CASE_With_BOOLEAN_Result.
