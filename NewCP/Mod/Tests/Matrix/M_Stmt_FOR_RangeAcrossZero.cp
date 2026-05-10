MODULE M_Stmt_FOR_RangeAcrossZero;
    PROCEDURE Run* (): INTEGER;
        VAR i, sum: INTEGER;
    BEGIN
        sum := 0;
        FOR i := -3 TO 3 DO sum := sum + i END;
        RETURN sum                            (* -3-2-1+0+1+2+3 = 0 *)
    END Run;
END M_Stmt_FOR_RangeAcrossZero.
