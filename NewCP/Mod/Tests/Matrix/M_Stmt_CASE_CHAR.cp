MODULE M_Stmt_CASE_CHAR;
    PROCEDURE Classify (c: CHAR): INTEGER;
    BEGIN
        CASE c OF
          "0".."9": RETURN 10
        | "A".."Z": RETURN 100
        | "a".."z": RETURN 1
        ELSE        RETURN 0
        END
    END Classify;

    PROCEDURE Run* (): INTEGER;
    BEGIN
        (* 'M' → 100, '7' → 10, 'q' → 1, '?' → 0 = 100+10+1+0 = 111;
           multiplied by 3 = 333 *)
        RETURN (Classify("M") + Classify("7") + Classify("q") + Classify("?")) * 3
    END Run;
END M_Stmt_CASE_CHAR.
