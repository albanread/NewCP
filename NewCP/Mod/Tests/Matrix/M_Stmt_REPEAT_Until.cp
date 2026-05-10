MODULE M_Stmt_REPEAT_Until;
    PROCEDURE Run* (): INTEGER;
        VAR i, count: INTEGER;
    BEGIN
        i := 0; count := 0;
        REPEAT
            INC(i);
            INC(count)
        UNTIL i >= 4;
        RETURN count
    END Run;
END M_Stmt_REPEAT_Until.
