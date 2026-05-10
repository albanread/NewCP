MODULE M_Stmt_IF_With_NestedIF;
    PROCEDURE Classify (a, b: INTEGER): INTEGER;
    BEGIN
        IF a > 0 THEN
            IF b > 0 THEN
                IF a > b THEN
                    RETURN 1
                ELSE
                    RETURN 2
                END
            ELSE
                RETURN 3
            END
        ELSE
            RETURN 4
        END
    END Classify;

    PROCEDURE Run* (): INTEGER;
    BEGIN
        (* Classify(2, 3) = 2, Classify(3, 2) = 1, Classify(2, -1) = 3, Classify(-1, 5) = 4
           sum 2+1+3+4 = 10 ... offset to 5 *)
        RETURN Classify(2, 3) + Classify(3, 2) + Classify(2, -1) + Classify(-1, 5) - 5
    END Run;
END M_Stmt_IF_With_NestedIF.
