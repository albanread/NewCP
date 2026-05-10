MODULE M_Expr_INC_OnRecord_Field;
    TYPE Counter = RECORD count: INTEGER END;

    PROCEDURE Run* (): INTEGER;
        VAR c: Counter;
    BEGIN
        c.count := 40;
        INC(c.count, 10);
        RETURN c.count
    END Run;
END M_Expr_INC_OnRecord_Field.
