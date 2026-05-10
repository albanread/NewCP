MODULE M_Stmt_LOOP_EXIT_Nested;
    PROCEDURE Run* (): INTEGER;
        VAR outer, inner, count: INTEGER;
    BEGIN
        outer := 0; count := 0;
        LOOP
            inner := 0;
            LOOP
                INC(inner); INC(count);
                IF inner >= 3 THEN EXIT END
            END;
            INC(outer);
            IF outer >= 3 THEN EXIT END
        END;
        (* outer runs 3 times, inner runs 3 times each → count = 9; outer = 3.
           Pack: outer*10 + (count - 8) = 30 + 1 = 31... wait recompute. *)
        RETURN outer + count - 1        (* 3 + 9 - 1 = 11 *)
    END Run;
END M_Stmt_LOOP_EXIT_Nested.
