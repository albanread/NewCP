MODULE M_Stmt_Nested_IF_Inside_For;
    PROCEDURE Run* (): INTEGER;
        VAR i, evens: INTEGER;
    BEGIN
        evens := 0;
        FOR i := 1 TO 5 DO
            IF ~ODD(i) THEN evens := evens + i END
        END;
        RETURN evens                          (* 2 + 4 = 6 *)
    END Run;
END M_Stmt_Nested_IF_Inside_For.
