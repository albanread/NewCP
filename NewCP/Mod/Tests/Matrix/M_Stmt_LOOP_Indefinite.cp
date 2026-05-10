MODULE M_Stmt_LOOP_Indefinite;
    PROCEDURE Run* (): INTEGER;
        VAR n: INTEGER;
    BEGIN
        n := 0;
        LOOP
            INC(n);
            IF n >= 10 THEN EXIT END
        END;
        RETURN n
    END Run;
END M_Stmt_LOOP_Indefinite.
